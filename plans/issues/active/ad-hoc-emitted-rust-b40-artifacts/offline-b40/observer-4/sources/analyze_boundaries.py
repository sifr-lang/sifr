"""Deterministic saved-event analysis; never launches or qualifies a target."""
import json
import sys
from pathlib import Path
from boundaries_core import E, OUT, METRICS, RETURN_FAMILIES, Pairing, read, save, validate_counter
from coverage_output import PHASE, parse_completion
from coverage_symbols import FAMILIES, require

def union(intervals):
    result = []
    for start, end in sorted(intervals):
        require(type(start) is int and type(end) is int and 0 <= start <= end, 'invalid counter interval')
        if start == end:
            continue
        if result and start <= result[-1][1]:
            result[-1][1] = max(result[-1][1], end)
        else:
            result.append([start, end])
    return result

def subtract(intervals, excluded):
    result = union(intervals)
    for left, right in union(excluded):
        next_result = []
        for start, end in result:
            if right <= start or left >= end:
                next_result.append([start, end])
            else:
                if start < left:
                    next_result.append([start, left])
                if right < end:
                    next_result.append([right, end])
        result = next_result
    return result

def total(intervals):
    return sum(b - a for a, b in union(intervals))

def intersection(intervals, window):
    return subtract(intervals, subtract(intervals, window))

def phase_records(stdout, stderr):
    parse_completion(stdout, stderr)
    records = []
    for line in stderr.splitlines()[1:]:
        m = PHASE.fullmatch(line)
        require(m is not None, 'invalid B20 record')
        values = [int(x) for x in m.groups()[1:]]
        records.append({'name': m.group(1).decode(), 'values': dict(zip(METRICS[:4], values))})
    names = [p['name'] for p in records]
    required = ['constructor', 'main', 'cli.parsed', 'fmt.entry', 'config.done',
                'ignore.before.read', 'ignore.after.read', 'ignore.loaded', 'files.selected', 'cli.done']
    require(all(names.count(n) == 1 for n in required), 'missing/ambiguous buffered phase')
    require([names.index(n) for n in required] == sorted(names.index(n) for n in required), 'buffered phase order')
    for key in METRICS[:4]:
        require(all(a['values'][key] <= b['values'][key] for a, b in zip(records, records[1:])),
                'nonmonotonic buffered phase counter')
    require(sum(n == 'match.done' for n in names) == 2 and names.count('file.after.check') == 2,
            'selected workload differs from exact two-file contract')
    return records

def replay(events, complete_required=True):
    pairs, previous, entries = Pairing(), None, {}
    for i, event in enumerate(events):
        require(event['sequence'] == i, 'missing/repeated event sequence')
        validate_counter(event['counter'], previous)
        previous = event['counter']
        if event.get('kind') == 'entry':
            label = event['label']
            require(label in FAMILIES, 'unknown family')
            entries.setdefault(label, []).append(i)
            if label in RETURN_FAMILIES:
                call = event['call']
                actual = pairs.enter(label, i, call['tid'], call['entry_pc'], call['entry_cfa'], call['caller'])
                require(actual == call, 'entry pairing receipt mismatch')
        elif event.get('kind') == 'return':
            call = event['call']
            caller = call['caller']
            actual = pairs.leave(i, call['tid'], caller['pc'], caller['cfa'], caller['module'], caller['uuid'])
            require(actual == call and actual['label'] == event['label'], 'return pairing receipt mismatch')
    if complete_required:
        pairs.finish()
        require(set(entries) == FAMILIES, 'missing complete entry coverage')
        for label in ('prepare', 'constructor', 'main', 'completion'):
            require(len(entries[label]) == 1, 'ambiguous phase entry')
        require(entries['prepare'][0] < entries['constructor'][0] < entries['main'][0] < entries['completion'][0],
                'entry phase order')
    return pairs, entries

def accounting(events, intervals, entries, phases, metric):
    def count(i):
        require(type(i) is int and 0 <= i < len(events), 'missing/invalid endpoint')
        return events[i]['counter']['values'][metric]
    def boundary(label):
        require(len(entries[label]) == 1, 'missing/ambiguous boundary')
        return count(entries[label][0])
    families = {label: union([(count(c['entry_sequence']), count(c['return_sequence']))
                             for c in intervals if c['label'] == label]) for label in RETURN_FAMILIES}
    pre, ctor, main, end = [boundary(n) for n in ('prepare', 'constructor', 'main', 'completion')]
    fixups = union(families['fixups'] + families['generic'])
    libs = union(families['libsystem'] + families['sanitizers'])
    loader = families['prepare']
    first = count(0)
    require(0 <= first <= pre <= ctor <= main <= end, 'backwards phase boundaries')
    require(all(first <= a <= b <= end for a, b in union(fixups + libs + loader)), 'family interval outside measured stops')
    info = {'loader_inclusive': total(loader), 'loader_exclusive_fixups': total(subtract(loader, fixups)),
            'jit_fixups_inclusive': total(families['fixups']), 'generic_fixups_inclusive': total(families['generic']),
            'fixups_union': total(fixups), 'library_union': total(libs),
            'libsystem_inclusive': total(families['libsystem']), 'sanitizers_inclusive': total(families['sanitizers'])}
    observed = union(loader + fixups + libs)
    windows = {'preconstructor': [[first, ctor]], 'constructor_to_main': [[ctor, main]],
               'main_to_completion': [[main, end]]}
    ranges = {'measured_pre_first_stop_prefix': [[0, first]],
        'pre_prepare': subtract([[first, pre]], observed), 'fixups': fixups,
        'library_initialization': subtract(libs, fixups),
        'loader_other': subtract(loader, fixups + libs),
        'remaining_preconstructor': subtract([[pre, ctor]], loader + fixups + libs),
        'constructor_to_main_unassigned': subtract([[ctor, main]], observed)}
    if metric in METRICS[:4]:
        def phase(name):
            values = [p['values'][metric] for p in phases if p['name'] == name]
            require(len(values) == 1, 'missing/ambiguous buffered endpoint')
            return values[0]
        require(ctor <= phase('constructor') <= main <= phase('main') <= phase('cli.parsed'), 'buffered/debugger startup crosscheck')
        require(phase('cli.done') <= end, 'buffered completion past debugger boundary')
        ranges.update(cli_parse=[[main, phase('cli.parsed')]],
            config=[[phase('cli.parsed'), phase('config.done')]],
            discovery=[[phase('config.done'), phase('files.selected')]])
        files = [p for p in phases if p['name'].startswith('file.')]
        checks = []
        for n in range(2):
            a, b, c = [p['values'][metric] for p in files[n * 3:n * 3 + 3]]
            require(phase('files.selected') <= a <= b <= c <= phase('cli.done'), 'file phases outside selected/completion')
            ranges['file_read_' + str(n + 1)] = [[a, b]]
            ranges['file_check_' + str(n + 1)] = [[b, c]]
            checks.append([a, c])
        ranges['completion_residual'] = subtract([[phase('files.selected'), end]], checks)
    else:
        # B20 does not buffer pageins/I/O counters. Do not invent subphase values.
        ranges['main_to_completion_unassigned'] = [[main, end]]
    for key in ('cli_parse', 'config', 'discovery', 'file_read_1', 'file_read_2',
                'file_check_1', 'file_check_2', 'completion_residual', 'main_to_completion_unassigned'):
        if key in ranges:
            ranges[key] = subtract(ranges[key], observed)
    flat = [r for values in ranges.values() for r in values]
    require(total(flat) == end and sum(total(v) for v in ranges.values()) == end, 'partition gap/double count')
    return {'inclusive_overlapping_do_not_sum': info, 'partition': {k: total(v) for k, v in ranges.items()},
            'partition_ranges': ranges, 'measured_endpoint': end,
            'window_intersections': {w: {k: intersection(v, bounds) for k, v in ranges.items()
                 if k != 'measured_pre_first_stop_prefix'} for w, bounds in windows.items()},
            'windows': windows,
            'prefix_kind': 'cumulative measured counter before first stop; not a directly observed interval',
            'initial_stop': first, 'after_completion_to_exit': None}

def analyze_target(directory, partial=True):
    directory = Path(directory)
    events = read(directory / 'events.json') or []
    observer = read(directory / 'observer-result.json') or {}
    result = {'index': int(directory.name.split('-')[-1]), 'warmup': directory.name == 'target-0',
        'pid': observer.get('pid'), 'run_id': observer.get('run_id'), 'state': 'PARTIAL',
        'event_count': len(events), 'limits': ['process-wide counters across threads; not per-thread work',
            'no debugger-overhead subtraction', 'post-completion exit tail unmeasured',
            'raw SDK time units are not Python wall seconds; no conversion',
            'localization under LLDB does not establish cause or production CV']}
    try:
        pairs, entries = replay(events)
        result['intervals'] = pairs.completed
        require(observer.get('state') == 'ENTRIES_COMPLETE' and observer.get('exit') == 0, 'observer incomplete')
        require(observer['intervals'] == pairs.completed, 'observer/analyzer interval mismatch')
        stdout, stderr = (directory / 'inferior.stdout').read_bytes(), (directory / 'inferior.stderr').read_bytes()
        phases = phase_records(stdout, stderr)
        result['phases'] = phases
        result['metrics'] = {metric: accounting(events, pairs.completed, entries, phases, metric) for metric in METRICS}
        result['state'] = 'OBSERVED_UNDER_LLDB'
    except (ValueError, KeyError, TypeError, IndexError, OSError) as error:
        result['error'] = str(error)
        result['partial_events'] = events
        result['partial_completed_intervals'] = observer.get('completed_intervals', [])
        if not partial:
            raise
    return result

def analyze():
    require(OUT.exists(), 'no saved run')
    rows = [analyze_target(p) for p in sorted(OUT.glob('target-*')) if p.is_dir()]
    report = {'item': '12K-B24', 'outcome': read(OUT / 'outcome.json'), 'processes': rows,
        'interpretation': 'INCONCLUSIVE for causation and production qualification; retained LLDB localization only.',
        'next_scope': 'Coordinator adjudication from concrete result; no production/OS/loader intervention authorized.'}
    observed = [r for r in rows if r['state'] == 'OBSERVED_UNDER_LLDB']
    ranges = {}
    if observed:
        for key in observed[0]['metrics']['ri_instructions']['partition']:
            values = [r['metrics']['ri_instructions']['partition'][key] for r in observed]
            ranges[key] = {'min': min(values), 'max': max(values), 'range': max(values) - min(values), 'all_processes': values}
    report['instruction_ranges_including_warmup'] = ranges
    save(E / 'analysis.json', report)
    lines = ['# B24 changed-observer diagnostic result', '', report['interpretation'], '',
             '| Process | Role | PID | State | Events |', '| --- | --- | --- | --- | --- |']
    for r in rows:
        lines.append('| {} | {} | {} | {} | {} |'.format(r['index'], 'warmup' if r['warmup'] else 'observation', r['pid'], r['state'], r['event_count']))
        if r.get('error'):
            lines.extend(['', 'Process{} partial: {}'.format(r['index'], r['error'])])
    if observed:
        lines.extend(['', 'Instruction partitions; inclusive overlapping loader/fixup/library details are in JSON.', '',
                      '| Partition | ' + ' | '.join('P' + str(r['index']) for r in observed) + ' | Range |',
                      '| --- | ' + ' | '.join('---' for r in observed) + ' | --- |'])
        for key, values in ranges.items():
            lines.append('| ' + key + ' | ' + ' | '.join(str(v) for v in values['all_processes']) + ' | ' + str(values['range']) + ' |')
    lines.extend(['', 'No debugger overhead is subtracted. Intervals are process-wide, including other threads.',
                  'Unknown work and the post-completion exit tail remain unassigned. Stability under LLDB',
                  'does not establish an uninstrumented fix or CV pass. B24/B27/full phase remain OPEN.'])
    (E / 'analysis.md').write_text('\n'.join(lines) + '\n')
    print(json.dumps({'processes': len(rows), 'observed': len(observed), 'partial': len(rows) - len(observed),
                      'instruction_ranges': ranges}, indent=2))
    return 0

if __name__ == '__main__':
    if sys.argv[1:] == ['--self-test']:
        from boundaries_tests import analyzer_tests
        raise SystemExit(analyzer_tests())
    require(not sys.argv[1:], 'unregistered analyzer arguments')
    raise SystemExit(analyze())
