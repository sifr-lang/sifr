"""One allocation owner: actual launcher exit and root-inclusive retirement."""
import json
import os
import subprocess
import sys
import time
import traceback
from boundaries_core import E, ROOT, OUT, BINARY, run_id, read, save
from boundaries_lifecycle import Inventory, Lifecycle, absence
from coverage_native import Darwin
from coverage_symbols import require


def execute():
    require(not (E / 'allocation.json').exists() and not OUT.exists(), 'allocation already consumed')
    start = time.monotonic()
    deadline = start + 540
    record = {'state': 'INCONCLUSIVE', 'begin': start, 'deadline': deadline,
              'clock_owner': 'supervisor local monotonic', 'setup_attempts': 1,
              'command': [sys.executable, str(E / 'run_boundaries.py'), '--allocated-child']}
    save(E / 'allocation.json', record)
    child = None
    backend = None
    retirement_started = False
    try:
        child = subprocess.Popen(record['command'], cwd=ROOT, env=dict(os.environ),
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, start_new_session=True)
        record['root_pid'] = child.pid
        save(E / 'allocation.json', record)
        # Observation ends by this independent supervisor's510s clock too.
        try:
            stdout, stderr = child.communicate(timeout=max(.001, start + 510 - time.monotonic()))
        except subprocess.TimeoutExpired:
            save(OUT / 'stop.json', {'reasons': ['supervisor final cleanup reserve reached']})
            try:
                stdout, stderr = child.communicate(timeout=max(.001, start + 534 - time.monotonic()))
            except subprocess.TimeoutExpired:
                # Only exact owned root handle. No inferred PID/group authority.
                child.kill()
                stdout, stderr = child.communicate(timeout=max(.001, start + 535 - time.monotonic()))
        record.update(root_exit=child.returncode, stdout=stdout, stderr=stderr,
                      root_exit_elapsed=time.monotonic() - start)
        save(E / 'launcher-exit.json', record)
        inv = Inventory(child.pid, BINARY)
        inv.groups.add(child.pid)
        producer = read(OUT / 'producer-lifecycle.json')
        if producer is not None:
            inv.load(producer['inventory'])
        else:
            inv.errors.append('missing complete launcher inventory')
        inv.collect_launch_records(OUT, run_id)
        save(E / 'supervisor-inventory.json', inv.document())
        life = Lifecycle(inv, deadline)
        backend = Darwin()
        retirement_started = True
        with life.section('root-inclusive-final-retirement', seconds=min(15, deadline - time.monotonic()), retirement=True):
            receipt = absence(inv, backend, life.capture, group_exists,
                              sink=lambda value: save(E / 'complete-process-release.json', value))
            receipt['launcher_exit'] = child.returncode
            inner = read(OUT / 'process-release.json') or {}
            receipt['watcher_alive'] = receipt['monitor_alive'] = False if child.returncode is not None else None
            receipt['watcher_join_recorded'] = inner.get('watcher_alive') is False
            receipt['monitor_join_recorded'] = inner.get('monitor_alive') is False
            if not receipt['watcher_join_recorded'] or not receipt['monitor_join_recorded']:
                receipt['state'] = 'FAIL'
                receipt['error'] = 'launcher did not record watcher/monitor join'
            receipt['root_exit_evidence'] = str(E / 'launcher-exit.json')
            save(E / 'complete-process-release.json', receipt)
        outcome = read(OUT / 'outcome.json')
        record['release'] = receipt['state']
        record['outcome'] = outcome
        require(receipt['state'] == 'PASS', 'root-inclusive retirement unproved')
        require(time.monotonic() <= deadline, 'whole allocation exceeded540s')
        record['state'] = 'PASS' if outcome and outcome['state'] == 'PASS' and child.returncode == 0 else 'INCONCLUSIVE'
    except BaseException:
        record['error'] = traceback.format_exc()
    finally:
        if child is not None and child.poll() is None:
            try:
                if OUT.exists():
                    save(OUT / 'stop.json', {'reasons': ['supervisor terminal failure']})
                child.communicate(timeout=max(.001, deadline - time.monotonic() - 2))
            except subprocess.TimeoutExpired:
                child.kill()
                child.communicate(timeout=max(.001, deadline - time.monotonic()))
        # Even an exception before normal communicate/launch-record reads must
        # retain the exact root exit and already-written six target obligations.
        if child is not None and child.poll() is not None and not retirement_started:
            record.update(root_exit=child.returncode, root_exit_elapsed=time.monotonic() - start)
            save(E / 'launcher-exit.json', record)
            try:
                inv = Inventory(child.pid, BINARY)
                inv.groups.add(child.pid)
                producer = read(OUT / 'producer-lifecycle.json')
                if producer:
                    inv.load(producer['inventory'])
                else:
                    inv.errors.append('missing complete launcher inventory')
                inv.collect_launch_records(OUT, run_id)
                save(E / 'supervisor-inventory.json', inv.document())
                retirement_started = True
                if backend is None:
                    backend = Darwin()
                life = Lifecycle(inv, deadline)
                with life.section('exception-root-inclusive-retirement', seconds=15, retirement=True):
                    receipt = absence(inv, backend, life.capture, group_exists,
                        sink=lambda value: save(E / 'complete-process-release.json', value))
                    inner = read(OUT / 'process-release.json') or {}
                    receipt.update(launcher_exit=child.returncode,
                        watcher_join_recorded=inner.get('watcher_alive') is False,
                        monitor_join_recorded=inner.get('monitor_alive') is False)
                    if not receipt['watcher_join_recorded'] or not receipt['monitor_join_recorded'] or time.monotonic()>deadline:
                        receipt['state']='FAIL'
                    save(E / 'complete-process-release.json',receipt)
                    record['release']=receipt['state']
            except BaseException:
                record['retirement_error']=traceback.format_exc()
        if backend is not None:
            backend.close()
        record['elapsed_seconds'] = time.monotonic() - start
        save(E / 'allocation-outcome.json', record)
        print(json.dumps({k: record.get(k) for k in ('state','root_pid','root_exit','release','elapsed_seconds','error')}, indent=2), flush=True)
    return 0 if record['state'] == 'PASS' else 1


def group_exists(group):
    try:
        os.killpg(group, 0)
    except ProcessLookupError:
        return False
    return True
