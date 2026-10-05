"""Harmless fixed Darwin fixture and signal-sequence observation, not a supervisor."""
import json
import os
from pathlib import Path
import re
import selectors
import signal
import subprocess
import sys
import time

PS = ['/bin/ps', '-ww', '-axo', 'pid=,ppid=,pgid=,uid=,ruid=,rss=,stat=,lstart=,comm=']
KINDS = ('ordinary', 'multi-child')
FIXTURE_SECONDS = 8
MAX_CAPTURE = 1024**2
IDENTITY = ('pid', 'pgid', 'uid', 'ruid', 'start')


def publish_ready(path, payload):
    path = Path(path)
    temporary = path.with_name(path.name+'.writing')
    # The directory is private and owned; this fixed fixture is its only
    # readiness writer. A previous destination is an error, never a retry.
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
    if kind == 'multi-child':
        for ignore in (False, True):
            read, write = os.pipe()
            child = os.fork()
            if child == 0:
                os.close(read)
                signal.signal(signal.SIGTERM, signal.SIG_IGN if ignore else signal.SIG_DFL)
                signal.alarm(FIXTURE_SECONDS)
                os.write(write, b'R'); os.close(write)
                while True:
                    signal.pause()
            os.close(write)
            if os.read(read, 1) != b'R':
                raise ValueError('fixture child readiness failed')
            os.close(read)
            children.append(child)
    # Children are fixed and never respawn; expose only a complete ready record.
    publish_ready(ready, {'kind': kind, 'leader': os.getpid(), 'children': children})
    while True:
        signal.pause()


def capture():
    """Bound the one fixed OS inventory by time and bytes while reading it."""
    proc = subprocess.Popen(PS, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                            env=os.environ | {'LC_ALL': 'C'})
    buffers = {'stdout': bytearray(), 'stderr': bytearray()}
    deadline = time.monotonic()+.5
    try:
        with selectors.DefaultSelector() as select:
            for name in buffers:
                select.register(getattr(proc, name), selectors.EVENT_READ, name)
            while select.get_map():
                if time.monotonic() >= deadline:
                    raise ValueError('ps deadline')
                for key, _ in select.select(.01):
                    data = os.read(key.fd, 65536)
                    if not data:
                        select.unregister(key.fileobj)
                    else:
                        buffers[key.data].extend(data)
                        if sum(map(len, buffers.values())) > MAX_CAPTURE:
                            raise ValueError('ps output bound')
        code = proc.wait(timeout=max(.001, deadline-time.monotonic()))
        if code or buffers['stderr']:
            raise ValueError('ps command failed')
        return buffers['stdout'].decode('utf-8', errors='strict')
    finally:
        if proc.poll() is None:
            proc.kill()
        proc.wait()
        proc.stdout.close(); proc.stderr.close()


def parse(raw):
    result = {}
    pattern = (r'\s*(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\S+)\s+'
               r'([A-Za-z]{3} [A-Za-z]{3}\s+\d{1,2} \d\d:\d\d:\d\d \d{4})\s+(.*)')
    for line in raw.splitlines():
        match = re.fullmatch(pattern, line)
        if not match:
            raise ValueError('malformed Darwin inventory')
        pid, parent, group, uid, ruid, rss = map(int, match.groups()[:6])
        if pid in result:
            raise ValueError('duplicate Darwin PID')
        result[pid] = dict(pid=pid, ppid=parent, pgid=group, uid=uid, ruid=ruid,
                           rss_bytes=rss*1024, state=match[7], start=match[8], command=match[9])
    if not result:
        raise ValueError('empty Darwin inventory')
    return result


def owned(rows, ready, driver, uid, ruid):
    leader, children = ready['leader'], ready['children']
    expected_count = 0 if ready['kind'] == 'ordinary' else 2
    if (type(leader) is not int or leader <= 0 or type(driver) is not int or driver <= 0
            or ready['kind'] not in KINDS or len(children) != expected_count
            or any(type(pid) is not int or pid <= 0 for pid in children)
            or len(set([leader, *children])) != len(children)+1):
        raise ValueError('invalid fixture identities')
    expected = {leader, *children}
    members = {pid for pid, row in rows.items() if row['pgid'] == leader}
    if members != expected:
        raise ValueError('initial group differs from fixed ready fixture')
    for pid in expected:
        row = rows[pid]
        if (row['ppid'] != (driver if pid == leader else leader)
                or row['uid'] != uid or row['ruid'] != ruid or row['state'].startswith('Z')):
            raise ValueError('initial ownership differs')
    return {str(pid): {key: rows[pid][key] for key in IDENTITY} for pid in sorted(expected)}


def eligible(rows, identities, leader):
    """Only a still-unreaped owned leader can authorize a group signal."""
    members = [row for row in rows.values() if row['pgid'] == leader]
    anchor = rows.get(leader)
    if anchor is not None and (str(leader) not in identities or
            any(anchor[key] != identities[str(leader)][key] for key in IDENTITY)):
        raise ValueError('owned leader identity changed')
    if not members:
        return False
    if leader not in rows or str(leader) not in identities:
        raise ValueError('owned group anchor unavailable')
    for row in members:
        expected = identities.get(str(row['pid']))
        if expected is None or any(row[key] != expected[key] for key in IDENTITY):
            raise ValueError('unknown or changed group identity')
    return True


def observe_case(kind, directory, python, snapshot, checkpoint, env):
    """Model TERM/.25s/KILL; record errors, never turn EPERM into success."""
    ready_path = directory/(kind+'-ready.json')
    argv = [python, '-I', '-S', '-B', str(Path(__file__).resolve()), kind, str(ready_path)]
    case = {'kind': kind, 'argv': argv, 'events': [], 'cleanup_verified': False}
    proc = subprocess.Popen(argv, start_new_session=True, stdin=subprocess.DEVNULL,
                            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, env=env)
    case['leader'] = proc.pid
    identities = None
    reaped = False

    def snap(label):
        row = snapshot(kind+'-'+label)
        case['events'].append({'kind': 'snapshot', 'label': label, 'record': {key: value for key, value in row.items() if key != 'raw'}})
        return parse(row['raw'])

    def send(number, rows, label):
        checkpoint()
        if reaped:
            raise ValueError('cannot signal after leader reaping')
        if not eligible(rows, identities, proc.pid):
            case['events'].append({'kind': 'signal', 'signal': number, 'label': label,
                                   'attempted': False, 'reason': 'group absent'})
            return
        event = {'kind': 'signal', 'signal': number, 'label': label, 'attempted': True,
                 'started': time.monotonic(), 'errno': None}
        case['events'].append(event)
        try:
            os.killpg(proc.pid, number)
        except OSError as error:
            event['errno'] = error.errno
        event['finished'] = time.monotonic()

    try:
        until = time.monotonic()+2
        while not ready_path.exists():
            checkpoint()
            if time.monotonic() >= until:
                raise ValueError('fixture readiness deadline')
            time.sleep(.01)
        if ready_path.stat().st_size > 1024:
            raise ValueError('fixture readiness output bound')
        case['ready'] = json.loads(ready_path.read_text())
        if case['ready']['leader'] != proc.pid or case['ready']['kind'] != kind:
            raise ValueError('fixture readiness source differs')
        rows = snap('before-term')
        identities = owned(rows, case['ready'], os.getpid(), os.geteuid(), os.getuid())
        case['identities'] = identities
        case['session'] = os.getsid(proc.pid)
        if case['session'] != proc.pid:
            raise ValueError('fixture session differs')
        send(signal.SIGTERM, rows, 'term')
        # Match the existing observer-stop branch's interval; the unreaped
        # direct child reserves the leader PID through both signal attempts.
        time.sleep(.25)
        rows = snap('after-term')
        send(signal.SIGKILL, rows, 'kill')
        snap('after-kill')
    except Exception as error:
        case['failure'] = type(error).__name__+': '+str(error)
    finally:
        # Reattempt only with fresh, still-owned identities. No group signal
        # is permitted after wait() releases the leader PID reservation.
        if identities is not None:
            try:
                rows = snap('cleanup-before-reap')
                if eligible(rows, identities, proc.pid):
                    send(signal.SIGKILL, rows, 'cleanup-kill')
            except Exception as error:
                case['cleanup_failure'] = type(error).__name__+': '+str(error)
        # Every fixed fixture has an independent 8s SIGALRM, including the
        # TERM-ignoring child. Waiting does not suppress any signal error.
        try:
            code = proc.wait(timeout=FIXTURE_SECONDS+1)
            reaped = True
            case['events'].append({'kind': 'reap', 'returncode': code, 'time': time.monotonic()})
        except subprocess.TimeoutExpired:
            case['cleanup_failure'] = 'fixed fixture failed its independent lifetime bound'
        if reaped:
            try:
                until = time.monotonic()+2
                attempt = 0
                while True:
                    rows = snap('after-reap-'+str(attempt))
                    members = [row for row in rows.values() if row['pgid'] == proc.pid]
                    if not members:
                        case['cleanup_verified'] = True
                        break
                    if any(str(row['pid']) not in (identities or {}) or
                           any(row[key] != identities[str(row['pid'])][key] for key in IDENTITY)
                           for row in members):
                        raise ValueError('post-reap group identity unknown or reused; no signal authorized')
                    if time.monotonic() >= until:
                        raise ValueError('owned group remains after reap')
                    time.sleep(.1); attempt += 1
            except Exception as error:
                case['cleanup_failure'] = type(error).__name__+': '+str(error)
    return case


if __name__ == '__main__':
    if len(sys.argv) != 3:
        raise SystemExit('fixed fixture requires kind and ready path')
    fixture(sys.argv[1], sys.argv[2])
