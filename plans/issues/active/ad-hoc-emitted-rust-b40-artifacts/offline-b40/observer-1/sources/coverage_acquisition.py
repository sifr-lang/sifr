"""B38 shared bounded acquisition; original observations never grant authority."""
import copy
from coverage_native import stable, live, DEBUGGER, DEBUGSERVER


def require(ok, reason):
    if not ok:
        raise ValueError(reason)


def plain(snapshot):
    return copy.deepcopy({k: v for k, v in snapshot.items() if k != 'native'})


def event_history(capture):
    return {str(pid): copy.deepcopy(events) for pid, events in capture.events.items()}


def batch(capture, original, native, subjects):
    from coverage_custody import validate_snapshot
    rounds = native.setdefault('rounds', [])
    require(len(rounds) < 2, 'acquisition corroboration count exhausted')
    require(capture.monotonic() - native['begin'] < 6, 'acquisition budget exhausted')
    receipt = {'begin': capture.monotonic(), 'subjects': sorted(subjects),
               'before': {}, 'after': {}, 'events_before': event_history(capture)}
    rounds.append(receipt)  # partial records survive any subsequent exception
    for pid in sorted(subjects):
        receipt['before'][str(pid)] = capture.backend.sample(pid)
    receipt['snapshot'] = plain(capture.ps_capture())
    if getattr(capture, 'inventory', None) is not None:
        capture.inventory.collect(receipt['snapshot'])
    for pid in sorted(subjects):
        receipt['after'][str(pid)] = capture.backend.sample(pid)
    capture.poll()
    receipt.update(end=capture.monotonic(), events_after=event_history(capture))
    validate_snapshot(receipt['snapshot'])
    previous = rounds[-2]['snapshot'] if len(rounds) > 1 else original
    require(previous['end_monotonic'] < receipt['snapshot']['begin_monotonic']
            and receipt['end'] - native['begin'] <= 6, 'acquisition budget/order failed')
    return receipt


def reconcile(capture, snapshot, missing):
    native = snapshot['native']
    records = native['records']
    rows = {r['pid']: r for r in snapshot['rows']}
    from coverage_initial_group import prove_initial_groups
    groups = prove_initial_groups(snapshot, records, capture.root, native['sequence'],
                                  getattr(capture, 'expected_binary', None))
    mismatches = []
    for key, entry in records.items():
        if entry['present']:
            pid = int(key)
            if rows[pid]['ppid'] not in (entry['before']['ppid'], entry['after']['ppid']):
                mismatches.append(pid)
    if not missing and not mismatches and not groups:
        return
    original = plain(snapshot)
    native['acquisition'] = {'original': original, 'records': copy.deepcopy(records),
        'missing': list(missing), 'parent_mismatches': mismatches, 'root': capture.root}
    if groups:
        native['acquisition'].update(group_mismatches=sorted(groups),
                                    expected_binary=str(capture.expected_binary))
    subjects = {int(pid) for pid in records}
    require(len(subjects) <= 64, 'native custody subject bound reached')
    for _ in range(2):
        receipt = batch(capture, original, native, subjects)
        fresh = {r['pid']: r for r in receipt['snapshot']['rows']}
        coherent = True
        for pid in sorted(subjects):
            key = str(pid)
            a, b = receipt['before'][key], receipt['after'][key]
            initial = native['acquisition']['records'][key]
            if not initial['present']:
                require(a is None and b is None and pid not in fresh,
                        'ps/native disappearance ambiguity: fresh corroboration contradicted')
                continue
            require(a is not None and b is not None and pid in fresh,
                    'live subject disappeared during reconciliation')
            require(stable(initial['after'], a) and stable(a, b),
                    'reconciliation native identity changed')
            require(fresh[pid]['pgid'] == b['pgid']
                    or pid in groups and fresh[pid]['pgid'] == rows[pid]['pgid'],
                    'reconciliation foreign group')
            require(not any(e['exec'] or e['lost'] or (e['eof'] and not e['exit'])
                            for e in capture.events[pid]), 'reconciliation exec/watch/exit event')
            coherent &= (fresh[pid]['ppid'] == a['ppid'] == b['ppid']
                         and fresh[pid]['pgid'] == b['pgid'])
        if coherent:
            break
    else:
        raise ValueError('acquisition corroboration count exhausted')
    native['corroborations'] = []
    for pid in missing:
        key = str(pid)
        native['corroborations'].append({'pid': pid, 'initial_sample': None,
            'begin': receipt['begin'], 'end': receipt['end'],
            'before': receipt['before'][key], 'after': receipt['after'][key],
            'snapshot': copy.deepcopy(receipt['snapshot']), 'round': len(native['rounds']) - 1})
        records[key].update(present=False, absence='ESRCH', acquisition_departure=True)
        capture.departed.add(pid)
    if mismatches or groups:
        snapshot.update(copy.deepcopy(receipt['snapshot']))
        for key, entry in records.items():
            if entry['present']:
                entry.update(before=receipt['before'][key], after=receipt['after'][key],
                             events=copy.deepcopy(capture.events[int(key)]))


def validate_rounds(snapshot):
    from coverage_custody import validate_snapshot, ancestry, probe_row
    native = snapshot.get('native', {})
    rounds = native.get('rounds', [])
    acquisition = native.get('acquisition')
    if acquisition is None:
        require(not rounds and not native.get('corroborations'), 'missing acquisition receipt')
        return
    require(isinstance(rounds, list) and 1 <= len(rounds) <= 2, 'invalid corroboration count')
    original, initial = acquisition['original'], acquisition['records']
    validate_snapshot(original)
    subjects = sorted(int(pid) for pid in initial)
    root = acquisition['root']
    require(0 < len(subjects) <= 64 and root in subjects, 'invalid acquisition subjects')
    require(initial[str(root)]['present'] and native['end'] - native['begin'] <= 6
            and native['begin'] <= original['begin_monotonic'] <= original['end_monotonic']
            <= native['end'], 'acquisition root/total budget/clock invalid')
    require(set(initial) == set(native['records']), 'acquisition lost subjects')
    require(all(e['initial_sample'] == (e['before'] if e['present'] else None)
                and native['records'][key]['initial_sample'] == e['initial_sample']
                for key, e in initial.items()), 'initial native sample receipt differs')
    rows = {r['pid']: r for r in original['rows']}
    missing = sorted(int(p) for p, e in initial.items() if not e['present'] and int(p) in rows)
    mismatches = sorted(int(p) for p, e in initial.items() if e['present']
                        and rows[int(p)]['ppid'] not in (e['before']['ppid'], e['after']['ppid']))
    from coverage_initial_group import prove_initial_groups
    groups = prove_initial_groups(original, initial, root, native['sequence'],
                                  acquisition.get('expected_binary'))
    require(sorted(groups) == acquisition.get('group_mismatches', []),
            'acquisition group request differs from observations')
    require(sorted(acquisition['missing']) == missing and root not in missing
            and sorted(acquisition['parent_mismatches']) == mismatches,
            'acquisition request differs from observations')
    previous = original
    previous_events = {p: e.get('events', []) for p, e in initial.items() if 'registration' in e}
    for receipt in rounds:
        s = receipt['snapshot']
        validate_snapshot(s)
        require(receipt['subjects'] == subjects and set(receipt['before']) == set(initial)
                and set(receipt['after']) == set(initial), 'batch subject coverage differs')
        require(native['begin'] <= receipt['begin'] <= s['begin_monotonic']
                <= s['end_monotonic'] <= receipt['end'] <= native['end']
                and receipt['end'] - native['begin'] <= 6
                and previous['end_monotonic'] < s['begin_monotonic'], 'batch clocks/budget invalid')
        fresh = {r['pid']: r for r in s['rows']}
        require(all(p in subjects or probe_row(s, fresh[p]) for p in ancestry(s['rows'], root)),
                'new unobserved acquisition descendant')
        for key, first in initial.items():
            pid = int(key)
            a, b = receipt['before'][key], receipt['after'][key]
            if 'registration' in first:
                require(first['registration'] == native['records'][key]['registration'],
                        'batch registration replaced')
                for field in ('events_before', 'events_after'):
                    events = receipt[field].get(key, receipt[field].get(pid))
                    require(isinstance(events, list) and events[:len(previous_events[key])] == previous_events[key]
                            and all(e['pid'] == pid and not (e['exec'] or e['lost']
                                        or e['eof'] and not e['exit']) for e in events),
                            'batch watch history lost/invalid')
                    previous_events[key] = events
            if not first['present']:
                require(a is None and b is None and pid not in fresh, 'batch absence contradiction')
                continue
            require(a is not None and b is not None and pid in fresh
                    and stable(first['before'], first['after'])
                    and stable(first['after'], a) and stable(a, b)
                    and stable(first['registration']['after'], b), 'batch identity contradiction')
            require(first['registration'] == native['records'][key]['registration'],
                    'batch registration replaced')
            require((pid in groups or rows[pid]['pgid'] == fresh[pid]['pgid'])
                    and (fresh[pid]['pgid'] == b['pgid'] or pid in groups
                         and receipt is not rounds[-1] and fresh[pid]['pgid'] == rows[pid]['pgid'])
                    and rows[pid]['created'] == fresh[pid]['created'], 'batch ps identity changed')
            if receipt is rounds[-1]:
                require(fresh[pid]['ppid'] == a['ppid'] == b['ppid'], 'fresh parent samples incoherent')
        previous = s
    require(all(native['records'][key]['events'] == events for key, events in previous_events.items()),
            'final batch watch history differs')
    if mismatches or groups:
        require(plain(snapshot) == rounds[-1]['snapshot'], 'selected coherent snapshot differs')
        for key, entry in native['records'].items():
            if entry['present']:
                require(entry['before'] == rounds[-1]['before'][key]
                        and entry['after'] == rounds[-1]['after'][key]
                        and entry['events'] == previous_events[key], 'selected native evidence differs')
    else:
        require(plain(snapshot) == original, 'original disappearance snapshot changed')
        fresh = {r['pid']: r for r in rounds[-1]['snapshot']['rows']}
        require(all(not e['present'] or rows[int(key)]['ppid'] == fresh[int(key)]['ppid']
                    for key, e in initial.items()), 'parent changed during disappearance batch')


def confirmed_departures(snapshot):
    validate_rounds(snapshot)
    native = snapshot.get('native', {})
    attempts = native.get('corroborations', [])
    require(isinstance(attempts, list) and len(attempts) <= 64, 'invalid corroboration count')
    departed = set()
    original = native.get('acquisition', {}).get('original', snapshot)
    for attempt in attempts:
        pid = attempt['pid']
        require(type(pid) is int and pid > 0 and pid not in departed, 'invalid corroboration subject')
        receipt = native['rounds'][attempt['round']]
        require(attempt['initial_sample'] is None and attempt['before'] is None
                and attempt['after'] is None and attempt['snapshot'] == receipt['snapshot']
                and attempt['begin'] == receipt['begin'] and attempt['end'] == receipt['end']
                and receipt['before'][str(pid)] is None and receipt['after'][str(pid)] is None,
                'corroboration native/round contradiction')
        require(any(r['pid'] == pid for r in original['rows'])
                and not any(r['pid'] == pid for r in attempt['snapshot']['rows']),
                'corroboration ps contradiction')
        entry = native['records'].get(str(pid), {})
        require(entry.get('present') is False and entry.get('absence') == 'ESRCH'
                and entry.get('acquisition_departure') is True, 'corroboration record mismatch')
        departed.add(pid)
    require(all(not e.get('acquisition_departure') or int(pid) in departed
                for pid, e in native.get('records', {}).items()), 'missing corroboration')
    require(departed == set(native.get('acquisition', {}).get('missing', [])),
            'incomplete departure batch')
    return departed


def initial_chain(snapshot, root, binary):
    acquisition = snapshot.get('native', {}).get('acquisition')
    from coverage_initial_group import validate_group_chains
    groups = validate_group_chains(snapshot, root, binary)
    if not acquisition or not acquisition['parent_mismatches']:
        return groups
    require(acquisition['root'] == root, 'acquisition root differs from custodian')
    records = snapshot['native']['records']
    old = {r['pid']: r for r in acquisition['original']['rows']}
    rows = {r['pid']: r for r in snapshot['rows']}
    result = {}
    for pid in acquisition['parent_mismatches']:
        e = records[str(pid)]
        require(e['after']['path'] == binary and live(e), 'parent disagreement is not live target')
        parents = {old[pid]['ppid'], rows[pid]['ppid'],
                   acquisition['records'][str(pid)]['before']['ppid'],
                   acquisition['records'][str(pid)]['after']['ppid']}
        require(len(parents) == 2 and all(str(p) in records and p in rows for p in parents),
                'unknown acquisition parents')
        debugger = next((p for p in parents if records[str(p)]['after']['path'] == DEBUGGER), None)
        tracer = next((p for p in parents if records[str(p)]['after']['path'] == DEBUGSERVER), None)
        require(debugger is not None and tracer is not None
                and rows[debugger]['ppid'] == root and rows[tracer]['ppid'] == debugger
                and old[debugger]['ppid'] == root and old[tracer]['ppid'] == debugger
                and all(live(records[str(p)]) for p in (root, debugger, tracer)),
                'acquisition debugger/tracer/root chain unproved')
        require(all(records[str(p)]['after']['pgid'] == p for p in (pid, debugger, tracer)),
                'acquisition foreign group')
        result[pid] = {'debugger': debugger, 'tracer': tracer}
        for receipt in snapshot['native']['rounds']:
            observed = next(r for r in receipt['snapshot']['rows'] if r['pid'] == pid)
            require(observed['ppid'] in parents
                    and receipt['before'][str(pid)]['ppid'] in parents
                    and receipt['after'][str(pid)]['ppid'] in parents,
                    'intermediate acquisition parent outside original chain')
    for key, entry in acquisition['records'].items():
        pid = int(key)
        if entry['present'] and pid not in result:
            require(old[pid]['ppid'] == rows[pid]['ppid'] == entry['before']['ppid']
                    == entry['after']['ppid'], 'unrelated acquisition parent changed')
            for receipt in snapshot['native']['rounds']:
                observed = next(r for r in receipt['snapshot']['rows'] if r['pid'] == pid)
                require(observed['ppid'] == old[pid]['ppid']
                        == receipt['before'][key]['ppid'] == receipt['after'][key]['ppid'],
                        'intermediate unrelated acquisition parent changed')
    for pid, chain in groups.items():
        if pid in result:
            require(result[pid]['debugger'] == chain['debugger']
                    and result[pid]['tracer'] == chain['tracer'],
                    'combined initial parent/group proofs differ')
        result[pid] = chain
    return result
