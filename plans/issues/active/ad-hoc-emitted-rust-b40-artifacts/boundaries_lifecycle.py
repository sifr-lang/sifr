"""B39 bounded producer serialization and read-only retirement obligations.

The lock spans host snapshot spawn through reap, or the entire native acquisition
and consumer validation. Historical candidates never grant signal authority.
"""
import copy
import subprocess
import threading
import time
from contextlib import contextmanager
from coverage_custody import ancestry, validate_snapshot
from coverage_symbols import require
from boundaries_inventory import Inventory, validate_absence


class Lifecycle:
    def __init__(self, inventory, deadline, sink=lambda value: None,
                 clock=time.monotonic, lock=None, popen=subprocess.Popen):
        self.inventory, self.deadline, self.sink = inventory, deadline, sink
        self.clock, self.lock, self.popen = clock, lock or threading.RLock(), popen
        self.local = threading.local()
        self.events, self.failure = [], None
        self.probes = set()

    def record(self, **row):
        self.events.append(dict(row, monotonic=self.clock()))
        self.sink({'events': self.events, 'failure': self.failure,
                   'inventory': self.inventory.document()})

    @contextmanager
    def section(self, kind, seconds=6, retirement=False):
        begin = self.clock()
        end = min(self.deadline, begin + seconds, getattr(self.local, 'end', self.deadline))
        remaining = end - begin
        acquired = remaining > 0 and self.lock.acquire(timeout=remaining)
        if not acquired:
            if not retirement and self.failure is None:
                self.failure = 'bounded lifecycle lock deadline'
            raise ValueError('bounded lifecycle lock deadline')
        previous = getattr(self.local, 'end', None)
        self.local.end = end
        try:
            require(retirement or self.failure is None, 'sticky producer failure: ' + str(self.failure))
            require(self.clock() < end, 'lifecycle deadline after lock wait')
            self.record(kind=kind, state='entered', requested=begin, deadline=end)
            yield
            require(retirement or self.failure is None, 'sticky producer failure: ' + str(self.failure))
            require(self.clock() < end, 'lifecycle deadline exceeded')
        except BaseException as error:
            if not retirement and self.failure is None:
                self.failure = str(error)
            self.record(kind=kind, state='failed', error=str(error))
            raise
        finally:
            self.local.end = previous if previous is not None else self.deadline
            try:
                self.record(kind=kind, state='released')
            finally:
                self.lock.release()

    def run(self, command, text=True, capture_output=True, timeout=2, env=None):
        require(self.lock._is_owned(), 'subprocess outside lifecycle ownership')
        end = min(self.local.end, self.clock() + timeout + 1)
        require(end - self.clock() > 1, 'helper deadline cannot fit reap reserve')
        process = self.popen(command, text=text, stdout=subprocess.PIPE,
                             stderr=subprocess.PIPE, env=env)
        self.inventory.pids.add(process.pid)
        prior_probes = sorted(self.probes)
        self.probes.add(process.pid)
        self.record(kind='helper', state='spawned', pid=process.pid,
                    argv=list(command), prior_probe_pids=prior_probes)
        try:
            stdout, stderr = process.communicate(timeout=min(timeout, end - self.clock() - 1))
        except BaseException as error:
            if self.failure is None:
                self.failure = 'helper failed: ' + str(error)
            self.record(kind='helper', state='failed', pid=process.pid, error=str(error),
                        partial_stdout=str(getattr(error, 'output', None)),
                        partial_stderr=str(getattr(error, 'stderr', None)))
            if process.poll() is None:
                process.kill()  # Only the retained exact Popen handle, no PID lookup.
            try:
                process.communicate(timeout=max(.001, end - self.clock()))
            finally:
                self.record(kind='helper', state='cleanup', pid=process.pid, exit=process.poll())
            raise
        self.record(kind='helper', state='reaped', pid=process.pid, exit=process.returncode)
        if process.returncode != 0 and self.failure is None:
            self.failure = 'helper nonzero exit: ' + str(process.returncode)
        require(process.returncode == 0, 'helper nonzero exit')
        require(self.clock() < end and process.returncode is not None, 'helper reap deadline')
        result = subprocess.CompletedProcess(command, process.returncode, stdout, stderr)
        result.probe_pid = process.pid
        return result

    def capture(self):
        from coverage_custody import capture
        result = capture(run=self.run, monotonic=self.clock)
        self.inventory.collect(result)
        return result


def absence(inventory, backend, capture, group_exists, keepers=(), sink=lambda value: None):
    """Fresh native-before/full-ps/native-after, independent of sticky authority."""
    subjects = set(inventory.pids) - set(keepers)
    receipt = {'pids': sorted(subjects), 'keepers': sorted(keepers), 'before': {},
               'after': {}, 'groups': {}, 'signals': [], 'state': 'FAIL', 'authority': 'absence only'}
    sink(receipt)
    before = receipt['before']
    for p in sorted(subjects):
        before[str(p)] = backend.sample(p)
        sink(receipt)
    snapshot = capture()
    inventory.collect(snapshot)
    # This capture's exact helper is already reaped; prove its absence too.
    extra = inventory.pids - subjects - set(keepers)
    subjects.update(extra)
    receipt.update(snapshot=snapshot, pids=sorted(subjects),
                   discovered_during_capture=sorted(extra))
    sink(receipt)
    after = {str(p): backend.sample(p) for p in sorted(subjects)}
    groups = {str(g): group_exists(g) for g in sorted(inventory.groups - set(keepers))}
    receipt.update(after=after, groups=groups)
    sink(receipt)
    validate_snapshot(snapshot)
    current_probe = snapshot.get('probe_pid')
    present = {r['pid'] for r in snapshot['rows'] if r['pid'] != current_probe}
    # Every extra non-probe discovery invalidates this bracket, rather than
    # pretending its post-table sample was a before-table observation.
    receipt['state'] = 'PASS' if (extra <= {current_probe}
        and not (present & subjects) and all(v is None for v in before.values())
        and all(v is None for v in after.values()) and all(v is False for v in groups.values())
        and not inventory.errors) else 'FAIL'
    receipt['current_probe_rule'] = 'exact capture handle reaped; only post-capture native sample exists for a newly spawned probe'
    sink(receipt)
    if receipt['state'] == 'PASS':
        validate_absence(receipt, inventory, keepers)
    return receipt
