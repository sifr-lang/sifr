"""B36 complete lifecycle contract; no native process launch/query/signals."""
import copy
import hashlib
import json
import tempfile
from pathlib import Path
from coverage_custody import Custody, capture, validate_ack, validate_completion
from coverage_custody_tests import row, snapshot
from coverage_native import NativeCapture, DEBUGGER, DEBUGSERVER
from coverage_native_fakes import enrich
from coverage_cleanup import final_absence
from coverage_symbols import E, require


def context():
    from coverage_output import RUN_ID
    c = Custody(99, 99, '/synthetic/binary', RUN_ID)
    rows = [row(100, 99, 100, DEBUGGER), row(123, 100, 123, c.binary),
            row(124, 100, 124, DEBUGSERVER)]
    c.observe(snapshot(rows))
    return c, {'pid': 123, 'pgid': 123, 'run_id': c.run_id, 'flags': 134}


def next_sample(c, mutate=lambda rows: None, display=False):
    rows = copy.deepcopy(c.last['rows'])
    mutate(rows)
    s = snapshot(rows, c.last['begin_monotonic'] + 1, prior=c.last)
    if display:
        n = s['native']['records']['124']
        for side in ('before', 'after'):
            n[side]['path'] = DEBUGSERVER
            n[side]['comm'] = 'debugserver'
    return s


def attach(c):
    s = next_sample(c, lambda rows: rows[1].update(ppid=124, stat='TX'))
    c.observe(s)
    require(not c.rejections and len(c.transitions) == 1, 'supported attach rejected')


def event(pid, kind):
    return {'pid': pid, 'exec': kind == 'exec', 'exit': kind == 'exit',
            'lost': kind == 'lost', 'eof': kind in ('exit', 'eof'),
            'flags': 0, 'fflags': 0, 'data': 0}


class Backend:
    def __init__(self, rows):
        s = enrich(snapshot(rows))
        self.samples = {int(pid): e['after'] for pid, e in s['native']['records'].items()}
        self.pending, self.registered = [], []
        self.on_register = lambda pid: None
        self.on_poll = lambda: None
        self.closed = False

    def sample(self, pid):
        return copy.deepcopy(self.samples.get(pid))

    def register(self, pid):
        self.registered.append(pid)
        self.on_register(pid)

    def poll(self):
        self.on_poll()
        events, self.pending = self.pending, []
        return events

    def close(self):
        self.closed = True


def run_tests(results):
    def check(name, action, error=None):
        caught = None
        try:
            action()
            passed = error is None
        except Exception as exc:
            caught = repr(exc)
            passed = isinstance(exc, ValueError) and error is not None and error in str(exc)
        results.append({'name': 'B36-' + name, 'pass': passed, 'error': caught,
            'expected_error': error, 'provenance': 'OFFLINE; added native identities/events SYNTHETIC'})

    def success():
        c, h = context()
        initial = copy.deepcopy(c.first[123])
        attach(c)
        c.observe(next_sample(c, lambda rows: rows[2].update(command='(debugserver)', stat='R'), True))
        ack = c.acknowledge(h)
        validate_ack(json.loads(json.dumps(ack)), 123, c.run_id, c.binary)
        require(c.cleanup_allowed(123, next_sample(c, display=True)), 'supported child cleanup denied')
        require(c.cleanup_allowed(124, next_sample(c, display=True)), 'supported tracer cleanup denied')
        require(c.first[123] == initial and ack['process']['ppid'] == 100
                and ack['current']['ppid'] == 124, 'initial/current evidence conflated')
        c.observe(next_sample(c, lambda rows: rows[1].update(ppid=100), True))
        require(len(c.transitions) == 2 and not c.rejections, 'supported detach return rejected')
        remaining, absent = final_absence(c, lambda: snapshot([], 30, prior=c.last), lambda g: False)
        require(remaining == [] and absent, 'native final absence missing')
        validate_completion(ack, json.loads(json.dumps(c.document())), 123, c.run_id, c.binary)
    check('attach-display-cleanup-return-finalization', success)

    def rejected(c, h, s, group=123):
        saved = copy.deepcopy(c.first)
        c.observe(s)
        require(c.rejections and all(c.first[k] == v for k, v in saved.items()),
                'rejection/initial evidence lost')
        require(not c.cleanup_allowed(group, next_sample(c)), 'rejected identity permits signal')
        try:
            c.acknowledge(h)
        except ValueError:
            return
        raise ValueError('rejected identity permits acknowledgement')

    for mode in ('unknown-parent', 'other-lldb-child', 'root-parent', 'foreign-group',
                 'target-replaced', 'old-parent-replaced', 'new-parent-replaced',
                 'root-replaced', 'missing-native', 'short-native', 'path-failure',
                 'exec', 'watch-loss', 'watch-eof', 'registration-replay',
                 'contradictory-bracket', 'foreign-member', 'unknown-ancestor'):
        def negative(mode=mode):
            c, h = context()
            s = next_sample(c, lambda rows: rows[1].update(ppid=124))
            n = s['native']['records']
            if mode == 'unknown-parent':
                s = next_sample(c, lambda rows: rows[1].update(ppid=777))
            if mode == 'other-lldb-child':
                s = next_sample(c, lambda rows: (rows.append(row(200, 99, 200, DEBUGGER)),
                    rows[2].update(ppid=200), rows[1].update(ppid=124)))
            if mode == 'root-parent': s = next_sample(c, lambda rows: rows[1].update(ppid=99))
            if mode == 'foreign-group': s = next_sample(c, lambda rows: rows[1].update(pgid=777))
            if mode in ('target-replaced', 'old-parent-replaced', 'new-parent-replaced', 'root-replaced'):
                pid = {'target-replaced': '123', 'old-parent-replaced': '100',
                       'new-parent-replaced': '124', 'root-replaced': '99'}[mode]
                for side in ('before', 'after'): n[pid][side]['start_usec'] += 1
            if mode == 'missing-native': del s['native']
            if mode == 'short-native': n['123']['after'].pop('start_usec')
            if mode == 'path-failure': s['native']['error'] = 'proc_pidpath failed'
            if mode in ('exec', 'watch-loss', 'watch-eof'):
                n['123']['events'].append(event(123, {'exec': 'exec', 'watch-loss': 'lost', 'watch-eof': 'eof'}[mode]))
            if mode == 'registration-replay': n['123']['registration']['serial'] += 1
            if mode == 'contradictory-bracket': n['123']['after']['path'] = '/replaced'
            if mode == 'foreign-member':
                s = next_sample(c, lambda rows: rows.append(row(777, 1, 123, '/foreign')))
            if mode == 'unknown-ancestor':
                s = next_sample(c, lambda rows: (rows.append(row(777, 99, 777, '/foreign')),
                                                rows[0].update(ppid=777)))
            rejected(c, h, s)
        check('reject-' + mode, negative)

    for pid in (123, 100, 124, 99):
        def executable(pid=pid):
            c, h = context()
            s = next_sample(c)
            for side in ('before', 'after'): s['native']['records'][str(pid)][side]['path'] = '/replacement'
            rejected(c, h, s, group=124 if pid == 124 else 123)
        check('native-executable-replacement-' + str(pid), executable)

    def display_without_native():
        c, h = context()
        rejected(c, h, next_sample(c, lambda rows: rows[2].update(command='(debugserver)')), group=124)
    check('parentheses-alone-never-identity', display_without_native)

    def historical(native=False):
        path = E / 'inputs/b35-custody.json'
        require(hashlib.sha256(path.read_bytes()).hexdigest() ==
                'acd8bacb5e31194a0c577669487caa5a7552e90bf2d4aa5ecd86ae74b08459da', 'B35 raw changed')
        data = json.loads(path.read_text())
        binary = data['initial_observations']['1959']['row']['command']
        c = Custody(1226, 1226, binary, 'B35-replay-only')
        selected = [data['initial_observations']['1959']['observation'],
                    next(r['observation'] for r in data['rejections'] if r['pid'] == 1959),
                    next(r['observation'] for r in data['rejections'] if r['pid'] == 1960)]
        for original in selected:
            s = copy.deepcopy(original)
            s['rows'] = [r for r in s['rows'] if r['pid'] in (1226, 1885, 1959, 1960)]
            s['raw'] = '\n'.join(r['raw'] for r in s['rows'])
            if native:
                enrich(s, c.last)
                for pid, e in s['native']['records'].items():
                    if e['present'] and e['after']['path'].startswith('('):
                        for side in ('before', 'after'):
                            e[side]['path'] = e['registration']['after']['path']
                            e[side]['comm'] = e['registration']['after']['comm']
            c.observe(s)
        if native:
            require(not c.rejections and c.transitions and
                    c.first[1959]['row']['ppid'] == 1885,
                    'B35 shape synthetic augmentation failed: ' + repr([r['reason'] for r in c.rejections]))
            h = {'pid': 1959, 'pgid': 1959, 'flags': 134, 'run_id': c.run_id}
            validate_ack(c.acknowledge(h), 1959, c.run_id, binary)
        else:
            require(c.rejections and not c.targets, 'historical missing native identity accepted')
        require(json.loads((E / 'inputs/b35-outcome.json').read_text())['state'] == 'INCONCLUSIVE',
                'B35 failure relabeled')
    check('actual-B35-raw-native-identity-remains-UNKNOWN', historical)
    check('actual-B35-row-shape-with-explicit-synthetic-native-evidence', lambda: historical(True))

    for mode in ('registration-parent-race', 'registration-start-race', 'exec-during-registration',
                 'registration-error', 'poll-error', 'path-error', 'short-sample', 'ordinary'):
        def producer(mode=mode):
            c, h = context()
            rows = c.last['rows']
            backend = Backend(rows)
            if mode == 'registration-parent-race':
                backend.on_register = lambda pid: backend.samples[pid].update(ppid=777)
            if mode == 'registration-start-race':
                backend.on_register = lambda pid: backend.samples[pid].update(start_usec=999)
            if mode == 'exec-during-registration':
                backend.on_register = lambda pid: backend.pending.append(event(pid, 'exec'))
            def fail(*args): raise ValueError('injected API failure')
            if mode == 'registration-error': backend.on_register = fail
            if mode == 'poll-error': backend.on_poll = fail
            if mode == 'path-error': backend.sample = fail
            if mode == 'short-sample': backend.sample = lambda pid: {'pid': pid}
            producer = NativeCapture(99, backend, lambda: snapshot(rows, 20), lambda: 20)
            s = producer()
            owned = Custody(99, 99, c.binary, c.run_id)
            owned.observe(s)
            if mode == 'ordinary':
                require(not owned.rejections and owned.targets == {123}, 'injected native producer failed')
            else:
                require(owned.rejections, 'producer race/error accepted')
            producer.close()
            require(backend.closed, 'native handle not released')
        check('producer-' + mode, producer)

    for notified in (False, True):
        def terminated(notified=notified):
            c, h = context()
            attach(c)
            ack = c.acknowledge(h)
            s = next_sample(c)
            if notified: s['native']['records']['123']['events'].append(event(123, 'exit'))
            c.observe(s)
            if notified:
                require(not c.cleanup_allowed(123, next_sample(c)), 'exited identity signaled')
                try: c.acknowledge(h)
                except ValueError: pass
                else: raise ValueError('exit event allows live acknowledgement')
            # Delayed NOTE_EXIT does not prevent fresh ESRCH + ps absence.
            remaining, absent = final_absence(c, lambda: snapshot([], 30, prior=c.last), lambda g: False)
            require(absent and not remaining and not c.rejections, 'expected termination rejected')
            validate_completion(ack, c.document(), 123, c.run_id, c.binary)
        check('termination-with-' + ('delivered' if notified else 'delayed') + '-exit', terminated)

    def reappearance():
        c, h = context()
        original = copy.deepcopy(c.last['rows'])
        c.observe(snapshot([], 20, prior=c.last))
        rejected(c, h, snapshot(original, 21, prior=c.last))
    check('departed-PID-never-readopted-even-identical-samples', reappearance)

    for phase in ('ack', 'completion'):
        for mode in ('lost-transition', 'lost-event', 'replaced-registration', 'native-path', 'foreign-parent'):
            def consumer(phase=phase, mode=mode):
                c, h = context()
                attach(c)
                ack = c.acknowledge(h)
                document = ack['custody'] if phase == 'ack' else c.document()
                if mode == 'lost-transition': document['transitions'] = []
                last = document['observations'][-1]
                record = last['native']['records']['123']
                if mode == 'lost-event': record['events'].append(event(123, 'exec'))
                if mode == 'replaced-registration': record['registration']['serial'] += 1
                if mode == 'native-path': record['after']['path'] = '/replacement'
                if mode == 'foreign-parent': last['rows'][1]['ppid'] = 777
                try:
                    if phase == 'ack': validate_ack(ack, 123, c.run_id, c.binary)
                    else: validate_completion(ack, document, 123, c.run_id, c.binary)
                except ValueError: return
                raise ValueError('tampered consumer evidence accepted')
            check(phase + '-replay-reject-' + mode, consumer)

    def final_output(stale=False):
        from coverage_output import RUN_ID, create_route, configure_launch, finalize
        from coverage_output_tests import SyntheticLaunch, GOOD
        c, h = context()
        attach(c)
        ack = c.acknowledge(h)
        root = Path(tempfile.mkdtemp(prefix='b36-output-'))
        route = create_route(root, RUN_ID, c.binary)
        launch = SyntheticLaunch()
        actions = configure_launch(launch, route, root, RUN_ID, c.binary)
        launch.produce(b'', GOOD)
        if not stale: final_absence(c, lambda: snapshot([], 30, prior=c.last), lambda g: False)
        observer = {'state': 'ENTRIES_COMPLETE', 'stage': 'completion', 'run_id': c.run_id,
            'pid': 123, 'target_launches': 1, 'output_actions': actions, 'exit': 0, 'exited': True,
            'final_phase': 'active', 'handover_count': 1,
            'hits': ['prepare', 'fixups', 'generic', 'libsystem', 'sanitizers', 'constructor', 'main', 'completion']}
        release = {'remaining': [], 'groups_absent': True, 'watcher_alive': False, 'monitor_alive': False}
        result = finalize(route, observer, h, ack, c.document(), release)
        require(result['state'] == 'PASS', 'full output lifecycle not complete')
    check('producer-ack-transition-release-real-final-output', final_output)
    check('final-output-rejects-boolean-release-with-live-native-subjects',
          lambda: final_output(True), 'fresh native process absence unproved')
