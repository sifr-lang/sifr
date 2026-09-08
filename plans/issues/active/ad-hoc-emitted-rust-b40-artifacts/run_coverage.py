"""Single registered B35 combined coverage attempt; no sampling or retries."""
import hashlib
import json
import os
import shutil
import signal
import subprocess
import sys
import threading
import time
import traceback
from pathlib import Path
from coverage_output import RUN_ID, create_route, finalize
from coverage_custody import Custody, capture
from coverage_native import NativeCapture, Darwin
from coverage_cleanup import signal_targets, drain, final_absence

E = Path('/private/tmp/sifr-b40.urwpgv/evidence')
ROOT = E.parent / 'sifr'
OUT = E / 'coverage'
BINARY = E / 'sifr-experiment09'
EXPECTED = 'a4386baecd7576b256dc6d68eb2e240952f591fa0a468df55f60080189c4b392'
sys.path.insert(0, str(ROOT / 'verification/areas/performance'))
from benchmark_process import run_owned_process, group_exists
from host_control import HostActivityMonitor, capture_host_snapshot, evaluate_snapshot


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for data in iter(lambda: stream.read(1048576), b''):
            h.update(data)
    return h.hexdigest()


def save(name, value):
    path = OUT / name
    temp = path.with_suffix('.tmp')
    temp.write_text(json.dumps(value, indent=2, sort_keys=True) + '\n')
    temp.replace(path)


def read(name):
    path = OUT / name
    return json.loads(path.read_text()) if path.exists() else None


def main():
    if OUT.exists():
        raise RuntimeError('Attempt already exists; no retry')
    OUT.mkdir()
    started = time.monotonic()
    deadline = started + 300
    execution_end = deadline - 30
    failures = []
    lock = threading.Lock()
    done = threading.Event()
    custody = Custody(os.getpid(), os.getpgrp(), BINARY, RUN_ID)
    native_capture = NativeCapture(os.getpid(), Darwin(), capture)
    owned_groups = custody.groups
    target_ids = custody.targets
    first_intent = None
    debugger_started = None
    monitor = None
    launched_debugger = False
    output = {'state': 'INCONCLUSIVE', 'coverage_only': True}

    def reject(reason):
        with lock:
            if reason not in failures:
                failures.append(reason)
            save('stop.json', {'reasons': list(failures),
                               'outer_elapsed_seconds': time.monotonic() - started})

    def inspect():
        nonlocal first_intent, debugger_started
        current = custody.observe(native_capture())
        # Persist the complete first/conflicting evidence before generic stop
        # notification. Later observations never replace a first conflict.
        save('custody.json', custody.document())
        for finding in custody.rejections:
            reject(finding['reason'])
        now = time.monotonic()
        for row in current:
            if row['pid'] in custody.first:
                if Path(row['command']).name == 'lldb':
                    if debugger_started is None:
                        debugger_started = now
        intent = read('launch-intent.json')
        if intent is not None and first_intent is None:
            first_intent = now
        handshake = read('inferior.json')
        if handshake and not (OUT / 'custody-ack.json').exists():
            if handshake['pid'] in target_ids and not failures:
                ack = custody.acknowledge(handshake)
                ack['outer_elapsed_seconds'] = now - started
                save('custody-ack.json', ack)
        if debugger_started is not None and first_intent is None and now - debugger_started >= 30:
            reject('debugger setup exceeded30s')
        if first_intent is not None and now - first_intent >= 60:
            reject('outer target elapsed limit60s')
        if now >= execution_end:
            reject('cleanup reserve reached')
        if monitor is not None and monitor.rejection_reasons():
            reject('host rejection: ' + repr(monitor.rejection_reasons()))
        if shutil.disk_usage(E).free < 4 * 1024**3:
            reject('free disk below4GiB')
        if sum(p.stat().st_size for p in E.rglob('*') if p.is_file()) >= 1024**3:
            reject('owned evidence exceeds1GiB')
        return current

    def kill_group(pgid):
        allowed = custody.cleanup_allowed(pgid, native_capture())
        save('custody.json', custody.document())
        for finding in custody.rejections:
            reject(finding['reason'])
        if not allowed:
            return
        try:
            os.killpg(pgid, signal.SIGKILL)
        except ProcessLookupError:
            pass

    def watch():
        while not done.wait(0.1):
            try:
                current = inspect()
                if failures:
                    # Kill inferior first, allowing the debugger to reap it.
                    signal_targets(custody, kill_group)
            except BaseException:
                reject('custody watcher failed: ' + traceback.format_exc())
                return

    watcher = threading.Thread(target=watch, name='b35-custody', daemon=True)
    save('window.json', {'started_monotonic': started, 'deadline_monotonic': deadline,
         'execution_end_monotonic': execution_end, 'clock_owner': 'outer Python only',
         'cleanup_seconds': 30, 'launcher_pid': os.getpid(), 'max_targets': 1})
    try:
        assert 'CARGO_TARGET_DIR' not in os.environ
        assert Path(os.environ['TMPDIR']).resolve() == E.parent / 'tmp'
        assert not (ROOT / 'target').exists()
        manifest = json.loads((E / 'prelaunch.json').read_text())
        assert manifest['binary_sha256'] == EXPECTED and digest(BINARY) == EXPECTED
        for filename, expected in manifest['file_hashes'].items():
            assert digest(Path(filename)) == expected, ('changed input', filename)
        for entry in manifest['ancestor_config']:
            path = Path(entry['path'])
            assert path.exists() == entry['exists'], ('changed ancestor config', str(path))
            if entry['exists']:
                assert digest(path) == entry['sha256']
        save('identity.json', manifest)
        assert manifest['output_run_id'] == RUN_ID
        route = create_route(OUT, RUN_ID, BINARY)
        save('output-route.json', route)
        assert shutil.disk_usage(E).free >= 4 * 1024**3
        assert sum(p.stat().st_size for p in E.rglob('*') if p.is_file()) < 1024**3
        # Coverage needs quiet-host custody, not a performance counter or CV
        # calibration. Preserve the first rejected snapshot and stop this attempt.
        admission = {'status': 'checking', 'calibration': False, 'snapshots': []}
        for index in range(3):
            assert time.monotonic() - started < 180, 'admission deadline'
            snapshot = capture_host_snapshot(include_calibration=False, control_mode='work')
            reasons = evaluate_snapshot(snapshot, enforce_load=True, control_mode='work',
                                        require_work_counter=False)
            admission['snapshots'].append({'snapshot': snapshot, 'rejections': reasons})
            save('admission.json', admission)
            assert not reasons, ('host admission rejected', reasons)
            if index < 2:
                time.sleep(1)
        admission['status'] = 'controlled-for-coverage-only'
        save('admission.json', admission)
        assert time.monotonic() + 90 <= execution_end, 'no full setup/target window'
        command = ['/usr/bin/lldb', '--no-lldbinit', '-b', '-s', str(E / 'coverage.lldb')]
        assert command == manifest['debugger_command']
        save('command.json', {'argv': command, 'cwd': str(ROOT)})
        with HostActivityMonitor(control_mode='work') as monitor:
            assert not monitor.rejection_reasons(), monitor.rejection_reasons()
            watcher.start()
            launched_debugger = True
            completed, timed_out = run_owned_process(
                command, ROOT, min(90.0, execution_end - time.monotonic()))
            save('debugger-result.json', {'exit': completed.returncode,
                 'timed_out': timed_out, 'stdout': completed.stdout,
                 'stderr': completed.stderr})
            if timed_out or completed.returncode:
                reject('debugger failed or timed out')
            if monitor.rejection_reasons():
                reject('host rejected completion: ' + repr(monitor.rejection_reasons()))
        report = read('observer-result.json')
        output['observer'] = report
        if not report or report.get('state') != 'ENTRIES_COMPLETE':
            reject('observer coverage criteria not satisfied')
    except BaseException:
        reject(traceback.format_exc())
    finally:
        done.set()
        if watcher.is_alive():
            watcher.join(timeout=3)
        try:
            current = inspect()
            signal_targets(custody, kill_group)
            # Keep LLDB/debugserver alive briefly to reap the inferior, then
            # clean any authenticated group left after the B23 helper returned.
            until = min(deadline - 5, time.monotonic() + 15)
            drain(custody, inspect, kill_group, group_exists, until,
                  time.monotonic, time.sleep)
            remaining, absent = final_absence(custody, native_capture, group_exists)
            save('custody.json', custody.document())
            for finding in custody.rejections:
                reject(finding['reason'])
        except BaseException:
            reject('cleanup verification failed: ' + traceback.format_exc())
            remaining, absent = [], False
        if watcher.is_alive() or not absent:
            reject('process release not proved')
        if time.monotonic() > deadline:
            reject('whole-window deadline exceeded')
        save('process-release.json', {'remaining': remaining, 'groups_absent': absent,
             'groups': sorted(owned_groups), 'watcher_alive': watcher.is_alive(),
             'monitor_alive': bool(monitor and monitor._thread and monitor._thread.is_alive()),
             'elapsed_seconds': time.monotonic() - started})
        if monitor is not None:
            save('host.json', {'snapshots': monitor.snapshots,
                              'rejections': monitor.rejection_reasons()})
        if not failures and absent:
            try:
                completion = finalize(route, report, read('inferior.json'), read('custody-ack.json'),
                                      read('custody.json'), read('process-release.json'))
                save('output-completion.json', completion)
                output['completion'] = completion
            except BaseException:
                reject('inferior output completion rejected: ' + traceback.format_exc())
        output.update(state='PASS' if not failures and absent else 'INCONCLUSIVE',
             failures=failures, debugger_sessions=int(launched_debugger),
             target_launches=max(len(target_ids), int(bool(read('inferior.json')))),
             zero_processes=absent, elapsed_seconds=time.monotonic() - started)
        save('outcome.json', output)
        native_capture.close()
        save('inventory.json', {'files': [{'path': str(p), 'sha256': digest(p)}
             for p in sorted(OUT.iterdir()) if p.is_file() and p.name != 'inventory.json']})
        print(json.dumps(output, indent=2), flush=True)
    return 0 if output['state'] == 'PASS' else 1


if __name__ == '__main__':
    raise SystemExit(main())
