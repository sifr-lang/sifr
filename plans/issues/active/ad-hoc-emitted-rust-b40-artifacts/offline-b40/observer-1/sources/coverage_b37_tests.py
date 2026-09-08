"""B37 encompassing offline additions; injected probes/native data only."""
import copy
import json
import tempfile
from pathlib import Path
from types import SimpleNamespace
from coverage_custody import (Custody, capture, validate_ack, validate_completion,
                              replay_document)
from coverage_custody_tests import row, snapshot
from coverage_lifecycle_tests import context, attach, next_sample, Backend, event
from coverage_native import NativeCapture, DEBUGGER, DEBUGSERVER
from coverage_cleanup import final_absence
from coverage_symbols import require


def race_context(known=False, mode='gone'):
    c, h = context()
    rows = copy.deepcopy(c.last['rows'])
    backend = Backend(rows)
    clock = [15.0]
    state = {'rows': rows, 'probe': None, 'calls': 0}
    def tick():
        clock[0] += .1
        return clock[0]
    def ps():
        state['calls'] += 1
        result = snapshot(state['rows'], tick(), probe_pid=state['probe'])
        if state['calls'] >= 3 and mode == 'stale':
            result = snapshot(state['rows'], 1)
        tick()
        return result
    producer = NativeCapture(99, backend, ps, tick)
    owned = Custody(99, 99, c.binary, c.run_id)
    owned.observe(producer())
    require(not owned.rejections, 'initial producer rejected')
    pid = 124 if known else 125
    if not known:
        state['rows'].append(row(pid, 100, pid, '/synthetic/transient'))
    backend.samples.pop(pid, None)
    original_ps = producer.ps_capture
    def disappearing_ps():
        result = original_ps()
        if mode not in ('still-listed', 'bound'):
            state['rows'] = [r for r in state['rows'] if r['pid'] != pid]
        if mode == 'native-return':
            original_sample = backend.sample
            returned = [False]
            def sample(subject):
                if subject == pid and not returned[0]:
                    returned[0] = True
                    backend.samples[pid] = Backend([row(pid, 100, pid, '/replacement')]).samples[pid]
                    return None
                return original_sample(subject)
            backend.sample = sample
        if mode == 'bound': clock[0] += 7
        return result
    producer.ps_capture = disappearing_ps
    return owned, h, producer, state, backend, pid


def terminal_snapshot(c, mode='zombie'):
    rows = copy.deepcopy(c.last['rows'])
    rows = [r for r in rows if r['pid'] != 124]
    for r in rows:
        if r['pid'] == 123:
            r['stat'] = 'Z' if mode != 'live' else 'T'
    s = snapshot(rows, c.last['begin_monotonic'] + 1, prior=c.last)
    if mode != 'live':
        for side in ('before', 'after'):
            s['native']['records']['123'][side]['status'] = 5
    return s


def run_tests(results):
    def check(name, action):
        error = None
        try:
            action()
        except Exception as exc:
            error = repr(exc)
        results.append({'name': 'B37-' + name, 'pass': error is None,
                        'error': error, 'provenance': 'OFFLINE injected native/probe evidence'})

    def refuses(action):
        try: action()
        except (ValueError, KeyError, TypeError): return
        raise ValueError('invalid evidence unexpectedly accepted')

    for known in (False, True):
        def gone(known=known):
            c, h, p, state, backend, pid = race_context(known)
            s = p()
            c.observe(s)
            require(s['native']['error'] is None and not c.rejections,
                    'confirmed departure rejected: ' + repr(c.rejections))
            require(any(r['pid'] == pid for r in s['rows'])
                    and len(s['native']['corroborations']) == 1
                    and not s['native']['records'][str(pid)]['present'],
                    'original or corroborating observations lost')
            ack = c.acknowledge(h)
            validate_ack(json.loads(json.dumps(ack)), 123, c.run_id, c.binary)
            signal_copy = replay_document(c.document(), c.binary, c.run_id)
            require(not signal_copy.cleanup_allowed(pid, p()), 'absent subject gained signal authority')
            state['rows'] = [r for r in state['rows'] if r['pid'] == 99]
            backend.samples = {99: backend.samples[99]}
            remaining, absent = final_absence(c, p, lambda g: False)
            require(absent and not remaining, 'confirmed disappearance failed final release')
            validate_completion(ack, json.loads(json.dumps(c.document())), 123, c.run_id, c.binary)
        check('acquisition-gone-' + str(known) + '-producer-ack-cleanup-finalization', gone)

    for mode in ('still-listed', 'native-return', 'stale', 'bound'):
        def negative(mode=mode):
            c, h, p, state, backend, pid = race_context(mode=mode)
            c.observe(p())
            require(c.rejections, 'ambiguous acquisition accepted')
            refuses(lambda: c.acknowledge(h))
            require(p.error is not None, 'acquisition failure not sticky')
        check('acquisition-reject-' + mode, negative)

    def replacement():
        c, h, p, state, backend, pid = race_context()
        c.observe(p())
        state['rows'].append(row(pid, 100, pid, '/synthetic/transient'))
        backend.samples[pid] = Backend([state['rows'][-1]]).samples[pid]
        p.ps_capture = lambda: snapshot(state['rows'], 25)
        c.observe(p())
        require(c.rejections and p.error is not None, 'departed PID readopted')
    check('acquisition-return-sticky-rejection', replacement)

    for field in ('before', 'after', 'initial_sample', 'end', 'snapshot', 'records'):
        def tamper(field=field):
            c, h, p, state, backend, pid = race_context()
            c.observe(p())
            ack = c.acknowledge(h)
            s = ack['custody']['observations'][-1]
            evidence = s['native']['corroborations'][0]
            if field in ('before', 'after', 'initial_sample'): evidence[field] = {'pid': pid}
            if field == 'end': evidence[field] += 20
            if field == 'snapshot': evidence[field] = copy.deepcopy(s)
            if field == 'records': s['native']['records'][str(pid)]['present'] = True
            refuses(lambda: validate_ack(ack, 123, c.run_id, c.binary))
        check('ack-reject-corroboration-tamper-' + field, tamper)

    for mode in ('zombie', 'delivered-exit'):
        def terminal(mode=mode):
            c, h = context()
            attach(c)
            ack = c.acknowledge(h)
            s = terminal_snapshot(c)
            if mode == 'delivered-exit': s['native']['records']['123']['events'].append(event(123, 'exit'))
            c.observe(s)
            require(not c.rejections, 'terminal-only tracer departure rejected')
            refuses(lambda: c.acknowledge(h))
            require(not c.cleanup_allowed(123, terminal_snapshot(c)), 'terminal inferior signaled')
            require(not c.cleanup_allowed(124, terminal_snapshot(c)), 'missing tracer signaled')
            remaining, absent = final_absence(c, lambda: snapshot([], 30, prior=c.last), lambda g: False)
            require(absent and not remaining and not c.rejections, 'terminal release failed')
            validate_completion(ack, json.loads(json.dumps(c.document())), 123, c.run_id, c.binary)
        check('terminal-tracer-' + mode + '-ack-cleanup-finalization', terminal)

    for mode in ('live', 'ps-only', 'exit-only', 'no-prior-chain', 'tracer-replaced',
                 'inferior-replaced', 'watch-loss', 'unknown-parent', 'foreign-group'):
        def bad_terminal(mode=mode):
            c, h = context()
            if mode != 'no-prior-chain': attach(c)
            ack = c.acknowledge(h)
            s = terminal_snapshot(c, 'live' if mode in ('live', 'ps-only', 'exit-only') else 'zombie')
            if mode == 'ps-only':
                rows = copy.deepcopy(s['rows'])
                next(r for r in rows if r['pid'] == 123)['stat'] = 'Z'
                s = snapshot(rows, s['begin_monotonic'], prior=c.last)
            if mode == 'exit-only': s['native']['records']['123']['events'].append(event(123, 'exit'))
            if mode == 'no-prior-chain':
                # Force claimed tracer ancestry without a preceding authenticated transition.
                rows = copy.deepcopy(s['rows'])
                next(r for r in rows if r['pid'] == 123)['ppid'] = 124
                s = snapshot(rows, s['begin_monotonic'], prior=c.last)
            if mode == 'tracer-replaced': s['native']['records']['124']['registration']['serial'] += 1
            if mode == 'inferior-replaced': s['native']['records']['123']['after']['start_usec'] += 1
            if mode == 'watch-loss': s['native']['records']['123']['events'].append(event(123, 'lost'))
            if mode in ('unknown-parent', 'foreign-group'):
                rows = copy.deepcopy(s['rows'])
                next(r for r in rows if r['pid'] == 123).update(
                    {('ppid' if mode == 'unknown-parent' else 'pgid'): 777})
                s = snapshot(rows, s['begin_monotonic'], prior=c.last)
            c.observe(s)
            require(c.rejections, 'invalid terminal tracer evidence accepted')
            refuses(lambda: c.acknowledge(h))
            refuses(lambda: validate_completion(ack, c.document(), 123, c.run_id, c.binary))
            require(not c.cleanup_allowed(123, terminal_snapshot(c)), 'invalid terminal signal authority')
        check('terminal-tracer-reject-' + mode, bad_terminal)

    def exact_probe():
        raw = '777 99 99 Tue Sep 8 07:05:45 2026 R /bin/ps'
        s = capture(run=lambda *a, **k: SimpleNamespace(stdout=raw, stderr='',
                    returncode=0, probe_pid=777))
        require(s['probe_pid'] == 777, 'spawned probe PID not retained')
    check('producer-preserves-actual-spawned-probe-pid', exact_probe)

    for actual in (True, False):
        def probe(actual=actual):
            c, h = context()
            rows = [row(99, 1, 99, '/synthetic/launcher'), row(777, 99, 99, '/bin/ps')]
            backend = Backend(rows)
            backend.samples.pop(777)
            p = NativeCapture(99, backend,
                lambda: snapshot(rows, 20, probe_pid=777 if actual else 888), lambda: 20)
            owned = Custody(99, 99, c.binary, c.run_id)
            s = p()
            owned.observe(s)
            if actual:
                require(not owned.rejections and '777' not in s['native']['records'], 'actual probe not exempt')
                _, absent = final_absence(owned, lambda: snapshot(rows, 21, prior=s, probe_pid=777), lambda g: False)
                require(absent, 'own probe prevents final release')
            else:
                require(owned.rejections, 'unrelated ps silently exempt')
        check('probe-pid-binding-' + str(actual), probe)

    def lingering_ps():
        c, h = context()
        ack = c.acknowledge(h)
        rows = [row(777, 99, 777, '/bin/ps')]
        s = snapshot(rows, 20, prior=c.last, probe_pid=888)
        remaining, absent = final_absence(c, lambda: s, lambda g: False)
        require(not absent and any(r['pid'] == 777 for r in remaining), 'foreign ps omitted at release')
        refuses(lambda: c.acknowledge(h))
    check('foreign-ps-remains-visible-at-final-release', lingering_ps)

    def output_terminal():
        from coverage_output import RUN_ID, create_route, configure_launch, finalize
        from coverage_output_tests import SyntheticLaunch, GOOD
        c, h = context()
        attach(c)
        ack = c.acknowledge(h)
        c.observe(terminal_snapshot(c))
        final_absence(c, lambda: snapshot([], 30, prior=c.last), lambda g: False)
        root = Path(tempfile.mkdtemp(prefix='b37-output-'))
        route = create_route(root, RUN_ID, c.binary)
        launch = SyntheticLaunch()
        actions = configure_launch(launch, route, root, RUN_ID, c.binary)
        launch.produce(b'', GOOD)
        observer = {'state': 'ENTRIES_COMPLETE', 'stage': 'completion', 'run_id': c.run_id,
            'pid': 123, 'target_launches': 1, 'output_actions': actions, 'exit': 0, 'exited': True,
            'final_phase': 'active', 'handover_count': 1,
            'hits': ['prepare', 'fixups', 'generic', 'libsystem', 'sanitizers', 'constructor', 'main', 'completion']}
        release = {'remaining': [], 'groups_absent': True, 'watcher_alive': False, 'monitor_alive': False}
        require(finalize(route, observer, h, ack, c.document(), release)['state'] == 'PASS',
                'actual finalization rejected terminal tracer history')
        observer['exited'] = False
        refuses(lambda: finalize(route, observer, h, ack, c.document(), release))
    check('terminal-tracer-canonical-output-requires-actual-exit', output_terminal)
