"""Expanded named offline suites; real saved failures, synthetic fresh evidence."""
import copy
import json
import subprocess
import threading
from pathlib import Path
from coverage_symbols import require
from boundaries_core import E, METRICS, run_id
from boundaries_lifecycle import Inventory, Lifecycle, absence


def historical():
    doc = json.loads((E / 'prior-b24/boundaries/target-0/final-custody.json').read_text())
    return doc


def historical_snapshot(seq):
    doc = historical()
    rows = doc['observations'] + [r['observation'] for r in doc['rejections']]
    return next(s for s in rows if s.get('native', {}).get('sequence') == seq)


def observer_correction_tests(suite):
    from coverage_custody import Custody
    from coverage_acquisition import validate_rounds
    from coverage_custody_tests import snapshot, row
    from coverage_lifecycle_tests import Backend
    from coverage_b38_tests import setup, refuses
    for seq, message in ((62, 'new unobserved acquisition descendant'), (63, 'batch absence contradiction')):
        def saved(seq=seq, message=message):
            s = historical_snapshot(seq)
            try:
                validate_rounds(s)
            except ValueError as error:
                require(message in str(error), 'actual rejection changed: ' + str(error))
            else:
                raise ValueError('historical invalid acquisition accepted')
            owner = Custody(69420, 69420, 'unused', 'historical')
            prior = owner.document()
            owner.observe(s)
            require(owner.rejections and not owner.capture_valid and owner.last is None
                    and owner.native_initial == {} and owner.groups == set(), 'failed receipt committed accepted state')
            require(owner.rejections[-1]['observation'] == s, 'failed original not preserved')
        suite.check('B39-actual-seq-' + str(seq) + '-still-rejected-and-uncommitted', saved)

    def all82():
        inv = Inventory(69420)
        inv.collect(historical())
        expected = set(map(int, json.loads((E / 'prior-b24/complete-release-record.json').read_text())['native_before']))
        require(len(expected) == 82 and inv.pids == expected, 'all82 observed candidates/probes differ')
        require(70234 in inv.pids and 70235 in inv.pids, 'rejected and current probe omitted')
        require(1 not in inv.pids, 'machine-wide inventory included')
        return inv
    suite.check('B39-all82-frozen-observed-subjects-no-machine-wide-import', all82)
    for mode in ('absent', 'still-present', 'new-descendant', 'missing-native', 'capture-failed'):
        def release(mode=mode):
            inv = all82()
            b = Backend([])
            from coverage_acquisition import plain
            s = plain(snapshot([], 100))
            if mode == 'still-present':
                b.samples[70234] = {'pid': 70234}
            if mode == 'new-descendant':
                s = snapshot([row(69420, 1, 69420, '/python'), row(88888, 69420, 88888, '/unknown')], 100)
            if mode == 'missing-native':
                b.sample = lambda pid: (_ for _ in ()).throw(ValueError('native unavailable'))
            if mode == 'capture-failed':
                s['error'] = 'injected capture failure'
            r = absence(inv, b, lambda: s, lambda g: False)
            require((r['state'] == 'PASS') == (mode == 'absent'), 'retirement result wrong')
            require(not r['signals'], 'inventory granted signal authority')
        suite.check('B39-complete-inventory-release-' + mode, release, mode in ('missing-native', 'capture-failed'))

    for mode in ('reuse', 'path', 'exec', 'lost', 'partial', 'unknown-parent'):
        def no_commit(mode=mode):
            c, h, p, state, backend = setup(mode)
            inv = Inventory(99)
            p.inventory = inv
            s = p()
            inv.collect(s)
            c.observe(s)
            require(c.rejections and not c.native_initial and not c.first and not c.groups,
                    'failed validation gained accepted custody')
            require({99, 100, 123, 124} <= inv.pids, 'partial rejected inventory lost')
            require(not c.cleanup_allowed(123, s), 'failed acquisition granted signals')
        suite.check('B39-invalid-state-commit-and-inventory-' + mode, no_commit)

    for command in ('(ps)', '/bin/ps', '(Python)'):
        def unknown(command=command):
            s = copy.deepcopy(historical_snapshot(62))
            r = next(r for r in s['native']['rounds'][0]['snapshot']['rows'] if r['pid'] == 70234)
            r['command'] = command
            snap = s['native']['rounds'][0]['snapshot']
            from coverage_custody import parse_rows
            snap['raw'] = '\n'.join('{pid} {ppid} {pgid} {created} {stat} {command}'.format(**r) for r in snap['rows'])
            snap['rows'] = parse_rows(snap['raw'])
            validate_rounds(s)
        suite.check('B39-same-name-unknown-reject-' + command, unknown, True)

    class FakeProcess:
        pid = 777
        returncode = None
        def __init__(self, *args, **kwargs):
            self.argv = args[0]
            self.calls = []
        def communicate(self, timeout):
            require(timeout > 0, 'unbounded helper wait')
            self.calls.append('reap')
            self.returncode = 0
            return '', ''
        def poll(self): return self.returncode
        def kill(self): self.calls.append('kill'); self.returncode = -9

    def serialized():
        clock = [100.0]
        life = Lifecycle(Inventory(99), 200, clock=lambda: clock[0], popen=FakeProcess)
        trace = []
        requested, finished = threading.Event(), threading.Event()
        def monitor():
            requested.set()
            with life.section('host-snapshot'):
                trace.append(('host-start', clock[0]))
                life.run(['/usr/bin/pmset'])
                trace.append(('host-reaped', clock[0]))
            finished.set()
        with life.section('native-capture-validation'):
            worker = threading.Thread(target=monitor)
            worker.start()
            require(requested.wait(1), 'monitor not scheduled')
            for part in ('initial-native', 'corroboration', 'validation'):
                trace.append((part, clock[0]))
                clock[0] += .25
                require(not finished.is_set(), 'host producer overlapped acquisition')
        worker.join(1)
        require(not worker.is_alive() and finished.is_set(), 'monitor did not join')
        require(trace[-2][0] == 'host-start' and trace[-2][1] == 100.75
                and trace[-1][0] == 'host-reaped', 'deferred monitor reused old timestamp/released before reap')
        require(life.events[-1]['state'] == 'released' and 777 in life.inventory.pids, 'helper provenance lost')
    suite.check('B39-producer-overlap-entire-acquisition-through-reap-fresh-monitor', serialized)

    for mode in ('contention', 'deadline', 'exception', 'helper-stall', 'sink-failure'):
        def bounded(mode=mode):
            clock = [0.0]
            class Lock:
                held = False
                calls = 0
                def acquire(self, timeout):
                    require(0 < timeout <= 6, 'unbounded lock wait')
                    self.calls += 1
                    if mode == 'contention' and self.calls == 1:
                        clock[0] += timeout
                        return False
                    self.held = True
                    return True
                def release(self): self.held = False
                def _is_owned(self): return self.held
            lock = Lock()
            class Stalled(FakeProcess):
                def communicate(self, timeout):
                    if self.returncode is None:
                        clock[0] += timeout
                        raise subprocess.TimeoutExpired(self.argv, timeout)
                    return '', ''
            def sink(row):
                if mode == 'sink-failure': raise ValueError('sink failure')
            life = Lifecycle(Inventory(99), 10, sink=sink, clock=lambda: clock[0], lock=lock,
                             popen=Stalled if mode == 'helper-stall' else FakeProcess)
            try:
                with life.section('acquisition'):
                    if mode == 'deadline': clock[0] = 7
                    elif mode == 'exception': raise ValueError('injected validator')
                    elif mode == 'helper-stall': life.run(['/bin/ps'])
            except (ValueError, subprocess.TimeoutExpired): pass
            else: raise ValueError('failure silently accepted')
            require(not lock.held and life.failure is not None, 'exception did not unlock/stick')
            if mode != 'sink-failure':
                with life.section('retirement', retirement=True):
                    require(absence(life.inventory, Backend([]), lambda: snapshot([], 20), lambda g: False,
                                    {99})['state'] == 'PASS', 'sticky failure blocked independent absence')
                require(life.failure is not None, 'retirement reset sticky error')
        suite.check('B39-bounded-lifecycle-' + mode, bounded)

    def prior_probes():
        life = Lifecycle(Inventory(99), 20, clock=lambda: 1, popen=FakeProcess)
        with life.section('capture'):
            life.run(['/bin/ps'])
            FakeProcess.pid = 778
            try: life.run(['/bin/ps'])
            finally: FakeProcess.pid = 777
        spawned = [e for e in life.events if e['state'] == 'spawned']
        require(spawned[1]['pid'] == 778 and 777 in spawned[1]['prior_probe_pids'], 'prior/current probe identity lost')
    suite.check('B39-exact-current-and-prior-probe-provenance', prior_probes)
    def caught_timeout_sticky():
        class Stalled(FakeProcess):
            def communicate(self, timeout):
                if self.returncode is None:
                    raise subprocess.TimeoutExpired(self.argv, timeout, output='raw partial', stderr='raw error')
                return '', ''
        life = Lifecycle(Inventory(99), 20, clock=lambda: 1, popen=Stalled)
        try:
            with life.section('host-snapshot'):
                try: life.run(['/bin/ps'])
                except subprocess.TimeoutExpired: pass  # Host policy parser catches this.
        except ValueError: pass
        else: raise ValueError('host swallowed first live helper failure')
        require(life.failure and any(e.get('partial_stdout') == 'raw partial' for e in life.events),
                'sticky producer failure/partial receipt lost')
    suite.check('B39-host-caught-helper-timeout-remains-terminal-with-raw-partial', caught_timeout_sticky)
    def actual_host_wrapper():
        import host_control
        previous, capture_fn = host_control.LIFECYCLE, host_control._capture_host_snapshot
        life = Lifecycle(Inventory(99), 100, clock=lambda: 1, popen=FakeProcess)
        try:
            host_control.LIFECYCLE = life
            host_control._capture_host_snapshot = lambda **kw: {'captured_at_unix': 123, 'kw': kw}
            result = host_control.capture_host_snapshot(include_calibration=False, control_mode='work')
            require(result['producer_begin_monotonic'] <= result['producer_end_monotonic']
                    and result['kw'] == {'include_calibration':False,'control_mode':'work'}
                    and life.events[0]['kind'] == 'host-snapshot', 'actual host wrapper bypassed lifecycle/policy')
        finally:
            host_control.LIFECYCLE, host_control._capture_host_snapshot = previous, capture_fn
    suite.check('B39-actual-host-wrapper-retains-policy-and-fresh-brackets', actual_host_wrapper)
    six_target_tests(suite)


def six_target_tests(suite):
    from boundaries_schedule import Schedule
    from coverage_lifecycle_tests import context, Backend
    from coverage_custody_tests import row, snapshot
    from coverage_custody import Custody
    from coverage_native import NativeCapture, DEBUGSERVER
    from boundaries_output import target_release, complete
    from coverage_output import create_route, configure_launch
    from coverage_output_tests import SyntheticLaunch
    from boundaries_tests import synthetic_events

    def full():
        initial, h = context()
        state = {'rows': copy.deepcopy(initial.last['rows'])}
        backend = Backend(state['rows'])
        clock = [30.0]
        def tick(): clock[0] += .025; return clock[0]
        inv = Inventory(99)
        producer = NativeCapture(99, backend, lambda: snapshot(state['rows'], tick()), tick, inv)
        schedule = Schedule(0, producer)
        for i in range(6):
            target, tracer = 123 + i * 100, 124 + i * 100
            if i:
                state['rows'] += [row(target, 100, target, initial.binary), row(tracer, 100, tracer, DEBUGSERVER)]
                backend.samples.update(Backend(state['rows']).samples)
            permit = schedule.permit(tick(), producer)
            require(permit['warmup'] == (i == 0), 'warmup schedule changed')
            owner = Custody(99, 99, initial.binary, run_id(i))
            owner.observe(producer())
            handshake = dict(h, pid=target, pgid=target, run_id=run_id(i))
            ack = owner.acknowledge(handshake)
            d = suite.root / ('six-output-' + str(i)); d.mkdir()
            route = create_route(d, run_id(i), initial.binary)
            launch = SyntheticLaunch()
            actions = configure_launch(launch, route, d, run_id(i), initial.binary)
            _, stderr = synthetic_events(); launch.produce(b'', stderr)
            observer = {'state': 'ENTRIES_COMPLETE', 'stage': 'completion', 'target_deleted': True,
                'pid': target, 'run_id': run_id(i), 'target_launches': 1, 'exit': 0, 'exited': True,
                'output_actions': actions, 'hits': ['prepare','fixups','generic','libsystem','sanitizers','constructor','main','completion'],
                'final_phase': 'active', 'handover_count': 1}
            state['rows'] = [r for r in state['rows'] if r['pid'] not in (target, tracer)]
            backend.samples.pop(target); backend.samples.pop(tracer)
            owner.observe(producer())
            release = target_release(owner, lambda g: False)
            output = complete(route, observer, handshake, ack, owner.document(), release)
            retirement = absence(inv, backend, lambda: snapshot(state['rows'], tick()), lambda g: False, {99,100})
            schedule.finish(tick(), retirement, output, dict(state='PASS',index=i,run_id=run_id(i)), inv.pids)
        require(len(schedule.receipts) == 6 and len(producer.registrations) == 14
                and len(inv.pids) == 14, 'six targets/continuous watches/cumulative inventory lost')
        try: schedule.permit(tick(), producer)
        except ValueError: return
        raise ValueError('seventh target allowed')
    suite.check('B39-full-six-target-continuous-custody-release-output-cumulative-inventory', full)
    for failure in ('custody', 'counter', 'output', 'release', 'host', 'evidence', 'deadline'):
        def stopped(failure=failure):
            token = object(); s = Schedule(0, token); s.permit(1, token)
            s.fail(failure)
            for index in range(1,6):
                try: s.permit(index + 2, token)
                except ValueError: continue
                raise ValueError('first failure launched later target')
            require(s.index == 0 and not s.receipts and s.failure == failure, 'failure reset')
        suite.check('B39-six-scheduler-first-' + failure + '-terminal', stopped)
    for mode in ('release60', 'observation510', 'full60fit', 'events257', 'bytes2GiB', 'free4GiB', 'collector'):
        def bounds(mode=mode):
            token = object(); s = Schedule(0, token)
            if mode == 'full60fit': s.permit(450.001, token); return
            if mode == 'collector': s.permit(1, object()); return
            s.permit(1, token)
            s.check(61 if mode == 'release60' else 510 if mode == 'observation510' else 2,
                    257 if mode == 'events257' else 256,
                    2 * 1024**3 if mode == 'bytes2GiB' else 0,
                    4 * 1024**3 - 1 if mode == 'free4GiB' else 4 * 1024**3)
        suite.check('B39-six-scheduler-bound-' + mode, bounds, True)


def analyzer_correction_tests(suite):
    from analyze_boundaries import replay, accounting, phase_records, intersection, total, analyze_target
    from boundaries_tests import synthetic_events
    def saved():
        events = json.loads((E / 'prior-b24/boundaries/target-0/events.json').read_text())
        pairs, entries = replay(events, complete_required=False)
        prepare = next(c for c in pairs.completed if c['label'] == 'prepare')
        require(prepare['entry_sequence'] == 6 and entries['constructor'] == [16]
                and prepare['return_sequence'] == 17, 'saved crossing ordering changed')
        require(analyze_target(E / 'prior-b24/boundaries/target-0')['state'] == 'PARTIAL', 'partial promoted')
    suite.check('B39-saved-6-16-17-ordering-and-partial-retained', saved)
    for metric in METRICS:
        for mode in ('constructor-crossing', 'main-crossing', 'nested', 'adjacent', 'zero-width'):
            def windows(metric=metric, mode=mode):
                events, stderr = synthetic_events()
                p, entries = replay(events)
                intervals = copy.deepcopy(p.completed)
                prepare = next(c for c in intervals if c['label'] == 'prepare')
                prepare['return_sequence'] = entries['main'][0] if mode != 'main-crossing' else entries['completion'][0]
                if mode == 'nested':
                    intervals.append(dict(prepare, label='generic', entry_sequence=entries['constructor'][0]))
                if mode == 'adjacent':
                    intervals.append(dict(prepare, label='libsystem', entry_sequence=entries['main'][0],return_sequence=entries['completion'][0]))
                if mode == 'zero-width':
                    intervals.append(dict(prepare, label='generic', entry_sequence=entries['constructor'][0],return_sequence=entries['constructor'][0]))
                r = accounting(events, intervals, entries, phase_records(b'',stderr), metric)
                require(sum(r['partition'].values()) == 260, 'union conservation failed')
                require(r['inclusive_overlapping_do_not_sum']['loader_inclusive'] ==
                        (240 if mode == 'main-crossing' else 120), 'inclusive family clipped/lost')
                require(r['partition']['measured_pre_first_stop_prefix'] == 10, 'prefix mislabeled')
                for key, bounds in r['windows'].items():
                    parts = r['window_intersections'][key]
                    require(sum(total(v) for v in parts.values()) == total(bounds), 'window gap/doublecount')
            suite.check('B39-every-metric-window-' + metric + '-' + mode, windows)
    for field in ('entry_sequence', 'return_sequence'):
        for bad in (-1, 10000, None):
            def endpoint(field=field,bad=bad):
                events, stderr = synthetic_events(); p, entries = replay(events)
                p.completed[0][field] = bad
                accounting(events,p.completed,entries,phase_records(b'',stderr),METRICS[0])
            suite.check('B39-invalid-endpoint-' + field + str(bad), endpoint, True)
