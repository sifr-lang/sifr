"""Offline B34 producer/consumer tests. Every process row is synthetic unless labeled."""
import copy
import hashlib
import json
import tempfile
from pathlib import Path
from types import SimpleNamespace
from coverage_custody import (Custody, capture, lifecycle, parse_rows,
                              validate_ack, validate_completion)
from coverage_symbols import E, require


def row(pid, ppid, pgid, command, stat='T', created='Tue Sep 8 07:05:45 2026'):
    return {'pid': pid, 'ppid': ppid, 'pgid': pgid, 'command': str(command),
            'stat': stat, 'created': created}


def snapshot(rows, tick=10, prior=None, **changes):
    from coverage_native_fakes import enrich
    rows = list(rows)
    if not any(r['pid'] == 99 for r in rows):
        rows.append(row(99, 1, 99, '/synthetic/launcher'))
    raw = '\n'.join('{pid} {ppid} {pgid} {created} {stat} {command}'.format(**r) for r in rows)
    times = iter((tick, tick + 0.01))
    result = capture(run=lambda *a, **k: SimpleNamespace(stdout=raw, stderr='', returncode=0),
                     monotonic=lambda: next(times), wall=lambda: 1000 + tick)
    enrich(result, prior)
    result.update(changes)
    return result


def synthetic_context(binary='/synthetic/binary', run_id=None):
    from coverage_output import RUN_ID
    run_id = RUN_ID if run_id is None else run_id
    c = Custody(99, 99, binary, run_id)
    c.observe(snapshot([row(100, 99, 100, '/synthetic/lldb'),
                        row(123, 100, 123, binary)]))
    handshake = {'pid': 123, 'pgid': 123, 'flags': 134, 'run_id': run_id}
    return c, handshake, c.acknowledge(handshake)


def run_tests(results):
    def check(name, action, error=None):
        caught = None
        try:
            action()
            passed = error is None
        except Exception as exc:
            caught = str(exc)
            passed = isinstance(exc, ValueError) and error is not None and error in caught
        results.append({'name': 'custody-' + name, 'pass': passed,
                        'error': caught, 'expected_error': error,
                        'provenance': 'SYNTHETIC process identities; no live process'})

    def producer():
        calls = []
        def run(command, **kwargs):
            calls.append((command, kwargs))
            return SimpleNamespace(stdout='123 99 123 Tue Sep 8 07:05:45 2026 TX /path with spaces',
                                   stderr='', returncode=0)
        ticks = iter((10, 10.1))
        s = capture(run=run, monotonic=lambda: next(ticks), wall=lambda: 2000)
        require(s['rows'][0]['command'] == '/path with spaces' and s['rows'][0]['stat'] == 'TX',
                'row fields lost')
        require(s['begin_monotonic'] == 10 and s['end_monotonic'] == 10.1
                and s['begin_wall'] == 2000 and s['raw'].endswith('/path with spaces'), 'clocks/raw lost')
        require(calls[0][1]['env']['LC_ALL'] == 'C' and calls[0][1]['env']['TZ'] == 'UTC'
                and calls[0][1]['timeout'] == 2, 'producer environment changed')
    check('real-parser-bracketed-producer-with-injected-ps', producer)

    for raw in ('bad row', '0 1 1 Tue Sep 8 07:05:45 2026 T /bad',
                '1 0 1 Tue Sep 8 07:05:45 2026 R /a\n1 0 1 Tue Sep 8 07:05:45 2026 R /b'):
        check('parser-reject-' + raw[:20], lambda raw=raw: parse_rows(raw),
              'duplicate' if raw.count('\n') else 'invalid' if raw.startswith('0') else 'malformed')

    def initial():
        c, h, ack = synthetic_context()
        validate_ack(ack, 123, c.run_id, c.binary)
        validate_completion(ack, json.loads(json.dumps(c.document())), 123, c.run_id, c.binary)
        ack['process']['command'] = '/external-mutation'
        require(c.first[123]['row']['command'] == c.binary, 'ack aliases initial identity')
        require(c.cleanup_allowed(123, snapshot(c.last['rows'], 11, prior=c.last)), 'unchanged owned cleanup denied')
    check('initial-ack-json-roundtrip-and-owned-cleanup', initial)

    def conflict(field, value):
        c, h, ack = synthetic_context()
        original = copy.deepcopy(c.first[123])
        changed = copy.deepcopy(c.last['rows'])
        changed[1][field] = value
        s = snapshot(changed, 11, prior=c.last)
        c.observe(s)
        require(c.rejections and c.document()['state'] == 'REJECTED', 'conflict not rejected')
        r = next(r for r in c.rejections if r['pid'] == 123)
        require(r['first'] == original and r['conflicting_row'][field] == value,
                'first/conflicting row lost')
        require(r['observation'] == s and r['comparison'][field]['first'] == original['row'][field]
                and r['comparison'][field]['current'] == value and r['ancestry'] is not None
                and r['custody_state'] == 'REJECTED', 'rejection context lost')
        require(r['lifecycle']['cause'] == 'UNKNOWN' and c.first[123] == original,
                'historical cause invented or initial replaced')
        saved = copy.deepcopy(r)
        c.observe(snapshot(changed, 12, prior=c.last))
        require(saved in c.rejections, 'first rejection timestamp overwritten')
        require(not c.cleanup_allowed(123, snapshot(changed, 13, prior=c.last)), 'changed signal identity accepted')
        c.observe(snapshot(original['observation']['rows'], 14, prior=c.last))
        require(not c.cleanup_allowed(123, snapshot(c.last['rows'], 15, prior=c.last)), 'tainted identity readopted')
        try:
            validate_completion(ack, c.document(), 123, c.run_id, c.binary)
        except ValueError:
            return
        raise ValueError('rejected custody passed output consumer')

    for field, value in (('command', '(binary)'), ('created', 'Tue Sep 8 07:06:45 2026'),
                         ('pgid', 456), ('ppid', 1), ('stat', '?')):
        check('retained-conflict-and-no-readoption-' + field,
              lambda field=field, value=value: conflict(field, value))

    for state in 'IRSTU':
        def state_change(state=state):
            c, h, ack = synthetic_context()
            rows = copy.deepcopy(c.last['rows'])
            rows[1]['stat'] = state
            c.observe(snapshot(rows, 11, prior=c.last))
            require(not c.rejections and c.first[123]['row']['stat'] == 'T', 'normal state reauthenticates')
            require(c.acknowledge(h)['current']['stat'] == state, 'current lifecycle lost')
        check('normal-state-observation-' + state, state_change)

    def zombie():
        c, h, ack = synthetic_context()
        rows = copy.deepcopy(c.last['rows'])
        rows[1]['stat'] = 'Z'
        c.observe(snapshot(rows, 11, prior=c.last))
        require(lifecycle(c.first[123]['row'], rows[1])['classification'] == 'ZOMBIE_OBSERVED',
                'zombie state not reported')
        c.acknowledge(h)
    check('zombie-is-state-not-identity-proof-or-live-ack', zombie, 'not currently authenticated')

    for first, current, expected in (
        (None, None, 'UNKNOWN_MISSING_ROW'),
        ({'command': 'a'}, {'command': 'b'}, 'UNKNOWN_PROCESS_STATE'),
        (row(1, 0, 1, 'a'), row(1, 0, 1, 'b'), 'COMMAND_CHANGED'),
        (row(1, 0, 1, 'a'), row(1, 0, 1, 'a', created='later'), 'START_FIELD_CHANGED_REPLACEMENT_SUSPECTED')):
        def diagnostic(first=first, current=current, expected=expected):
            value = lifecycle(first, current)
            require(value['classification'] == expected and value['cause'] == 'UNKNOWN',
                    'classification invented a cause')
        check('truthful-classification-' + expected, diagnostic)

    for mode in ('foreign-member', 'ancestry-cycle', 'wrong-initial-group', 'two-targets', 'ancestor-replaced'):
        def reject_topology(mode=mode):
            c, h, ack = synthetic_context()
            rows = copy.deepcopy(c.last['rows'])
            if mode == 'foreign-member':
                rows.append(row(456, 1, 123, '/foreign'))
            elif mode == 'ancestry-cycle':
                rows[0]['ppid'] = 123
            elif mode == 'wrong-initial-group':
                c = Custody(99, 99, c.binary, c.run_id)
                rows[1]['pgid'] = 99
            elif mode == 'ancestor-replaced':
                rows[0]['created'] = 'Tue Sep 8 08:00:00 2026'
            else:
                rows.append(row(124, 100, 124, c.binary))
            c.observe(snapshot(rows, 11, prior=c.last))
            require(c.rejections, 'topology accepted')
            c.acknowledge(h)
        check(mode, reject_topology, 'custody rejected')

    for mutate in ('raw-disagrees', 'error', 'wrong-command', 'nan-clock', 'reverse-clock', 'stale'):
        def invalid(mutate=mutate):
            c, h, ack = synthetic_context()
            s = snapshot(c.last['rows'], 11, prior=c.last)
            if mutate == 'raw-disagrees': s['rows'][0]['command'] = 'forged'
            if mutate == 'error': s['error'] = 'permission denied'
            if mutate == 'wrong-command': s['command'] = ['/other/ps']
            if mutate == 'nan-clock': s['end_monotonic'] = float('nan')
            if mutate == 'reverse-clock': s['end_monotonic'] = 0
            if mutate == 'stale': s = copy.deepcopy(c.last)
            require(not c.cleanup_allowed(123, s) and not c.capture_valid
                    and c.rejections[-1]['observation'] == s, 'invalid capture permits cleanup')
            c.acknowledge(h)
        check('invalid-snapshot-' + mutate, invalid, 'custody rejected')

    def missing_then_replacement():
        c, h, ack = synthetic_context()
        initial = copy.deepcopy(c.first[123])
        c.observe(snapshot([c.last['rows'][0]], 11, prior=c.last))
        require(c.first[123] == initial, 'disappearance erased original')
        rows = copy.deepcopy(initial['observation']['rows'])
        rows[1]['created'] = 'Tue Sep 8 09:00:00 2026'
        c.observe(snapshot(rows, 12, prior=c.last))
        require(c.rejections and c.first[123] == initial, 'replacement PID adopted')
    check('absent-then-replacement-keeps-original', missing_then_replacement)

    for field, value in (('created', 'Tue Sep 8 09:00:00 2026'), ('command', '/replaced/lldb'),
                         ('ppid', 1), ('pgid', 999), ('stat', '?')):
        def ancestor_cleanup(field=field, value=value):
            c, h, ack = synthetic_context()
            rows = copy.deepcopy(c.last['rows'])
            rows[0][field] = value
            require(not c.cleanup_allowed(123, snapshot(rows, 11, prior=c.last)),
                    'changed ancestor grants child signal authority')
            require(c.rejections and c.first[100]['row'][field] != value,
                    'ancestor identity overwritten')
        check('cleanup-reject-changed-ancestor-' + field, ancestor_cleanup)

    for field, value in (('state', 'REJECTED'), ('run_id', 'wrong'), ('schema', 'old'),
                         ('rejections', ['fault']), ('compared_fields', ['pid'])):
        def bad_ack(field=field, value=value):
            c, h, ack = synthetic_context()
            ack[field] = value
            validate_ack(ack, 123, c.run_id, c.binary)
        check('consumer-ack-' + field, bad_ack,
              'comparison' if field == 'compared_fields' else 'rejected/wrong-run')
    def bad_chain():
        c, h, ack = synthetic_context()
        ack['initial']['ancestry'] = [123, 1]
        validate_ack(ack, 123, c.run_id, c.binary)
    check('consumer-ack-forged-ancestry', bad_chain, 'ancestry unproved')

    def historical():
        path = E / 'inputs' / 'b32-custody.json'
        require(hashlib.sha256(path.read_bytes()).hexdigest() ==
                '890321de29542a85a2790ded26f037b1f2284e66330bcd54bbc957a5d4d298aa', 'raw changed')
        data = json.loads(path.read_text())
        stop = json.loads((E / 'inputs' / 'b32-stop.json').read_text())
        require('owned process identity changed' in stop['reasons'], 'historical rejection missing')
        require(all(lifecycle(r, None)['classification'] == 'UNKNOWN_MISSING_ROW'
                    for r in data['processes']), 'historical conflicting row invented')
    check('actual-B32-first-rows-missing-conflict-remain-UNKNOWN', historical)

    def integrated(failure=None):
        from coverage_module import Resolver
        from coverage_module_fakes import API, Module, Process, Target
        from coverage_output import RUN_ID, configure_launch, create_route, finalize
        from coverage_output_tests import GOOD, SyntheticLaunch
        from coverage_symbols import load_contract
        event = next(e for e in json.loads((E / 'inputs' / 'b32-events.json').read_text()) if e['stop_id'] == 7)
        rows = [r for r in event['modules'] if r['path'] == '/usr/lib/dyld']
        entries = load_contract()
        token = None if failure == 'module-unknown' else 'SYNTHETIC-combined'
        modules = [Module(r, entries['notifier'], token) for r in rows]
        anchor = Module(rows[0], entries['notifier'], token)
        resolver = Resolver(API, Target(modules, anchor), Process(), entries, lambda: 1, lambda r: None)
        resolver.resolve('notifier')
        root = Path(tempfile.mkdtemp(prefix='b34-combined-'))
        binary = root / 'synthetic-inferior'
        route = create_route(root, RUN_ID, binary)
        launch = SyntheticLaunch()
        actions = configure_launch(launch, route, root, RUN_ID, binary)
        c, h, ack = synthetic_context(binary)
        validate_ack(ack, 123, RUN_ID, binary)  # exact observer consumer
        launch.produce(b'', GOOD if failure != 'output-empty' else b'')
        observer = {'state': 'ENTRIES_COMPLETE', 'stage': 'completion', 'run_id': RUN_ID,
                    'pid': 123, 'target_launches': 1, 'output_actions': actions,
                    'exit': 0, 'exited': True, 'final_phase': 'active', 'handover_count': 1,
                    'hits': ['prepare', 'fixups', 'generic', 'libsystem', 'sanitizers',
                             'constructor', 'main', 'completion']}
        if failure == 'custody-conflict':
            process_rows = copy.deepcopy(c.last['rows'])
            process_rows[1]['command'] = '(synthetic-inferior)'
            c.observe(snapshot(process_rows, 11, prior=c.last))
        if failure == 'handover': observer['handover_count'] = 0
        if failure == 'missing-entry': observer['hits'].pop()
        c.observe(snapshot([], 20, prior=c.last))
        release = {'remaining': [], 'groups_absent': True, 'watcher_alive': False, 'monitor_alive': False}
        result = finalize(route, observer, h, ack, c.document(), release)
        require(result['state'] == 'PASS', 'combined completion missing')
    check('combined-module-custody-output-contract', integrated)
    for failure, error in (('module-unknown', 'distinct or unknown'),
                           ('custody-conflict', 'custody rejected at completion'),
                           ('output-empty', 'empty or truncated phase stream'),
                           ('handover', 'missing complete handover'),
                           ('missing-entry', 'missing required entry coverage')):
        check('combined-reject-' + failure, lambda failure=failure: integrated(failure), error)
