"""B24 single allocation: external Python owns host, clocks and target custody."""
import hashlib
import json
import os
import shutil
import signal
import sys
import threading
import time
import traceback
from pathlib import Path
from boundaries_core import E, ROOT, OUT, BINARY, run_id, save, read, elapsed, full_target_fits
from coverage_custody import Custody, capture
from coverage_native import NativeCapture, Darwin
from coverage_cleanup import signal_targets, drain, final_absence
from coverage_output import create_route
from boundaries_output import target_release, complete
from coverage_symbols import require

sys.path.append(str(ROOT / 'verification/areas/performance'))
from benchmark_process import run_owned_process, group_exists
from host_control import HostActivityMonitor, capture_host_snapshot, evaluate_snapshot
import host_control
import benchmark_process
from boundaries_lifecycle import Inventory, Lifecycle, absence
from boundaries_schedule import Schedule

def digest(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()

def main():
    require(not OUT.exists(), 'allocation already used; no retry')
    OUT.mkdir()
    start = time.monotonic()
    # Supervisor retains the full540s allocation and root-exit evidence. This
    # child returns by530, preserving the final10s for root-inclusive absence.
    deadline, end = start + 530, start + 510
    inventory = Inventory(os.getpid(), BINARY)
    lifecycle = Lifecycle(inventory, deadline,
        lambda value: save(OUT / 'producer-lifecycle.json', value))
    host_control.LIFECYCLE = lifecycle
    benchmark_process.LIFECYCLE = lifecycle
    failures, receipts = [], []
    done = threading.Event()
    native = None
    schedule = None
    monitor = None
    watcher = None
    launched = False
    custody = Custody(os.getpid(), os.getpgrp(), BINARY, run_id(0))
    index, intent_start = 0, None
    debugger_start = None
    release_begin = None
    all_groups, all_pids = set(), {os.getpid()}
    result = {'state': 'INCONCLUSIVE', 'diagnostic_only': True, 'acceptance': False}
    save(OUT / 'window.json', {'outer_monotonic_start': start, 'outer_deadline': deadline,
        'observation_end': end, 'clock_owner': 'outer Python only', 'cleanup_seconds': 30,
        'max_targets': 6, 'launcher_pid': os.getpid(), 'parent_group': os.getpgrp()})

    def reject(reason):
        if schedule is not None:
            schedule.fail(reason)
        if reason not in failures:
            failures.append(reason)
        save(OUT / 'stop.json', {'reasons': list(failures), 'outer_elapsed': time.monotonic() - start})

    def directory():
        return OUT / ('target-' + str(index))

    def provision():
        full_target_fits(time.monotonic(), start)
        permit = schedule.permit(time.monotonic(), native)
        d = directory()
        d.mkdir()
        route = create_route(d, run_id(index), BINARY)
        save(d / 'output-route.json', route)
        save(d / 'launch-permit.json', dict(permit, **{'index': index, 'run_id': run_id(index),
            'full_target_seconds': 60, 'outer_elapsed': time.monotonic() - start,
            'previous_completion': receipts[-1] if receipts else None}))

    def inspect():
        with lifecycle.section('native-capture-through-validation'):
            inventory.collect_launch_records(OUT, run_id)
            lifecycle.record(kind='launch-inventory', state='before-validation')
            snapshot = native()
            inventory.collect(snapshot)
            current = custody.observe(snapshot)
            save(directory() / 'custody.json', custody.document())
            if custody.rejections and native.error is None:
                native.error = 'consumer validation failed: ' + custody.rejections[0]['reason']
            require(not custody.rejections, 'custody validation failed: ' + repr(custody.rejections[0]['reason']) if custody.rejections else '')
        save(directory() / 'custody.json', custody.document())
        all_groups.update(custody.groups)
        all_pids.update(custody.native_initial)
        for finding in custody.rejections:
            reject(finding['reason'])
        return current

    def kill_group(group):
        # Failed custody is sticky. Read-only retirement is separate and cannot
        # restore signal authority from an invalid acquisition.
        if native.error or lifecycle.failure or custody.rejections:
            return
        with lifecycle.section('cleanup-authority'):
            allowed = custody.cleanup_allowed(group, native())
        save(directory() / 'custody.json', custody.document())
        for finding in custody.rejections:
            reject(finding['reason'])
        if allowed:
            try:
                os.killpg(group, signal.SIGKILL)
            except ProcessLookupError:
                pass

    def watch():
        nonlocal index, intent_start, custody, debugger_start, release_begin
        try:
            while not done.wait(.1):
                inspect()
                now = time.monotonic()
                if debugger_start is None and any(custody.role(p) == 'debugger' for p in custody.first):
                    debugger_start = now
                d = directory()
                intent = read(d / 'launch-intent.json')
                if intent is not None and intent_start is None:
                    # Conservatively count the external permit-to-launch delay too.
                    intent_start = start + read(d / 'launch-permit.json')['outer_elapsed']
                if debugger_start is not None and intent_start is None and now - debugger_start >= 30:
                    reject('debugger setup exceeded30s')
                if intent_start is not None and now - intent_start >= 60 and not (d / 'next-ready.json').exists():
                    reject('target plus release exceeded60s')
                if now >= end:
                    reject('final cleanup reserve reached')
                if monitor.rejection_reasons():
                    reject('host rejection: ' + repr(monitor.rejection_reasons()))
                if shutil.disk_usage(E).free < 4 * 1024**3:
                    reject('free space below4GiB')
                if sum(p.stat().st_size for p in E.rglob('*') if p.is_file()) >= 2 * 1024**3:
                    reject('evidence exceeds2GiB')
                handshake = read(d / 'inferior.json')
                if handshake and handshake['pid'] in custody.targets and not failures and not (d / 'custody-ack.json').exists():
                    save(d / 'custody-ack.json', custody.acknowledge(handshake))
                report = read(d / 'observer-result.json')
                if report and not failures and not (d / 'next-ready.json').exists():
                    require(report['state'] == 'ENTRIES_COMPLETE', 'observer failed: ' + str(report.get('error')))
                    # Observer atomically updates this after DeleteTarget.
                    if not report.get('target_deleted'):
                        continue
                    if release_begin is None:
                        release_begin = now
                    require(now - release_begin < 10, 'target release exceeded10s')
                    release = target_release(custody, group_exists)
                    save(d / 'target-release.json', release)
                    if release['state'] == 'PASS':
                        with lifecycle.section('per-target-complete-retirement'):
                            complete_inventory = absence(inventory, native.backend,
                                lifecycle.capture, group_exists, release['keepers'],
                                lambda value: save(d / 'complete-inventory-release.json', value))
                            save(d / 'complete-inventory-release.json', complete_inventory)
                            require(complete_inventory['state'] == 'PASS', 'incomplete candidate retirement')
                        completion = complete(read(d / 'output-route.json'), report, handshake,
                            read(d / 'custody-ack.json'), custody.document(), release)
                        save(d / 'output-completion.json', completion)
                        # Offline parsing/invariants are part of live validity; no
                        # next launch can follow invalid intervals or buffered phases.
                        from analyze_boundaries import analyze_target
                        analysis = analyze_target(d, partial=False)
                        save(d / 'analysis.json', analysis)
                        receipt = {'state': 'PASS', 'index': index, 'run_id': run_id(index),
                            'completion_sha256': digest(d / 'output-completion.json'),
                            'release_sha256': digest(d / 'target-release.json')}
                        schedule.finish(time.monotonic(), complete_inventory, completion, receipt, inventory.pids)
                        receipts.append(receipt)
                        save(d / 'next-ready.json', receipt)
                        if index < 5:
                            full_target_fits(time.monotonic(), start)
                            index += 1
                            intent_start = release_begin = None
                            custody = Custody(os.getpid(), os.getpgrp(), BINARY, run_id(index))
                            provision()
                if failures:
                    signal_targets(custody, kill_group)
                    return
        except BaseException:
            reject('watcher: ' + traceback.format_exc())
            signal_targets(custody, kill_group)

    try:
        require('CARGO_TARGET_DIR' not in os.environ, 'shared target forbidden')
        require(Path(os.environ['TMPDIR']).resolve() == E.parent / 'tmp', 'wrong TMPDIR')
        require(not (ROOT / 'target').exists(), 'production build forbidden')
        manifest = read(E / 'prelaunch.json')
        require(manifest is not None, 'missing prelaunch registration')
        for path, expected in manifest['file_hashes'].items():
            require(digest(path) == expected, 'changed frozen input: ' + path)
        for row in manifest['ancestor_config']:
            p = Path(row['path'])
            require(p.exists() == row['exists'], 'ancestor configuration changed')
            if row['exists']:
                require(digest(p) == row['sha256'], 'ancestor config hash changed')
        save(OUT / 'identity.json', manifest)
        require(shutil.disk_usage(E).free >= 4 * 1024**3, 'free space below4GiB')
        require(sum(p.stat().st_size for p in E.rglob('*') if p.is_file()) < 2 * 1024**3, 'evidence cap')
        admission = {'state': 'CHECKING', 'calibration': False, 'snapshots': []}
        for n in range(3):
            elapsed(time.monotonic(), start, 180)
            snapshot = capture_host_snapshot(include_calibration=False, control_mode='work')
            reasons = evaluate_snapshot(snapshot, enforce_load=True, control_mode='work', require_work_counter=False)
            admission['snapshots'].append({'snapshot': snapshot, 'rejections': reasons})
            save(OUT / 'admission.json', admission)
            require(not reasons, 'host admission rejected: ' + repr(reasons))
            if n < 2:
                time.sleep(1)
        elapsed(time.monotonic(), start, 180)
        admission['state'] = 'CONTROLLED_DIAGNOSTIC_ONLY'
        save(OUT / 'admission.json', admission)
        native = NativeCapture(os.getpid(), Darwin(), lifecycle.capture, inventory=inventory,
                               expected_binary=BINARY)
        schedule = Schedule(start, native)
        provision()
        command = ['/usr/bin/lldb', '--no-lldbinit', '-b', '-s', str(E / 'boundaries.lldb')]
        require(command == manifest['debugger_command'], 'unregistered debugger command')
        save(OUT / 'command.json', {'argv': command, 'cwd': str(ROOT)})
        with HostActivityMonitor(control_mode='work') as monitor:
            require(not monitor.rejection_reasons(), 'host rejected at session start')
            watcher = threading.Thread(target=watch, daemon=True, name='B24-custody')
            watcher.start()
            launched = True
            completed, timeout = run_owned_process(command, ROOT, max(.1, end - time.monotonic() - 12))
            save(OUT / 'debugger-result.json', {'exit': completed.returncode, 'timed_out': timeout,
                'stdout': completed.stdout, 'stderr': completed.stderr})
            require(not timeout and completed.returncode == 0, 'debugger failed')
            require(not monitor.rejection_reasons(), 'host rejected completion')
        session = read(OUT / 'session-result.json')
        require(session and session['state'] == 'COMPLETE' and len(receipts) == 6, 'schedule incomplete')
    except BaseException:
        reject(traceback.format_exc())
    finally:
        done.set()
        if watcher and watcher.is_alive():
            watcher.join(timeout=7)
        inventory.collect_launch_records(OUT, run_id)
        lifecycle.record(kind='launch-inventory', state='finalizer-before-retirement')
        absent, remaining = not launched, []
        if native:
            try:
                require(not watcher or not watcher.is_alive(), 'watcher not joined')
                signal_targets(custody, kill_group)
                with lifecycle.section('final-read-only-retirement', seconds=15, retirement=True):
                    release_receipt = absence(inventory, native.backend,
                        lifecycle.capture, group_exists, {os.getpid()},
                        lambda value: save(OUT / 'complete-inventory-release.json', value))
                    save(OUT / 'complete-inventory-release.json', release_receipt)
                    absent = release_receipt['state'] == 'PASS'
                save(directory() / 'final-custody.json', custody.document())
            except BaseException:
                reject('cleanup: ' + traceback.format_exc())
                absent = False
            native.close()
        # Admission/setup helpers also create retirement obligations.
        elif inventory.pids != {os.getpid()}:
            backend = Darwin()
            try:
                with lifecycle.section('admission-retirement', seconds=15, retirement=True):
                    receipt = absence(inventory, backend, lifecycle.capture, group_exists, {os.getpid()},
                        lambda value: save(OUT / 'complete-inventory-release.json', value))
                    save(OUT / 'complete-inventory-release.json', receipt)
                    absent = receipt['state'] == 'PASS'
            except BaseException:
                reject('admission retirement: ' + traceback.format_exc())
                absent = False
            finally:
                backend.close()
        alive = bool(watcher and watcher.is_alive())
        monitor_alive = bool(monitor and monitor._thread and monitor._thread.is_alive())
        if not absent or alive or monitor_alive:
            reject('final resource release unproved')
        if time.monotonic() > deadline:
            reject('whole-window deadline exceeded')
        release = {'groups_absent': absent, 'remaining': remaining, 'groups': sorted(all_groups),
            'pids': sorted(inventory.pids), 'observed_groups': sorted(inventory.groups),
            'watcher_alive': alive, 'monitor_alive': monitor_alive,
            'root_pid': os.getpid(), 'root_status': 'returning; command completion proves exit',
            'elapsed_seconds': time.monotonic() - start}
        save(OUT / 'process-release.json', release)
        if monitor:
            save(OUT / 'host.json', {'snapshots': monitor.snapshots, 'rejections': monitor.rejection_reasons()})
        launches = [read(p) for p in sorted(OUT.glob('target-*/inferior.json'))]
        result.update(state='PASS' if not failures and absent else 'INCONCLUSIVE', failures=failures,
            debugger_sessions=int(launched), target_launches=len(launches), targets=launches,
            completed=receipts, zero_processes=absent and not alive and not monitor_alive,
            elapsed_seconds=time.monotonic() - start)
        save(OUT / 'outcome.json', result)
        print(json.dumps(result, indent=2), flush=True)
    return 0 if result['state'] == 'PASS' else 1

if __name__ == '__main__':
    if sys.argv[1:] == ['--self-test']:
        from boundaries_tests import observer_tests
        raise SystemExit(observer_tests())
    if sys.argv[1:] == ['--allocated-child']:
        raise SystemExit(main())
    require(not sys.argv[1:], 'unregistered arguments')
    from boundaries_supervisor import execute
    raise SystemExit(execute())
