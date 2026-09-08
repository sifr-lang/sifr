"""B40 frozen initial receipt and expanded synthetic bounded acquisition cases."""
import copy
import hashlib
import json
from coverage_symbols import E, require
from coverage_acquisition import plain, validate_rounds, confirmed_departures
from coverage_custody import Custody, parse_rows, validate_ack
from coverage_b38_tests import setup, refuses
from coverage_lifecycle_tests import event


def saved():
    path = E / 'prior-b39-host/boundaries/target-0/custody.json'
    require(hashlib.sha256(path.read_bytes()).hexdigest() ==
            '45b7332529bcd8fdc99f80bfdd799e8477d65d9727e4c7dd445d002d7f77026f', 'frozen seq13 changed')
    return json.loads(path.read_text())


def coherent_saved():
    doc = saved()
    original = copy.deepcopy(doc['rejections'][0]['observation'])
    native = original['native']
    target = 49460
    first = copy.deepcopy(native['records'])
    snapshot = plain(original)
    next(r for r in snapshot['rows'] if r['pid'] == target)['pgid'] = target
    snapshot['raw'] = '\n'.join('{pid} {ppid} {pgid} {created} {stat} {command}'.format(**r)
                                for r in snapshot['rows'])
    snapshot['rows'] = parse_rows(snapshot['raw'])
    end = native['end']
    snapshot.update(begin_monotonic=end + .02, end_monotonic=end + .03)
    records = {p: e for p, e in first.items() if e['present']}
    before = {p: copy.deepcopy(e['after']) if e['present'] else None for p, e in first.items()}
    events = {p: copy.deepcopy(e['events']) for p, e in first.items() if 'registration' in e}
    native['acquisition'] = {'original': plain(original), 'records': first, 'missing': [],
        'parent_mismatches': [], 'group_mismatches': [target], 'root': doc['root_pid'],
        'expected_binary': records[str(target)]['after']['path']}
    native['rounds'] = [{'begin': end + .01, 'end': end + .04,
        'subjects': sorted(map(int, first)), 'before': before, 'after': copy.deepcopy(before),
        'events_before': events, 'events_after': copy.deepcopy(events), 'snapshot': snapshot}]
    native.update(end=end + .05, error=None, corroborations=[])
    for p, entry in native['records'].items():
        if entry['present']:
            entry.update(before=copy.deepcopy(before[p]), after=copy.deepcopy(before[p]))
    original.update(copy.deepcopy(snapshot))
    return doc, original


def initial_setup(mode='ordinary', departed=(), rounds=1, combined=True):
    c, h, p, state, backend = setup(mode, departed, rounds)
    p.expected_binary = c.binary
    next(r for r in state['rows'] if r['pid'] == 123)['pgid'] = 100
    if not combined:
        backend.samples[123]['ppid'] = 100
    original_ps = p.ps_capture
    def ps():
        if state['calls'] >= rounds:
            next(r for r in state['rows'] if r['pid'] == 123)['pgid'] = 123
        return original_ps()
    p.ps_capture = ps
    return c, h, p, state, backend


def run_tests(results):
    def check(name, action):
        error = None
        try:
            action()
        except Exception as exc:
            error = repr(exc)
        results.append({'name': 'B40-' + name, 'pass': error is None, 'error': error,
            'provenance': 'frozen original retained; appended fresh evidence explicitly SYNTHETIC, zero native launches'})

    def raw_negative():
        doc = saved()
        s = doc['rejections'][0]['observation']
        c = Custody(doc['root_pid'], doc['parent_group'], s['native']['records']['49460']['after']['path'], doc['run_id'])
        for previous in doc['observations']:
            c.observe(previous)
        before = c.document()
        c.observe(s)
        require(c.rejections and c.targets == set() and 49460 not in c.native_initial,
                'original mismatch acquired authority')
        require(c.observations == before['observations'], 'original rejected observation committed')
        require(s['native']['error'] == 'native/ps group mismatch', 'historical error rewritten')
    check('frozen-seq13-original-still-rejected', raw_negative)

    def saved_positive():
        doc, s = coherent_saved()
        c = Custody(doc['root_pid'], doc['parent_group'], s['native']['acquisition']['expected_binary'], doc['run_id'])
        for previous in doc['observations']:
            c.observe(previous)
        c.observe(s)
        require(not c.rejections, repr(c.rejections and c.rejections[0]['reason']))
        ack = c.acknowledge({'pid':49460, 'pgid':49460, 'run_id':c.run_id, 'flags':134})
        validate_ack(json.loads(json.dumps(ack)), 49460, c.run_id, c.binary)
        require(s['native']['acquisition']['original'] == plain(doc['rejections'][0]['observation'])
                and not c.transitions and c.targets == {49460}, 'frozen initial receipt changed or historical transition claimed')
    check('frozen-seq13-appended-coherent-initial-only-pass', saved_positive)

    for combined in (False, True):
        for count in (1, 2):
            def positive(combined=combined, count=count):
                c,h,p,state,b = initial_setup(rounds=count, combined=combined)
                if not combined:
                    # Keep native and every ps parent direct to LLDB.
                    base = p.ps_capture
                    def direct():
                        s = base()
                        next(r for r in s['rows'] if r['pid'] == 123)['ppid'] = 100
                        s['raw'] = '\n'.join('{pid} {ppid} {pgid} {created} {stat} {command}'.format(**r) for r in s['rows'])
                        s['rows'] = parse_rows(s['raw'])
                        return s
                    p.ps_capture = direct
                s = p(); c.observe(s)
                require(not c.rejections, repr(c.rejections and c.rejections[0]['reason']))
                require(len(s['native']['rounds']) == count and state['calls'] == count + 1, 'shared round count differs')
                validate_ack(c.acknowledge(h),123,c.run_id,c.binary)
                require(not c.transitions and c.acquired_chains[123]['initial_group'], 'initial group inferred historical transition')
            check('producer-group-parent-' + str(combined) + '-rounds-' + str(count), positive)

    for mode in ('unknown-parent','foreign-group','reuse','path','exec','lost','eof','budget',
                 'root-gone','partial','transient-native-parent','transient-ps-parent'):
        def negative(mode=mode):
            c,h,p,state,b = initial_setup(mode)
            s=p(); c.observe(s)
            require(c.rejections and not c.targets and not c.native_initial, 'invalid group acquisition committed')
            refuses(lambda:c.acknowledge(h))
            require(not c.cleanup_allowed(123,s), 'invalid group gained signal authority')
            if mode == 'partial':
                require(s['native']['rounds'][0]['before'] and s['native']['rounds'][0]['after'], 'partial samples discarded')
        check('group-negative-' + mode, negative)

    for mode in ('arbitrary-old-group', 'unknown-old-parent', 'unknown-debugger',
                 'established-registration', 'native-registration-group', 'native-bracket-group', 'no-expected-binary'):
        def negative(mode=mode):
            c,h,p,state,b = initial_setup()
            target = next(r for r in state['rows'] if r['pid']==123)
            if mode == 'arbitrary-old-group': target['pgid']=777
            if mode == 'unknown-old-parent': target['ppid']=777
            if mode == 'unknown-debugger': b.samples[100]['path']='/unknown/debugger'
            if mode == 'no-expected-binary': p.expected_binary=None
            if mode == 'native-registration-group': b.on_register=lambda pid: b.samples[123].update(pgid=777) if pid==123 else None
            if mode == 'native-bracket-group':
                sample=b.sample; calls=[0]
                def changed(pid):
                    if pid==123:
                        calls[0]+=1
                        if calls[0]==3: b.samples[123]['pgid']=777
                    return sample(pid)
                b.sample=changed
            if mode == 'established-registration':
                s=p(); require(s['native']['error'] is None, 'setup failed')
                # New capture retains old registration; cannot treat it as initial again.
                state['calls']=0
                target['pgid']=100
            s=p(); c.observe(s)
            require(c.rejections and not c.targets, 'ineligible original group accepted')
        check('initial-eligibility-' + mode, negative)

    for count in (3, 10, 60):
        def departures(count=count):
            c,h,p,state,b=initial_setup(departed=tuple(range(125,125+count)))
            s=p(); c.observe(s)
            require(not c.rejections and len(s['native']['rounds'])==1
                    and len(confirmed_departures(s))==count, 'combined acquisition did not share budget')
            validate_ack(c.acknowledge(h),123,c.run_id,c.binary)
        check('same-budget-group-parent-departures-' + str(count),departures)
    def exhaustion():
        c,h,p,state,b=initial_setup(rounds=3)
        s=p(); c.observe(s)
        require(c.rejections and len(s['native']['rounds'])==2 and state['calls']==3,
                'exhaustion lost receipt or allocated a third round')
    check('shared-two-round-exhaustion',exhaustion)
    def subject_bound():
        c,h,p,state,b=initial_setup(departed=tuple(range(125,186)))
        s=p();c.observe(s)
        require(c.rejections and state['calls']==1,'65 subjects started a round')
    check('shared-64-active-bound',subject_bound)
    for mode in ('budget','partial','return','exhausted'):
        def mixed_negative(mode=mode):
            c,h,p,state,b=initial_setup('ordinary' if mode=='exhausted' else mode,
                                      departed=(125,126,127),rounds=3 if mode=='exhausted' else 1)
            s=p();c.observe(s)
            require(c.rejections and not c.targets and not c.native_initial,'mixed failed batch committed authority')
            require(s['native']['acquisition']['missing']==[125,126,127]
                    and s['native']['acquisition']['group_mismatches']==[123]
                    and s['native']['acquisition']['parent_mismatches']==[123],
                    'mixed initial requests lost')
            require(len(s['native']['rounds'])<=2 and state['calls']<=3,'mixed batch exceeded shared rounds')
            if mode=='partial':require(s['native']['rounds'][0]['before'] and s['native']['rounds'][0]['after'], 'mixed partials lost')
            refuses(lambda:c.acknowledge(h))
        check('mixed-parent-group-departure-negative-'+mode,mixed_negative)
    def new_descendant():
        from coverage_custody_tests import row
        c,h,p,state,b=initial_setup()
        base=p.ps_capture
        def ps():
            if state['calls']==1:state['rows'].append(row(777,100,100,'/bin/ps'))
            return base()
        p.ps_capture=ps
        s=p();c.observe(s)
        require(c.rejections and 'new unobserved acquisition descendant' in s['native']['error'], 'new same-named descendant exempted')
    check('fresh-new-descendant-no-name-exemption',new_descendant)
    def accepted():
        doc,s=coherent_saved()
        c=Custody(doc['root_pid'],doc['parent_group'],s['native']['acquisition']['expected_binary'],doc['run_id'])
        c.observe(s);require(not c.rejections,'initial setup failed')
        second=copy.deepcopy(s)
        # Move every clock forward without changing the proof subjects.
        def clocks(value):
            if isinstance(value,dict):
                for k,v in value.items():
                    if k in ('begin','end','begin_monotonic','end_monotonic') and isinstance(v,(int,float)):value[k]=v+10
                    else:clocks(v)
            elif isinstance(value,list):
                for v in value:clocks(v)
        clocks(second);c.observe(second)
        require(c.rejections and c.targets=={49460},'accepted target reacquired initial group')
    check('accepted-target-initial-group-replay-denied',accepted)
    for field in ('group-list','expected-binary','fresh-group','original-group','registration',
                  'birth','exec','watch-loss','missing-round','budget','selected-ps'):
        def tamper(field=field):
            doc,s=coherent_saved();n=s['native'];a=n['acquisition']
            if field=='group-list':a['group_mismatches']=[]
            if field=='expected-binary':a['expected_binary']='/unknown'
            if field=='fresh-group':n['rounds'][0]['after']['49460']['pgid']=777
            if field=='original-group':a['original']['rows'][0]['pgid']=777
            if field=='registration':a['records']['49460']['registration']['sequence']=12
            if field=='birth':n['rounds'][0]['after']['49460']['start_usec']+=1
            if field in ('exec','watch-loss'):n['rounds'][0]['events_after']['49460'].append(event(49460,'exec' if field=='exec' else 'lost'))
            if field=='missing-round':n['rounds']=[]
            if field=='budget':n['end']+=7
            if field=='selected-ps':s['rows'][0]['ppid']=777
            c=Custody(doc['root_pid'],doc['parent_group'],doc['rejections'][0]['observation']['native']['records']['49460']['after']['path'],doc['run_id'])
            c.observe(s);require(c.rejections and not c.targets,'independent receipt consumer trusted tampered proof')
        check('independent-receipt-negative-' + field,tamper)
