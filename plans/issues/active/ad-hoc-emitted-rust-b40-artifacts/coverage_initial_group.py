"""B40 initial non-atomic group acquisition proof, never group-transition authority."""
from coverage_native import stable, live, DEBUGGER, DEBUGSERVER


def require(ok, reason):
    if not ok:
        raise ValueError(reason)


def prove_initial_groups(original, records, root, sequence, binary):
    """Recompute eligibility from raw initial evidence; producer and replay agree."""
    from coverage_custody import ancestry
    rows = {r['pid']: r for r in original['rows']}
    chains = ancestry(original['rows'], root)
    result = {}
    for key, entry in records.items():
        if not entry.get('present'):
            continue
        pid = int(key)
        row = rows[pid]
        if row['pgid'] == entry['after']['pgid']:
            continue
        require(binary is not None and pid != root and pid in chains
                and entry['after']['path'] == str(binary), 'group mismatch outside expected initial target')
        require(entry['registration']['sequence'] == sequence,
                'established native identity cannot reacquire group')
        samples = [entry['initial_sample'], entry['before'], entry['after'],
                   entry['registration']['before'], entry['registration']['after']]
        require(all(s is not None and stable(samples[0], s) for s in samples)
                and samples[0]['pid'] == samples[0]['pgid'] == pid
                and live(entry), 'initial target native group/identity contradiction')
        chain = chains[pid]
        require(len(chain) in (3, 4), 'initial target chain not bounded debugger chain')
        debugger = chain[-2]
        tracer = chain[1] if len(chain) == 4 else None
        require(records[str(debugger)]['after']['path'] == DEBUGGER
                and row['pgid'] == debugger, 'old target group is not authenticated LLDB own group')
        parents = {debugger}
        if tracer is not None:
            require(records[str(tracer)]['after']['path'] == DEBUGSERVER,
                    'unknown initial target parent')
            parents.add(tracer)
        # Existing parent acquisition may see the exact sibling debugserver in
        # native samples before it appears as this target's ps parent.
        for parent in {s['ppid'] for s in samples} - parents:
            require(parent in rows and str(parent) in records
                    and rows[parent]['ppid'] == debugger
                    and records[str(parent)]['after']['path'] == DEBUGSERVER,
                    'unknown native initial target parent')
            require(tracer is None or tracer == parent, 'multiple initial tracers')
            tracer = parent
            parents.add(parent)
        for subject in {root, debugger} | ({tracer} if tracer is not None else set()):
            e = records[str(subject)]
            require(live(e) and stable(e['before'], e['after'])
                    and stable(e['registration']['before'], e['registration']['after'])
                    and stable(e['registration']['after'], e['after']),
                    'initial group chain native identity unproved')
            require(rows[subject]['pgid'] == e['after']['pgid']
                    and rows[subject]['ppid'] == e['before']['ppid'] == e['after']['ppid'],
                    'initial group chain native/ps discrepancy')
            if subject != root:
                require(e['after']['pgid'] == subject, 'initial chain foreign group')
        require(rows[debugger]['ppid'] == root
                and (tracer is None or rows[tracer]['ppid'] == debugger),
                'initial group chain escaped root')
        result[pid] = {'debugger': debugger, 'tracer': tracer,
                       'parents': sorted(parents), 'initial_group': True}
    return result


def validate_group_chains(snapshot, root, binary):
    acquisition = snapshot.get('native', {}).get('acquisition')
    if not acquisition or not acquisition.get('group_mismatches'):
        return {}
    require(acquisition['root'] == root and acquisition.get('expected_binary') == str(binary),
            'initial group expected root/binary differs from consumer')
    initial = acquisition['records']
    result = prove_initial_groups(acquisition['original'], initial, root,
                                  snapshot['native']['sequence'], binary)
    require(sorted(result) == acquisition['group_mismatches'], 'initial group subjects differ')
    old = {r['pid']: r for r in acquisition['original']['rows']}
    for receipt in snapshot['native']['rounds']:
        rows = {r['pid']: r for r in receipt['snapshot']['rows']}
        for target, chain in result.items():
            allowed_groups = {target} if receipt is snapshot['native']['rounds'][-1] else {target, chain['debugger']}
            require(rows[target]['pgid'] in allowed_groups and rows[target]['ppid'] in chain['parents'],
                    'fresh target group or parent outside initial chain')
            require(all(receipt[side][str(target)]['ppid'] in chain['parents']
                        for side in ('before', 'after')), 'fresh native target parent outside chain')
            for subject in (root, chain['debugger'], chain['tracer']):
                if subject is None:
                    continue
                require(rows[subject]['ppid'] == old[subject]['ppid']
                        == receipt['before'][str(subject)]['ppid']
                        == receipt['after'][str(subject)]['ppid'], 'fresh initial root chain changed')
        # Any other native parent movement requires the existing parent proof.
        exempt = set(result) | set(acquisition['parent_mismatches'])
        for key, e in initial.items():
            pid = int(key)
            if e['present'] and pid not in exempt:
                require(old[pid]['ppid'] == rows[pid]['ppid'] == e['before']['ppid']
                        == e['after']['ppid'] == receipt['before'][key]['ppid']
                        == receipt['after'][key]['ppid'], 'unrelated initial parent changed')
    return result
