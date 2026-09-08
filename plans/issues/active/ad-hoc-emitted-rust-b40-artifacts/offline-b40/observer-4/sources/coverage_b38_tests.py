"""B38 full offline producer/consumer qualification; never start native processes."""
import copy
import hashlib
import json
import tempfile
from pathlib import Path
from coverage_symbols import E, require
from coverage_native import NativeCapture, DEBUGSERVER
from coverage_custody import Custody, validate_ack, validate_completion, replay_document, parse_rows
from coverage_acquisition import plain, confirmed_departures, validate_rounds, batch
from coverage_cleanup import final_absence
from coverage_lifecycle_tests import context, Backend, event
from coverage_custody_tests import row, snapshot
from coverage_b37_tests import terminal_snapshot


def refuses(action):
    try:
        action()
    except (ValueError, KeyError, TypeError, IndexError):
        return
    raise ValueError('invalid acquisition accepted')


def setup(mode='ordinary', departed=(), rounds=1):
    c, h = context()
    rows = copy.deepcopy(c.last['rows'])
    for pid in departed:
        if not any(r['pid'] == pid for r in rows):
            rows.append(row(pid, 99, 99, '/usr/bin/pmset'))
    backend = Backend(rows)
    backend.samples[123]['ppid'] = 124
    clock = [20.0]
    state = {'calls': 0, 'rows': rows}
    def tick():
        clock[0] += .025
        return clock[0]
    def ps():
        state['calls'] += 1
        count = state['calls']
        current = copy.deepcopy(state['rows'])
        if count > rounds:
            current = [r for r in current if r['pid'] not in departed]
            target = next((r for r in current if r['pid'] == 123), None)
            if target is not None: target['ppid'] = 124
        if count > 1:
            if mode == 'transient-native-parent' and count == 2:
                backend.samples[123]['ppid'] = 777
                backend.on_poll = lambda: backend.samples[123].update(ppid=124)
            if mode == 'transient-ps-parent' and count == 2:
                next(r for r in current if r['pid'] == 123)['ppid'] = 777
            if mode == 'unknown-parent':
                next(r for r in current if r['pid'] == 123)['ppid'] = 777
                backend.samples[123]['ppid'] = 777
            if mode == 'foreign-group':
                next(r for r in current if r['pid'] == 123)['pgid'] = 777
                backend.samples[123]['pgid'] = 777
            if mode == 'reuse': backend.samples[123]['start_usec'] += 1
            if mode == 'path': backend.samples[123]['path'] = '/replacement'
            if mode in ('exec', 'lost', 'eof'): backend.pending.append(event(123, mode))
            if mode == 'budget': clock[0] += 7
            if mode == 'root-gone': backend.samples.pop(99, None)
            if mode == 'return': backend.samples[departed[-1]] = Backend(rows).samples[departed[-1]]
            if mode == 'partial':
                sample = backend.sample
                def partial(pid):
                    if pid == 124: raise ValueError('injected after-sample failure')
                    return sample(pid)
                backend.sample = partial
        s = snapshot(current, tick())
        tick()
        return s
    for pid in departed:
        backend.samples.pop(pid, None)
    p = NativeCapture(99, backend, ps, tick)
    owned = Custody(99, 99, c.binary, c.run_id)
    return owned, h, p, state, backend


def finish(c, h, p, state, backend):
    ack = c.acknowledge(h)
    validate_ack(json.loads(json.dumps(ack)), 123, c.run_id, c.binary)
    require(not c.transitions and c.acquired_chains, 'invented initial historical transition')
    # The coherent current chain supports live cleanup before any departure.
    state['rows'] = copy.deepcopy(c.last['rows'])
    require(c.cleanup_allowed(123, p()), 'coherent target cleanup denied')
    c.observe(terminal_snapshot(c))
    require(not c.rejections, 'initial acquisition terminal tracer departure rejected')
    refuses(lambda: c.acknowledge(h))
    require(not c.cleanup_allowed(123, terminal_snapshot(c)), 'terminal target signaled')
    remaining, absent = final_absence(c, lambda: snapshot([], c.last['begin_monotonic'] + 1,
                                     prior=c.last), lambda g: False)
    require(absent and not remaining and not c.rejections, 'coherent acquisition release failed')
    validate_completion(ack, json.loads(json.dumps(c.document())), 123, c.run_id, c.binary)
    from coverage_output import RUN_ID, create_route, configure_launch, finalize
    from coverage_output_tests import SyntheticLaunch, GOOD
    root = Path(tempfile.mkdtemp(prefix='b38-output-'))
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
            'canonical output rejected coherent acquisition')
    observer['exit'] = 1
    refuses(lambda: finalize(route, observer, h, ack, c.document(), release))


def historical(sequence):
    path = E / 'prior-b37-cont/failure-evidence.json'
    require(hashlib.sha256(path.read_bytes()).hexdigest() ==
            '223c1e01b8aa0952e2b3dcd31138ccd7b3a93a0f40cb5030620b76e71c3e8d98', 'raw B37 changed')
    data = json.loads(path.read_text())
    original = (data['first_rejection_complete']['observation'] if sequence == 16
                else data['cleanup_sequence26_complete'])
    seed = data['first_rejection_complete']['observation']['native']['records']
    backend = Backend([])
    backend.samples = {int(p): copy.deepcopy(e['after']) for p, e in seed.items()}
    clock = [original['begin_monotonic'] - .002]
    calls = [0]
    gone = {36554, 36592, 36593, 36633}
    def tick():
        clock[0] += .001
        return clock[0]
    def ps():
        calls[0] += 1
        if calls[0] == 1:
            s = plain(original)
            clock[0] = s['end_monotonic'] + .001
            return s
        s = plain(original)
        # New rounds are explicitly synthetic additions; raw initial evidence is intact.
        rows = copy.deepcopy(s['rows'])
        if sequence == 16:
            next(r for r in rows if r['pid'] == 36592)['ppid'] = 36593
        else:
            rows = [r for r in rows if r['pid'] not in gone]
        s['raw'] = '\n'.join('{pid} {ppid} {pgid} {created} {stat} {command}'.format(**r) for r in rows)
        s['rows'] = parse_rows(s['raw'])
        s.update(begin_monotonic=tick(), end_monotonic=tick())
        return s
    if sequence == 26:
        backend.samples = {35993: backend.samples[35993]}
    p = NativeCapture(35993, backend, ps, tick)
    s = p()
    c = Custody(35993, 35993, seed['36592']['after']['path'], 'B38-historical-replay')
    c.observe(s)
    require(not c.rejections and s['native']['error'] is None, repr([r['reason'] for r in c.rejections]))
    require(s['native']['acquisition']['original'] == plain(original), 'actual initial receipt changed')
    require(len(s['native']['rounds']) == 1, 'per-subject rounds consumed')
    if sequence == 16:
        require(s['native']['acquisition']['records']['36592']['before'] == seed['36592']['before'],
                'recorded native target identity changed')
        require(c.acquired_chains[36592]['tracer'] == 36593 and not c.transitions,
                'actual canonical tracer chain not authenticated')
        ack = c.acknowledge({'pid':36592, 'pgid':36592, 'run_id':c.run_id, 'flags':134})
        validate_ack(ack,36592,c.run_id,c.binary)
    else:
        require(confirmed_departures(s) == gone, 'sequence26 lost tracer/helper departure')
        require(len(s['native']['corroborations']) == 4, 'sequence26 partial subject receipt lost')
    require(data['first_rejection_complete']['first'] is None
            and data['cleanup_sequence26_complete']['native']['error'] ==
            'acquisition corroboration count exhausted', 'prior failure relabeled')


def run_tests(results):
    def check(name, action):
        error = None
        try: action()
        except Exception as exc: error = repr(exc)
        results.append({'name': 'B38-' + name, 'pass': error is None, 'error': error,
            'provenance': 'OFFLINE injected native/probe evidence; actual initial B37 replay where named, fresh rounds SYNTHETIC'})
    for count in (1, 2):
        def positive(count=count):
            c,h,p,state,backend = setup(rounds=count)
            s = p()
            c.observe(s)
            require(not c.rejections, repr([r['reason'] for r in c.rejections]))
            require(len(s['native']['rounds']) == count and state['calls'] == count + 1,
                    'snapshot allowance differs')
            require(s['native']['acquisition']['original']['rows'][1]['ppid'] == 100,
                    'original parent disagreement lost')
            finish(c,h,p,state,backend)
        check('coherent-' + str(count) + '-round-producer-ack-cleanup-finalization', positive)
    for sequence in (16,26):
        check('actual-sequence-' + str(sequence) + '-raw-and-injected-fresh-replay', lambda sequence=sequence: historical(sequence))
    for mode in ('unknown-parent','foreign-group','reuse','path','exec','lost','eof','budget',
                 'root-gone','partial','transient-native-parent','transient-ps-parent'):
        def negative(mode=mode):
            c,h,p,state,backend = setup(mode)
            s = p()
            c.observe(s)
            require(c.rejections and not c.targets and not c.groups, 'invalid initial authority')
            refuses(lambda:c.acknowledge(h))
            require(not c.cleanup_allowed(123,p()), 'invalid cleanup authority')
            if mode == 'partial':
                r = s['native']['rounds'][0]
                require(r['before'] and r['after'] and 'snapshot' in r
                        and 'end' not in r, 'partial failed batch receipt discarded')
        check('initial-reject-' + mode, negative)
    def exhausted():
        c,h,p,state,backend = setup(rounds=3)
        s=p(); c.observe(s)
        require(c.rejections and len(s['native']['rounds']) == 2 and state['calls'] == 3,
                'count exhaustion reran acquisition')
    check('two-round-exhaustion-no-third-snapshot',exhausted)
    for count in (3,10,60):
        def combined(count=count):
            gone=tuple(range(125,125+count))
            c,h,p,state,backend=setup(departed=gone)
            s=p(); c.observe(s)
            require(not c.rejections and confirmed_departures(s)==set(gone)
                    and len(s['native']['rounds'])==1, 'F2/F3 did not share one batch')
            require(len(s['native']['corroborations'])==count, 'departed subject records dropped')
            finish(c,h,p,state,backend)
        check('combined-parent-and-'+str(count)+'-departures',combined)
    def subject_limit():
        c,h,p,state,backend=setup(departed=tuple(range(125,186)))
        s=p(); c.observe(s)
        require(c.rejections and state['calls']==1 and not s['native'].get('rounds'), 'subject bound ignored')
    check('65-subject-limit-before-round',subject_limit)
    def returned():
        c,h,p,state,backend=setup('return',departed=(125,126,127))
        s=p(); c.observe(s)
        require(c.rejections and s['native']['rounds'][0]['after']['127'] is not None,
                'mixed absent/live contradictory batch accepted or lost')
        refuses(lambda:c.acknowledge(h))
    check('mixed-batch-reappearance-retains-all-samples',returned)
    for field in ('original','before','after','registration','events','subjects','rounds','end','missing','root'):
        for phase in ('ack','completion'):
            def tamper(field=field,phase=phase):
                c,h,p,state,backend=setup(departed=(125,126,127))
                s=p(); c.observe(s); ack=c.acknowledge(h)
                document=ack['custody'] if phase=='ack' else c.document()
                n=document['observations'][0]['native']
                if field=='original':n['acquisition']['original']['rows'][1]['ppid']=777
                if field in ('before','after'): n['rounds'][0][field]['123']['start_usec']+=1
                if field=='registration': n['acquisition']['records']['123']['registration']['serial']+=10
                if field=='events': n['rounds'][0]['events_after']['123' if '123' in n['rounds'][0]['events_after'] else 123].append(event(123,'lost'))
                if field=='subjects': n['rounds'][0]['subjects'].pop()
                if field=='rounds': n['rounds'].extend(copy.deepcopy(n['rounds'])*2)
                if field=='end': n['rounds'][0]['end']+=7
                if field=='missing': n['corroborations'].pop()
                if field=='root': n['acquisition']['root']=777
                if phase=='ack': refuses(lambda:validate_ack(ack,123,c.run_id,c.binary))
                else: refuses(lambda:validate_completion(ack,document,123,c.run_id,c.binary))
            check(phase+'-reject-tamper-'+field,tamper)
    def registration_not_replaced():
        c,h,p,state,backend=setup()
        s=p(); c.observe(s)
        require(backend.registered==[99,100,123,124], 'resampling registered watches again')
        require(all(e['registration']==s['native']['acquisition']['records'][key]['registration']
                    for key,e in s['native']['records'].items()), 'original registration lost')
    check('continuous-original-registration-no-replacement',registration_not_replaced)

    def known_departures():
        c,h,p,state,backend=setup()
        c.observe(p()); ack=c.acknowledge(h)
        state['rows']=copy.deepcopy(c.last['rows'])
        for pid in (125,126):
            state['rows'].append(row(pid,99,99,'/usr/bin/pmset'))
            backend.samples[pid]=Backend([state['rows'][-1]]).samples[pid]
        c.observe(p())
        require(not c.rejections, 'helper initial observations rejected')
        # Actual native terminal state, not ps display/no-event exit inference.
        backend.samples[123]['status']=5
        next(r for r in state['rows'] if r['pid']==123)['stat']='Z'
        for pid in (124,125,126):backend.samples.pop(pid)
        original_ps=p.ps_capture
        calls=[0]
        def depart_ps():
            calls[0]+=1
            if calls[0]>1:state['rows']=[r for r in state['rows'] if r['pid'] not in (124,125,126)]
            return original_ps()
        p.ps_capture=depart_ps
        s=p(); c.observe(s)
        require(not c.rejections and confirmed_departures(s)=={124,125,126}
                and len(s['native']['rounds'])==1, 'tracer/helper batch not accepted')
        refuses(lambda:c.acknowledge(h))
        require(not c.cleanup_allowed(123,p()), 'terminal tracer batch grants signal')
        state['rows']=[r for r in state['rows'] if r['pid']==99]
        backend.samples={99:backend.samples[99]}
        remaining,absent=final_absence(c,p,lambda g:False)
        require(absent and not remaining and not c.rejections,'tracer/helper batch release failed')
        validate_completion(ack,c.document(),123,c.run_id,c.binary)
    check('three-known-tracer-helper-departures-terminal-cleanup-finalization',known_departures)

    for mode in ('old-parent','new-parent','other-chain','root-parent','group'):
        def chain_negative(mode=mode):
            c,h,p,state,backend=setup()
            if mode=='old-parent':
                next(r for r in state['rows'] if r['pid']==123)['ppid']=777
                state['rows'].append(row(777,99,777,'/unknown/parent'))
                backend.samples[777]=Backend([state['rows'][-1]]).samples[777]
            if mode=='new-parent':backend.samples[124]['path']='/not-debugserver'
            if mode=='other-chain':
                next(r for r in state['rows'] if r['pid']==124)['ppid']=99
                backend.samples[124]['ppid']=99
            if mode=='root-parent':
                next(r for r in state['rows'] if r['pid']==100)['ppid']=1
                backend.samples[100]['ppid']=1
            if mode=='group':
                next(r for r in state['rows'] if r['pid']==124)['pgid']=99
                backend.samples[124]['pgid']=99
            s=p();c.observe(s)
            require(not c.targets, 'unknown/foreign target chain accepted')
            refuses(lambda:c.acknowledge(h))
            require(not c.cleanup_allowed(123,p()), 'unknown/foreign target gained signal authority')
        check('initial-chain-reject-'+mode,chain_negative)

    def prefilled_count():
        c,h,p,state,backend=setup()
        native={'begin':20,'rounds':[{},{}]}
        refuses(lambda:batch(p,snapshot(state['rows'],20),native,{99}))
        require(state['calls']==0 and len(native['rounds'])==2,'exhausted batch captured again')
    check('exhausted-shared-batch-never-captures-third',prefilled_count)
