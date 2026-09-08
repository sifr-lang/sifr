"""Named offline suites: synthetic native identities, no live probes/targets."""
import ast
import copy
import hashlib
import json
import shutil
import tempfile
import traceback
from pathlib import Path
from boundaries_core import (E, METRICS, RETURN_FAMILIES, Pairing, elapsed,
                             full_target_fits, run_id, sdk_layout, validate_counter, preflight_count)
from coverage_symbols import require, load_contract, resolve_identity

class Suite:
    def __init__(self, name):
        self.name, self.cases = name, []
        self.root = Path(tempfile.mkdtemp(prefix='B24-' + name + '-', dir=E.parent / 'tmp'))
        sources = self.root / 'sources'
        sources.mkdir()
        self.hashes = {}
        for path in sorted(E.glob('*.py')) + sorted(E.glob('*.lldb')):
            shutil.copyfile(path, sources / path.name)
            self.hashes[path.name] = hashlib.sha256(path.read_bytes()).hexdigest()

    def check(self, name, action, reject=False):
        caught, error = False, None
        try:
            action()
        except (ValueError, KeyError, TypeError, IndexError) as exc:
            caught, error = True, repr(exc)
        except BaseException:
            error = traceback.format_exc()
        self.cases.append({'name': name, 'pass': caught == reject and (error is None or caught),
                           'expected_rejection': reject, 'error': error, 'native': 'SYNTHETIC'})

    def finish(self):
        report = {'suite': self.name, 'state': 'PASS' if all(c['pass'] for c in self.cases) else 'FAIL',
            'cases': self.cases, 'count': len(self.cases), 'source_hashes': self.hashes,
            'sources': str(self.root / 'sources'), 'native_launches': 0, 'native_probes': 0,
            'unchanged_B38_reused': 493}
        path = self.root / 'receipt.json'
        path.write_text(json.dumps(report, indent=2, sort_keys=True) + '\n')
        print(json.dumps({'state': report['state'], 'count': report['count'], 'receipt': str(path),
                          'failures': [c for c in self.cases if not c['pass']]}, indent=2))
        return 0 if report['state'] == 'PASS' else 1

def caller(pc=900, cfa=1200):
    return {'pc': pc, 'cfa': cfa, 'valid': True, 'executable': True, 'module': '/synthetic/dyld', 'uuid': 'synthetic-uuid'}

def counter(value=10, pid=123):
    fields = sdk_layout()['fields']
    values = {f: value if f in METRICS else 0 for f in fields}
    values['ri_proc_start_abstime'] = 555
    return {'pid': pid, 'status': 0, 'errno': 0, 'uuid': [1] * 16, 'values': values}

def close(pairing, seq, tid=7):
    c = pairing.stacks[tid][-1]['caller']
    return pairing.leave(seq, tid, c['pc'], c['cfa'], c['module'], c['uuid'])

def synthetic_events():
    pairing, events = Pairing(), []
    def add(label=None, kind=None, cfa=1000):
        sequence = len(events)
        event = {'sequence': sequence, 'counter': counter((sequence + 1) * 10)}
        if kind == 'entry':
            event.update(kind=kind, label=label)
            if label in RETURN_FAMILIES:
                event['call'] = copy.deepcopy(pairing.enter(label, sequence, 7, 100 + sequence, cfa, caller(900 + sequence, cfa + 100)))
        elif kind == 'return':
            call = close(pairing, sequence)
            event.update(kind=kind, label=call['label'], call=call)
        events.append(event)
    add()
    add('prepare', 'entry', 1000)
    add('fixups', 'entry', 900)
    add('generic', 'entry', 800)
    add(kind='return')
    add(kind='return')
    add(kind='return')
    add('libsystem', 'entry', 1000)
    add('sanitizers', 'entry', 900)
    add(kind='return')
    add(kind='return')
    add('constructor', 'entry')
    add('main', 'entry')
    events[-1]['counter'] = counter(140)
    add('completion', 'entry')
    events[-1]['counter'] = counter(260)
    phases = [('constructor', 130), ('main', 150), ('cli.parsed', 160), ('fmt.entry', 165),
        ('config.done', 170), ('ignore.before.read', 175), ('ignore.after.read', 180),
        ('ignore.loaded', 185), ('match.before.rules', 186), ('match.after.rules', 187),
        ('match.engine.built', 188), ('match.done', 189), ('match.before.rules', 190),
        ('match.after.rules', 191), ('match.engine.built', 192), ('match.done', 193),
        ('files.selected', 200), ('file.before.read', 201), ('file.after.read', 205),
        ('file.after.check', 220), ('file.before.read', 221), ('file.after.read', 225),
        ('file.after.check', 240), ('cli.done', 250)]
    stderr = b'format check passed\n' + b''.join(
        ('B20_PHASE {} {} {} {} {}\n'.format(n, v, v, v, v)).encode() for n, v in phases)
    return events, stderr

def release_context():
    from coverage_lifecycle_tests import context, attach, next_sample
    c, h = context()
    attach(c)
    ack = c.acknowledge(h)
    s = next_sample(c, lambda rows: rows.__setitem__(slice(None), [r for r in rows if r['pid'] in (99, 100)]))
    c.observe(s)
    require(not c.rejections, 'synthetic release custody rejected')
    return c, h, ack

def output_context():
    from coverage_output import create_route, configure_launch
    from coverage_output_tests import SyntheticLaunch
    from boundaries_output import target_release
    c, h, ack = release_context()
    d = Path(tempfile.mkdtemp(prefix='B24-output-', dir=E.parent / 'tmp'))
    route = create_route(d, c.run_id, c.binary)
    launch = SyntheticLaunch()
    actions = configure_launch(launch, route, d, c.run_id, c.binary)
    events, stderr = synthetic_events()
    launch.produce(b'', stderr)
    observer = {'state': 'ENTRIES_COMPLETE', 'stage': 'completion', 'target_deleted': True,
        'pid': 123, 'run_id': c.run_id, 'target_launches': 1, 'exit': 0, 'exited': True,
        'output_actions': actions, 'hits': ['prepare', 'fixups', 'generic', 'libsystem',
        'sanitizers', 'constructor', 'main', 'completion'], 'final_phase': 'active', 'handover_count': 1}
    return [route, observer, h, ack, c.document(), target_release(c, lambda g: False)]

def observer_tests():
    suite = Suite('observer')
    def nesting():
        p = Pairing()
        for i, label in enumerate(('prepare', 'fixups', 'generic', 'generic')):
            p.enter(label, i, 7, 100, 1000 - i * 100, caller(900, 1100 - i * 100))
        # A different thread has its own independent root and nesting.
        p.enter('libsystem', 4, 8, 200, 1000, caller(950, 1100))
        p.enter('sanitizers', 5, 8, 300, 900, caller(960, 1000))
        for seq, tid in enumerate((7, 8, 7, 8, 7, 7), 6):
            close(p, seq, tid)
        require(len(p.finish()) == 6, 'nested calls lost')
    suite.check('nested-reentrant-same-PC-multithread-pairing', nesting)
    for tid in (1, 7, 900):
        def calls(tid=tid):
            p = Pairing()
            for i, family in enumerate(sorted(RETURN_FAMILIES)):
                p.enter(family, i * 2, tid, 100, 1000, caller())
                close(p, i * 2 + 1, tid)
            require(len(p.finish()) == 5, 'invocations lost')
        suite.check('all-families-thread-' + str(tid), calls)
    for field, bad in (('valid', False), ('executable', False), ('pc', 0), ('pc', (1 << 64) - 1),
                       ('cfa', 0), ('cfa', 999), ('module', ''), ('uuid', '')):
        suite.check('bad-unwind-' + field + '-' + str(bad),
            lambda field=field, bad=bad: Pairing().enter('prepare', 0, 7, 100, 1000, dict(caller(), **{field: bad})), True)
    for field, bad in (('pc', 901), ('cfa', 1201), ('module', '/foreign'), ('uuid', 'foreign')):
        def badreturn(field=field, bad=bad):
            p = Pairing()
            p.enter('prepare', 0, 7, 100, 1000, caller())
            c = dict(caller(), **{field: bad})
            p.leave(1, 7, c['pc'], c['cfa'], c['module'], c['uuid'])
        suite.check('wrong-return-' + field, badreturn, True)
    suite.check('unmatched-return', lambda: Pairing().leave(1, 7, 900, 1200, '/synthetic/dyld', 'synthetic-uuid'), True)
    def crossing():
        p = Pairing()
        p.enter('prepare', 0, 7, 100, 1000, caller())
        p.enter('generic', 1, 7, 200, 900, caller(901, 1000))
        p.leave(2, 7, 900, 1200, '/synthetic/dyld', 'synthetic-uuid')
    suite.check('crossing-return-rejected', crossing, True)
    def missing():
        p = Pairing()
        p.enter('prepare', 0, 7, 100, 1000, caller())
        p.finish()
    suite.check('missing-return-rejected', missing, True)
    def duplicate_stack():
        p = Pairing()
        p.enter('prepare', 0, 7, 100, 1000, caller())
        p.enter('generic', 1, 7, 200, 1000, caller())
    suite.check('ambiguous-frame-rejected', duplicate_stack, True)
    suite.check('SDK-all35-fields-exact-size', lambda: require(sdk_layout()['size_bytes'] == 296, 'SDK size'))
    for label in ('constructor', 'main', 'completion', 'libsystem', 'sanitizers'):
        suite.check('prelaunch-unique-' + label, lambda label=label: preflight_count(label, 1))
        suite.check('prelaunch-ambiguous-' + label, lambda label=label: preflight_count(label, 2), True)
        suite.check('prelaunch-pending-' + label, lambda label=label: preflight_count(label, 0),
                    label in ('constructor', 'main', 'completion'))
    suite.check('counter-monotonic-success', lambda: validate_counter(counter(20), counter(10)))
    for field in METRICS:
        def backwards(field=field):
            c = counter(20)
            c['values'][field] = 9
            validate_counter(c, counter(10))
        suite.check('counter-backwards-' + field, backwards, True)
    for field, value in (('status', -1), ('errno', 1), ('pid', 0), ('pid', 124), ('uuid', [2] * 16)):
        suite.check('counter-invalid-' + field + str(value),
            lambda field=field, value=value: validate_counter(dict(counter(20), **{field: value}), counter(10)), True)
    for value in (-1, 0, (1 << 64), None, True):
        def invalid(value=value):
            c = counter()
            c['values']['ri_instructions'] = value
            validate_counter(c)
        suite.check('counter-uint-' + str(value), invalid, True)
    suite.check('counter-field-missing', lambda: validate_counter(dict(counter(), values={})), True)
    for origin in (0, 1000, 1000000000):
        suite.check('local-clock-origin-' + str(origin), lambda origin=origin: elapsed(origin + 20, origin, 60))
        suite.check('last-full-window-' + str(origin), lambda origin=origin: full_target_fits(origin + 450, origin))
        suite.check('refuse-short-window-' + str(origin), lambda origin=origin: full_target_fits(origin + 450.001, origin), True)
    for now in (60, -1, float('nan'), float('inf')):
        suite.check('elapsed-reject-' + str(now), lambda now=now: elapsed(now, 0, 60), True)
    suite.check('separate-clock-origin-rejected', lambda: elapsed(100000, 100, 60), True)
    for i in (-1, 6, True):
        suite.check('schedule-bound-' + str(i), lambda i=i: run_id(i), True)
    for label, entry in load_contract().items():
        def ambiguous(entry=entry, count=0):
            resolve_identity(entry, {}, [entry['cached_or_app_symbol']] * count)
        suite.check('missing-symbol-' + label, ambiguous, True)
        suite.check('ambiguous-identical-symbol-' + label, lambda entry=entry: ambiguous(entry, 2), True)
    from boundaries_output import complete, target_release
    suite.check('canonical-output-after-target-release-session-retained', lambda: require(complete(*output_context())['state'] == 'PASS', 'completion'))
    for field, value in (('state', 'PENDING'), ('groups_absent', False), ('run_id', 'foreign'),
                          ('target_ids', [124]), ('native_sequence', -1), ('keepers', [99, 124])):
        def bad_release(field=field, value=value):
            args = output_context()
            args[5][field] = value
            complete(*args)
        suite.check('reject-next-launch-release-' + field, bad_release, True)
    for field, value in (('exit', 1), ('run_id', 'foreign'), ('target_deleted', False), ('hits', []),
                          ('output_actions', []), ('handover_count', 0), ('target_launches', 2)):
        def bad_observer(field=field, value=value):
            args = output_context()
            args[1][field] = value
            complete(*args)
        suite.check('per-target-observer-' + field, bad_observer, True)
    def active_group():
        c, h, ack = release_context()
        require(target_release(c, lambda g: True)['state'] == 'PENDING', 'live group passed')
    suite.check('fresh-group-query-blocks-next', active_group)
    def live_target():
        from coverage_lifecycle_tests import context
        c, h = context()
        require(target_release(c, lambda g: True)['state'] == 'PENDING', 'live target passed')
    suite.check('live-target-blocks-next', live_target)
    def debugger_lost():
        from coverage_custody_tests import snapshot
        c, h, ack = release_context()
        c.observe(snapshot([], c.last['begin_monotonic'] + 1, prior=c.last))
        target_release(c, lambda g: False)
    suite.check('lost-session-blocks-next', debugger_lost, True)
    def changed_inode():
        args = output_context()
        path = Path(args[0]['streams']['stderr']['path'])
        path.rename(path.with_name('preserved.stderr'))
        path.write_text('replacement')
        path.chmod(0o600)
        complete(*args)
    suite.check('canonical-inode-replacement-rejected', changed_inode, True)
    def sequential():
        from coverage_lifecycle_tests import context, Backend
        from coverage_custody_tests import row, snapshot
        from coverage_custody import Custody
        from coverage_native import NativeCapture, DEBUGSERVER
        c, h = context()
        rows = copy.deepcopy(c.last['rows'])
        backend = Backend(rows)
        clock = [30]
        state = {'rows': rows}
        def tick():
            clock[0] += .1
            return clock[0]
        producer = NativeCapture(99, backend, lambda: snapshot(state['rows'], tick()), tick)
        previous = None
        for i in range(2):
            target = 123 + i * 100
            tracer = 124 + i * 100
            if i:
                require(previous['state'] == 'PASS', 'next before release')
                state['rows'] += [row(target, 100, target, c.binary), row(tracer, 100, tracer, DEBUGSERVER)]
                added = Backend(state['rows'])
                backend.samples.update(added.samples)
            owner = Custody(99, 99, c.binary, run_id(i))
            owner.observe(producer())
            require(not owner.rejections and owner.targets == {target}, 'new target custody rejected')
            handshake = dict(h, pid=target, pgid=target, run_id=run_id(i))
            owner.acknowledge(handshake)
            state['rows'] = [r for r in state['rows'] if r['pid'] not in (target, tracer)]
            backend.samples.pop(target)
            backend.samples.pop(tracer)
            owner.observe(producer())
            previous = target_release(owner, lambda g: False)
            require(previous['state'] == 'PASS', 'per-target absence unproved')
        require(len(producer.registrations) == 6, 'continuous root/debugger registrations not retained')
    suite.check('two-targets-continuous-native-watch-and-release-before-next', sequential)
    for name in ('boundaries_core.py', 'boundaries_output.py', 'lldb_boundaries.py', 'analyze_boundaries.py'):
        suite.check('embedded-Python39-syntax-' + name, lambda name=name: ast.parse((E / name).read_text(), feature_version=(3, 9)))
    from boundaries_b39_tests import observer_correction_tests
    observer_correction_tests(suite)
    require(len(suite.cases) == 162, 'inherited observer suite count changed')
    from boundaries_b40_tests import observer_tests as b40_observer_tests
    b40_observer_tests(suite)
    return suite.finish()

def analyzer_tests():
    from analyze_boundaries import union, subtract, total, replay, phase_records, accounting, analyze_target
    suite = Suite('analyzer')
    suite.check('nested-overlap-adjacent-union', lambda: require(union([(1, 5), (2, 4), (5, 8), (7, 9)]) == [[1, 9]], 'union double counted'))
    suite.check('disjoint-intervals-union', lambda: require(total([(1, 2), (3, 5)]) == 3, 'disjoint total'))
    suite.check('exclusive-subtraction', lambda: require(subtract([(0, 20)], [(2, 4), (3, 9), (12, 16)]) == [[0, 2], [9, 12], [16, 20]], 'exclusive intervals'))
    suite.check('empty-zero-union', lambda: require(union([(1, 1)]) == [], 'zero interval'))
    for bad in ([(-1, 1)], [(2, 1)], [(1.5, 3)]):
        suite.check('bad-range-' + repr(bad), lambda bad=bad: union(bad), True)
    def all_metrics():
        events, stderr = synthetic_events()
        p, entries = replay(events)
        phases = phase_records(b'', stderr)
        for metric in METRICS:
            r = accounting(events, p.completed, entries, phases, metric)
            require(sum(r['partition'].values()) == 260, 'accounting total')
            require(r['inclusive_overlapping_do_not_sum']['fixups_union'] == 30, 'fixups double counted')
            require(r['inclusive_overlapping_do_not_sum']['loader_exclusive_fixups'] == 20, 'loader exclusivity')
            require(r['partition']['remaining_preconstructor'] == 20, 'unassigned startup wrong')
    suite.check('full-per-process-seven-metric-partition', all_metrics)
    for field in ('sequence', 'counter', 'call'):
        def malformed(field=field):
            events, stderr = synthetic_events()
            del events[2][field]
            replay(events)
        suite.check('missing-event-field-' + field, malformed, True)
    for label in ('prepare', 'fixups', 'generic', 'libsystem', 'sanitizers', 'constructor', 'main', 'completion'):
        def missing(label=label):
            events, stderr = synthetic_events()
            next(e for e in events if e.get('label') == label).pop('kind')
            replay(events)
        suite.check('missing-entry-' + label, missing, True)
    def missing_return():
        events, stderr = synthetic_events()
        events[4]['kind'] = 'unknown'
        replay(events)
    suite.check('missing-return-partial-rejected', missing_return, True)
    for metric in METRICS:
        def nonmono(metric=metric):
            events, stderr = synthetic_events()
            events[4]['counter']['values'][metric] = 0
            replay(events)
        suite.check('nonmonotonic-raw-' + metric, nonmono, True)
    for name in ('constructor', 'main', 'cli.parsed', 'config.done', 'ignore.loaded', 'files.selected', 'cli.done', 'file.after.check'):
        def missing_phase(name=name):
            events, stderr = synthetic_events()
            phase_records(b'', b'\n'.join(line for line in stderr.split(b'\n') if not line.startswith(('B20_PHASE ' + name + ' ').encode())))
        suite.check('missing-buffered-' + name, missing_phase, True)
    for kind in ('truncated', 'third-file', 'duplicate', 'foreign-stdout', 'nonmonotonic'):
        def badphase(kind=kind):
            events, stderr = synthetic_events()
            stdout = b''
            if kind == 'truncated': stderr = stderr[:-1]
            if kind == 'third-file': stderr = stderr.replace(b'B20_PHASE cli.done', b'B20_PHASE file.after.check 245 245 245 245\nB20_PHASE cli.done')
            if kind == 'duplicate': stderr += b'B20_PHASE cli.done 251 251 251 251\n'
            if kind == 'foreign-stdout': stdout = b'foreign'
            if kind == 'nonmonotonic': stderr = stderr.replace(b'cli.parsed 160', b'cli.parsed 120')
            phase_records(stdout, stderr)
        suite.check('phase-invalid-' + kind, badphase, True)
    def partial():
        d = suite.root / 'target-0'
        d.mkdir()
        events, stderr = synthetic_events()
        (d / 'events.json').write_text(json.dumps(events[:4]))
        result = analyze_target(d)
        require(result['state'] == 'PARTIAL' and len(result['partial_events']) == 4 and result['warmup'], 'partial process omitted')
    suite.check('warmup-partial-preserved-with-all-events', partial)
    def full():
        d = suite.root / 'target-1'
        d.mkdir()
        events, stderr = synthetic_events()
        p, entries = replay(events)
        (d / 'events.json').write_text(json.dumps(events))
        (d / 'observer-result.json').write_text(json.dumps({'state': 'ENTRIES_COMPLETE', 'exit': 0, 'pid': 123, 'run_id': run_id(1), 'intervals': p.completed}))
        (d / 'inferior.stdout').write_bytes(b'')
        (d / 'inferior.stderr').write_bytes(stderr)
        report = analyze_target(d, partial=False)
        require(report['state'] == 'OBSERVED_UNDER_LLDB' and not report['warmup'], 'valid process rejected')
    suite.check('complete-buffered-crosschecked-per-process-table', full)
    from boundaries_b39_tests import analyzer_correction_tests
    analyzer_correction_tests(suite)
    return suite.finish()
