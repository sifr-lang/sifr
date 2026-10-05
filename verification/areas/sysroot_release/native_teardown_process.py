"""Fixed harmless fixtures and forwarding observation of the production executor."""
from contextlib import contextmanager
from dataclasses import asdict
import errno
import json
import os
from pathlib import Path
import re
import selectors
import signal
import subprocess
import sys
import time

if __name__ != "__main__":
    from sifr_verify import process_execution as production

KINDS = ('waitid-capability', 'natural-output', 'deadline-term', 'resistant-pipe')
FIXTURE_SECONDS = 8
MAX_CAPTURE = 1024**2
MAX_EVENTS = (8*1024**2-256*1024)//4
PS = ['/bin/ps', '-axo', 'pid=,ppid=,pgid=,uid=,ruid=,stat=,lstart=']
OUTPUT = b'harmless fixture stdout\n'
ERROR_OUTPUT = b'harmless fixture stderr\n'
DEADLINES = {'natural-output': 3, 'deadline-term': 2, 'resistant-pipe': 3}


def publish_ready(path, payload):
    path = Path(path)
    temporary = path.with_name(path.name+'.writing')
    if path.exists() or path.is_symlink():
        raise ValueError('fixture readiness already exists')
    with temporary.open('x') as stream:
        json.dump(payload, stream)
        stream.flush()
    if path.exists() or path.is_symlink():
        raise ValueError('fixture readiness destination appeared')
    temporary.rename(path)


def fixture(kind, ready):
    if kind not in KINDS:
        raise ValueError('unknown fixture')
    signal.signal(signal.SIGTERM, signal.SIG_DFL)
    signal.signal(signal.SIGALRM, signal.SIG_DFL)
    signal.alarm(FIXTURE_SECONDS)
    children = []
    if kind == 'resistant-pipe':
        read, write = os.pipe()
        child = os.fork()
        if child == 0:
            os.close(read)
            signal.signal(signal.SIGTERM, signal.SIG_IGN)
            signal.alarm(FIXTURE_SECONDS)
            os.write(write, b'R'); os.close(write)
            while True:
                signal.pause()
        os.close(write)
        if os.read(read, 1) != b'R':
            raise ValueError('fixture child readiness failed')
        os.close(read)
        children.append(child)
    publish_ready(ready, {'kind': kind, 'leader': os.getpid(), 'children': children})
    if kind == 'waitid-capability':
        return 7
    os.write(1, OUTPUT); os.write(2, ERROR_OUTPUT)
    if kind == 'natural-output':
        return 7
    if kind == 'resistant-pipe':
        return 0
    while True:
        signal.pause()


def failure(error):
    return {'type': type(error).__name__, 'message': str(error)[:4096],
            'errno': getattr(error, 'errno', None), 'notes': [str(note)[:1024] for note in list(getattr(error, '__notes__', []))[:4]]}


class Events:
    def __init__(self):
        self.rows = []
        self.bytes = 0

    def add(self, kind, **fields):
        row = {'kind': kind, 'time': time.monotonic(), **fields}
        size = len(json.dumps(row).encode())
        if self.bytes+size > MAX_EVENTS:
            raise ValueError('bounded process observation overflow')
        self.rows.append(row)
        self.bytes += size
        return row


def parse(raw):
    """Independent retained ps parser; do not trust producer-parsed rows."""
    result = {}
    pattern = (r'\s*(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\S+)\s+'
               r'([A-Za-z]{3} [A-Za-z]{3}\s+\d{1,2} \d\d:\d\d:\d\d \d{4})\s*')
    if len(raw.encode()) > MAX_CAPTURE:
        raise ValueError('ps output bound')
    for line in raw.splitlines():
        match = re.fullmatch(pattern, line)
        if not match:
            raise ValueError('malformed Darwin inventory')
        pid, parent, group, uid, ruid = map(int, match.groups()[:5])
        if pid in result:
            raise ValueError('duplicate Darwin PID')
        result[pid] = dict(pid=pid, parent=parent, group=group, uid=uid, ruid=ruid,
                           state=match[6], start=' '.join(match[7].split()))
    if not result:
        raise ValueError('empty Darwin inventory')
    return result


def capture():
    proc = subprocess.Popen(PS, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                            env=os.environ | {'LC_ALL': 'C'})
    buffers = {'stdout': bytearray(), 'stderr': bytearray()}
    end = time.monotonic()+.5
    try:
        with selectors.DefaultSelector() as select:
            for name in buffers:
                select.register(getattr(proc, name), selectors.EVENT_READ, name)
            while select.get_map():
                if time.monotonic() >= end:
                    raise ValueError('ps deadline')
                for key, _ in select.select(.01):
                    capacity = MAX_CAPTURE-sum(map(len, buffers.values()))
                    data = os.read(key.fd, min(65536, capacity+1))
                    if not data:
                        select.unregister(key.fileobj)
                    else:
                        buffers[key.data].extend(data)
                        if len(data) > capacity:
                            raise ValueError('ps output bound')
        code = proc.wait(timeout=max(.001, end-time.monotonic()))
        if code or buffers['stderr']:
            raise ValueError('ps command failed')
        return buffers['stdout'].decode('utf-8', errors='strict')
    finally:
        if proc.poll() is None:
            proc.kill()
        proc.wait(timeout=.5)
        proc.stdout.close(); proc.stderr.close()


@contextmanager
def observe_production(events):
    """Forward exactly; never add a signal, poll, wait or cleanup decision."""
    original_group = production._DarwinGroup
    original_parse = production._darwin_process_rows
    original_killpg = os.killpg
    groups = []
    waits = []

    def parser(raw):
        events.add('snapshot', raw=raw)
        return original_parse(raw)

    def killpg(pid, number):
        events.add('signal-start', pid=pid, signal=int(number))
        try:
            value = original_killpg(pid, number)
        except BaseException as error:
            events.add('signal-end', pid=pid, signal=int(number), error=failure(error))
            raise
        events.add('signal-end', pid=pid, signal=int(number), error=None)
        return value

    class ObservedGroup(original_group):
        def __init__(self, proc):
            super().__init__(proc)
            groups.append(self)
            events.add('spawn', pid=proc.pid, returncode=proc.returncode)
            original_wait = proc.wait
            waits.append((proc, original_wait))
            def wait(*args, **kwargs):
                events.add('wait-start', pid=proc.pid, returncode=proc.returncode,
                           args=list(args), kwargs=kwargs)
                try:
                    value = original_wait(*args, **kwargs)
                except BaseException as error:
                    events.add('wait-end', pid=proc.pid, error=failure(error))
                    raise
                events.add('wait-end', pid=proc.pid, error=None, returncode=value)
                return value
            proc.wait = wait

        def members(self):
            value = super().members()
            events.add('members', pid=self.proc.pid, reaped=self.reaped, rows=value)
            return value

        def send(self, number, members):
            events.add('send', pid=self.proc.pid, signal=int(number), rows=members,
                       reaped=self.reaped, returncode=self.proc.returncode)
            return super().send(number, members)

        def cleanup(self):
            events.add('cleanup-start', complete=self.complete, reaped=self.reaped,
                       deadline=self.deadline)
            try:
                value = super().cleanup()
            except BaseException as error:
                events.add('cleanup-end', complete=self.complete, reaped=self.reaped,
                           deadline=self.deadline, error=failure(error),
                           signal_errors=[failure(e) for e in self.signal_errors])
                raise
            events.add('cleanup-end', complete=self.complete, reaped=self.reaped,
                       deadline=self.deadline, error=None,
                       signal_errors=[failure(e) for e in self.signal_errors])
            return value

    production._DarwinGroup = ObservedGroup
    production._darwin_process_rows = parser
    os.killpg = killpg
    try:
        yield groups
    finally:
        for proc, original_wait in waits:
            proc.wait = original_wait
        os.killpg = original_killpg
        production._darwin_process_rows = original_parse
        production._DarwinGroup = original_group


def ready_record(path):
    if path.is_symlink() or path.stat().st_size > 1024:
        raise ValueError('fixture readiness output bound/type')
    return json.loads(path.read_text())


def snapshot(events, label):
    started = time.monotonic()
    raw = capture()
    events.add('independent-snapshot', label=label, started=started, raw=raw)
    return parse(raw)


def capability(argv, env, events, checkpoint):
    production._darwin_require_waitid()
    proc = subprocess.Popen(argv, start_new_session=True, stdin=subprocess.DEVNULL,
                            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, env=env)
    events.add('spawn', pid=proc.pid, returncode=proc.returncode)
    reaped = False
    try:
        end = time.monotonic()+2
        flags = os.WEXITED | os.WNOHANG | os.WNOWAIT
        while True:
            checkpoint()
            if time.monotonic() >= end:
                raise ValueError('waitid capability exit deadline')
            value = os.waitid(os.P_PID, proc.pid, flags)
            if value is not None:
                break
            time.sleep(.01)
        for index in range(2):
            if index:
                value = os.waitid(os.P_PID, proc.pid, flags)
            events.add('waitid', flags=flags, idtype=os.P_PID, pid=proc.pid,
                       returncode=proc.returncode,
                       value=None if value is None else dict(zip(
                           ('si_pid', 'si_uid', 'si_signo', 'si_status', 'si_code'), value)))
            snapshot(events, 'held-zombie-'+str(index))
        events.add('wait-start', pid=proc.pid, returncode=proc.returncode, args=[], kwargs={'timeout': 1})
        code = proc.wait(timeout=1)
        reaped = True
        events.add('wait-end', pid=proc.pid, error=None, returncode=code)
        snapshot(events, 'after-reap')
        return proc.pid
    finally:
        if not reaped:
            # Containment only; no signal, no pass conversion. Independent ALRM
            # bounds fixture lifetime even when the capability call fails.
            try:
                code = proc.wait(timeout=FIXTURE_SECONDS+1)
                events.add('failure-containment-wait', pid=proc.pid, returncode=code)
            except BaseException as error:
                events.add('failure-containment-wait', pid=proc.pid, error=failure(error))


def observe_case(kind, directory, python, checkpoint, env):
    ready_path = directory/(kind+'-ready.json')
    argv = [python, '-I', '-S', '-B', str(Path(__file__).resolve()), kind, str(ready_path)]
    events = Events()
    case = {'kind': kind, 'argv': argv, 'events': events.rows, 'outcome': None,
            'failure': None, 'deadline_seconds': None if kind == KINDS[0] else DEADLINES[kind]}
    try:
        checkpoint()
        if kind == KINDS[0]:
            case['leader'] = capability(argv, env, events, checkpoint)
        else:
            with observe_production(events) as groups:
                outcome = production.execute(argv, cwd=directory, env=env,
                    deadline_seconds=DEADLINES[kind], limit_bytes=4096)
            case['outcome'] = asdict(outcome) | {'stdout': outcome.stdout.hex(), 'stderr': outcome.stderr.hex()}
            if len(groups) != 1:
                raise ValueError('actual production group count differs')
            case['leader'] = groups[0].proc.pid
            snapshot(events, 'after-execute')
        case['ready'] = ready_record(ready_path)
    except BaseException as error:
        case['failure'] = failure(error)
        spawned = [e['pid'] for e in events.rows if e['kind'] == 'spawn']
        if len(spawned) == 1:
            case['leader'] = spawned[0]
        if ready_path.exists():
            try:
                case['ready'] = ready_record(ready_path)
            except (OSError, ValueError):
                pass
    return case


if __name__ == '__main__':
    # Isolated -I fixtures need no project imports: defer that import in the
    # module bootstrap above when this file is run as the child command.
    if len(sys.argv) != 3:
        raise SystemExit('fixed fixture requires kind and ready path')
    raise SystemExit(fixture(sys.argv[1], sys.argv[2]))
