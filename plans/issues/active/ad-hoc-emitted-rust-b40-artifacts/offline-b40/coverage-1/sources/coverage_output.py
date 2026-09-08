"""B32 sole inferior-file route, strict completion parser and final decision.

No debugger transcript or SBProcess stream is an input to this consumer.
The immutable binary cannot emit a run nonce: custody instead binds exclusive
prelaunch inodes, installed fd actions, one authenticated process and exit/release.
"""
import hashlib
import os
import re
import stat
from pathlib import Path
from coverage_symbols import Protocol, require

STREAMS = ((1, 'stdout'), (2, 'stderr'))
MAX_STREAM = 4 * 1024**2
RUN_ID = '12K-B40-urwpgv-target-0'
PHASE = re.compile(rb'B20_PHASE ([a-z.]+) ([0-9]+) ([0-9]+) ([0-9]+) ([0-9]+)')


def identity(info):
    require(stat.S_ISREG(info.st_mode) and info.st_nlink == 1,
            'output is not a sole-link regular file')
    require(info.st_uid == os.getuid() and stat.S_IMODE(info.st_mode) == 0o600,
            'output owner/mode changed')
    return {'device': info.st_dev, 'inode': info.st_ino, 'uid': info.st_uid}


def create_route(directory, run_id, binary):
    """Called once by outer custodian; existing paths are a terminal error."""
    directory = Path(directory).resolve()
    streams = {}
    for fd, name in STREAMS:
        path = directory / ('inferior.' + name)
        handle = os.open(str(path), os.O_CREAT | os.O_EXCL | os.O_WRONLY | os.O_NOFOLLOW, 0o600)
        try:
            info = os.fstat(handle)
            require(info.st_size == 0, 'nonempty new output')
            streams[name] = dict(identity(info), path=str(path), fd=fd)
        finally:
            os.close(handle)
    return {'run_id': run_id, 'binary': str(binary), 'streams': streams,
            'route': 'exclusive-files', 'producer': 'sole-inferior', 'consumer': 'outer-after-release'}


def open_stream(saved):
    handle = os.open(saved['path'], os.O_RDONLY | os.O_NOFOLLOW)
    try:
        info = os.fstat(handle)
        require(identity(info) == {k: saved[k] for k in ('device', 'inode', 'uid')},
                'stale or replaced output inode')
        return handle, info
    except BaseException:
        os.close(handle)
        raise


def configure_launch(launch, route, directory, run_id, binary):
    """Observer installs exactly two explicit file actions before target launch."""
    require(route['run_id'] == run_id and route['binary'] == str(binary), 'wrong launch route/run')
    require(route['route'] == 'exclusive-files' and set(route['streams']) == {'stdout', 'stderr'},
            'wrong stream route')
    actions = []
    for fd, name in STREAMS:
        saved = route['streams'][name]
        require(saved['fd'] == fd and saved['path'] == str(Path(directory) / ('inferior.' + name)),
                'wrong fd/path mapping')
        handle, info = open_stream(saved)
        os.close(handle)
        require(info.st_size == 0, 'stale prelaunch output')
        require(launch.AddOpenFileAction(fd, saved['path'], False, True) is True,
                'installed output action unsupported')
        actions.append({'fd': fd, 'path': saved['path'], 'read': False, 'write': True})
    return actions


def parse_completion(stdout, stderr):
    """Parse complete stderr records; numeric payloads are syntax only, not metrics."""
    require(stdout == b'', 'unexpected inferior stdout')
    require(stderr and stderr.endswith(b'\n'), 'empty or truncated phase stream')
    lines = stderr.splitlines()
    require(lines[0] == b'format check passed' and lines.count(b'format check passed') == 1,
            'missing exact format-check success')
    records = []
    for line in lines[1:]:
        match = PHASE.fullmatch(line)
        require(match is not None, 'malformed or non-inferior phase record')
        # Keep payload bytes verbatim, without counter calculations or inference.
        records.append({'name': match.group(1).decode('ascii'),
                        'payload': [part.decode('ascii') for part in match.groups()[1:]],
                        'raw': line.decode('ascii')})
    names = [r['name'] for r in records]
    require(names and names[-1] == 'cli.done' and names.count('cli.done') == 1, 'missing/duplicate cli completion')
    file_records = [r for r in records if r['name'].startswith('file.')]
    require([r['name'] for r in file_records] ==
            ['file.before.read', 'file.after.read', 'file.after.check'] * 2,
            'wrong file phase sequence or count')
    checks = [r for r in file_records if r['name'] == 'file.after.check']
    require(checks[0]['raw'] != checks[1]['raw'], 'duplicate file completion record')
    return checks


def validate_provenance(route, observer, handshake, ack, custody, release):
    from coverage_custody import validate_completion
    validate_completion(ack, custody, observer['pid'], route['run_id'], route['binary'])
    require(observer['state'] == 'ENTRIES_COMPLETE' and observer['stage'] == 'completion'
            and 'error' not in observer, 'observer not complete')
    require(observer['run_id'] == route['run_id'] == handshake['run_id'] == RUN_ID,
            'wrong run identity')
    pid = observer['pid']
    require(type(pid) is int and pid > 0 and observer['target_launches'] == 1,
            'invalid sole inferior identity')
    require(handshake['pid'] == handshake['pgid'] == pid and handshake['flags'] == 134,
            'wrong inferior handshake')
    process = ack['process']
    require(process['pid'] == process['pgid'] == pid,
            'wrong authenticated process')
    require(custody['target_ids'] == [pid] and process in custody['processes'],
            'missing sole-process custody')
    expected = [{'fd': fd, 'path': route['streams'][name]['path'], 'read': False, 'write': True}
                for fd, name in STREAMS]
    require(observer['output_actions'] == expected, 'wrong installed output actions')
    require(observer['exit'] == 0 and observer['exited'] is True, 'inferior not exited successfully')
    require(release['remaining'] == [] and release['groups_absent'] is True
            and release['watcher_alive'] is False and release['monitor_alive'] is False,
            'process release incomplete')
    from coverage_custody import replay_document
    final = replay_document(custody, route['binary'], route['run_id'])
    require(final.last is not None and all(not entry['present']
                for owned_pid, entry in final.native_last.items() if owned_pid != final.root),
            'fresh native process absence unproved')


def finalize(route, observer, handshake, ack, custody, release):
    """Only outer custodian calls this, after debugger return and zero-process release."""
    validate_provenance(route, observer, handshake, ack, custody, release)
    raw = {}
    receipt = {'run_id': route['run_id'], 'pid': observer['pid'], 'streams': {}}
    for fd, name in STREAMS:
        saved = route['streams'][name]
        handle, before = open_stream(saved)
        with os.fdopen(handle, 'rb') as stream:
            require(before.st_size <= MAX_STREAM, 'stream bound exceeded')
            data = stream.read(MAX_STREAM + 1)
            after = os.fstat(stream.fileno())
        require(len(data) == before.st_size == after.st_size
                and before.st_mtime_ns == after.st_mtime_ns, 'output changed during final read')
        raw[name] = data
        receipt['streams'][name] = dict(saved, size=len(data), sha256=hashlib.sha256(data).hexdigest())
    checks = parse_completion(raw['stdout'], raw['stderr'])
    protocol = Protocol()
    protocol.phase = observer['final_phase']
    protocol.handover_count = observer['handover_count']
    protocol.hits = list(observer['hits'])
    protocol.finish(observer['exit'], len(checks))
    receipt.update(state='PASS', final_phase=protocol.phase, file_check_phases=len(checks),
                   parsed_file_checks=checks, finalized_after_exit_and_release=True)
    return receipt
