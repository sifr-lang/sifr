"""B35-F1 synthetic changing-role scheduling and final-release negatives."""
import copy
from coverage_cleanup import drain, final_absence, signal_targets
from coverage_custody import Custody
from coverage_custody_tests import row, snapshot, synthetic_context
from coverage_symbols import require


class World:
    def __init__(self, initial=None):
        self.custody = Custody(99, 99, '/synthetic/binary', 'B35-synthetic')
        self.rows = initial if initial is not None else [row(100, 99, 100, '/synthetic/lldb')]
        self.tick = 10
        self.clock = 0
        self.signals = []
        self.requests = []
        self.before_kill = lambda group: None
        self.keep_alive = False
        self.inspect()

    def capture(self):
        self.tick += 1
        return snapshot(self.rows, self.tick, prior=self.custody.last)

    def inspect(self):
        return self.custody.observe(self.capture())

    def exists(self, group):
        return any(r['pgid'] == group for r in self.rows)

    def kill(self, group):
        self.requests.append(group)
        self.before_kill(group)
        if self.custody.cleanup_allowed(group, self.capture()):
            self.signals.append(group)
            if not self.keep_alive:
                self.rows = [r for r in self.rows if r['pgid'] != group]

    def sleep(self, elapsed):
        self.clock += elapsed

    def drain(self):
        drain(self.custody, self.inspect, self.kill, self.exists, 0.2,
              lambda: self.clock, self.sleep)

    def release(self):
        return final_absence(self.custody, self.capture, self.exists)


def run_tests(results):
    def check(name, action):
        error = None
        try:
            action()
            passed = True
        except Exception as exc:
            passed, error = False, repr(exc)
        results.append({'name': 'cleanup-' + name, 'pass': passed, 'error': error,
                        'provenance': 'SYNTHETIC rows/clocks/signals; no live process'})

    def new_group():
        w = World()
        def grow(group):
            if len(w.requests) == 1:
                w.rows.append(row(200, 99, 200, '/synthetic/lldb'))
        w.before_kill = grow
        w.drain()
        require(w.signals == [100, 200] and 200 in w.custody.first
                and not w.custody.rejections and w.release() == ([], True),
                'new group skipped, unowned or release unproved')
    check('group-grows-during-fresh-cleanup-observation', new_group)

    def new_target():
        w = World([row(123, 99, 123, '/synthetic/binary')])
        w.before_kill = lambda g: w.rows.append(row(124, 99, 124, '/synthetic/binary'))
        signal_targets(w.custody, w.kill)
        require(w.custody.targets == {123} and w.requests == [123]
                and any(any(r['pid'] == 124 for r in rejection['observation']['rows'])
                        for rejection in w.custody.rejections),
                'target iteration changed accepted state or lost rejected candidate')
        require(w.custody.rejections and 124 not in w.signals,
                'second target bypassed custody or snapshot scheduling')
        require(not w.release()[1], 'new rejected target treated absent')
    check('target-set-growth-retains-single-target-rejection', new_target)

    def late_role():
        w = World()
        def grow(group):
            if len(w.requests) == 1:
                w.rows.append(row(123, 99, 123, w.custody.binary))
        w.before_kill = grow
        w.drain()
        require(123 in w.custody.first and 123 in w.signals and w.release() == ([], True),
                'late sole target not discovered/authenticated/drained')
    check('sole-target-discovered-during-group-pass', late_role)

    for failure in ('foreign-member', 'changed-ancestor', 'wrong-group', 'unknown-state'):
        def rejected(failure=failure):
            w = World()
            def change(group):
                if len(w.requests) != 1:
                    return
                if failure == 'foreign-member':
                    w.rows.append(row(201, 1, 100, '/foreign'))
                elif failure == 'changed-ancestor':
                    w.rows[0]['created'] = 'Tue Sep 8 08:00:00 2026'
                    w.rows.append(row(123, 100, 123, w.custody.binary))
                elif failure == 'wrong-group':
                    w.rows.append(row(123, 99, 99, w.custody.binary))
                else:
                    w.rows[0]['stat'] = '?'
            w.before_kill = change
            w.drain()
            require(w.custody.rejections and not w.release()[1], 'rejected role proved absent')
            require(123 not in w.signals and (failure == 'wrong-group' or 100 not in w.signals),
                    'new role bypassed unchanged signal authority')
        check('new-role-reject-' + failure, rejected)

    def bounded():
        w = World()
        w.keep_alive = True
        w.drain()
        require(0.2 <= w.clock < 0.3 and len(w.requests) <= 5
                and not w.release()[1], 'bounded cleanup claimed release or failed to stop')
    check('surviving-group-exhausts-bounded-discovery', bounded)

    for kind in ('new-debugger', 'new-target', 'foreign-known-group', 'binary-unowned',
                 'known-helper', 'capture-invalid', 'kernel-group-survives'):
        def final_negative(kind=kind):
            w = World([row(100, 99, 100, '/synthetic/lldb'), row(101, 99, 99, '/helper')])
            w.rows = []
            require(w.release() == ([], True), 'empty reference release denied')
            if kind == 'new-debugger': w.rows = [row(200, 99, 200, '/synthetic/lldb')]
            if kind == 'new-target': w.rows = [row(123, 99, 123, w.custody.binary)]
            if kind == 'foreign-known-group': w.rows = [row(201, 1, 100, '/foreign')]
            if kind == 'binary-unowned': w.rows = [row(123, 1, 123, w.custody.binary)]
            if kind == 'known-helper': w.rows = [row(101, 99, 99, '/helper')]
            if kind == 'capture-invalid':
                w.capture = lambda: snapshot([], 50, error='synthetic capture unavailable')
            if kind == 'kernel-group-survives': w.exists = lambda g: True
            require(not w.release()[1], 'stale scheduling state used to prove release')
        check('fresh-final-release-reject-' + kind, final_negative)

    def final_custody_output():
        # Reuse the actual combined module/custody/output decision suite retained
        # by B34; additionally connect actual final absence to its output consumer.
        from coverage_output import validate_provenance
        c, handshake, ack = synthetic_context()
        late = copy.deepcopy(c.last['rows']) + [row(200, 99, 200, '/synthetic/lldb')]
        remaining, absent = final_absence(c, lambda: snapshot(late, 20, prior=c.last), lambda g: True)
        release = {'remaining': remaining, 'groups_absent': absent,
                   'watcher_alive': False, 'monitor_alive': False}
        route = {'run_id': c.run_id, 'binary': c.binary,
                 'streams': {name: {'path': '/synthetic/' + name} for name in ('stdout', 'stderr')}}
        observer = {'state': 'ENTRIES_COMPLETE', 'stage': 'completion', 'run_id': c.run_id,
                    'pid': 123, 'target_launches': 1, 'exit': 0, 'exited': True,
                    'output_actions': [{'fd': fd, 'path': '/synthetic/' + name,
                                        'read': False, 'write': True}
                                       for fd, name in ((1, 'stdout'), (2, 'stderr'))]}
        try:
            validate_provenance(route, observer, handshake, ack, c.document(), release)
        except ValueError as exc:
            require('process release incomplete' in str(exc), 'unexpected decision rejection')
            return
        raise ValueError('output accepted surviving newly discovered group')
    check('fresh-release-feeds-real-output-decision', final_custody_output)
