"""Offline retirement producer/supervisor tests; no native process or signals."""
import copy
import json
from unittest.mock import patch
from boundaries_core import E, BINARY, run_id, save
from boundaries_inventory import Inventory, validate_absence
from boundaries_lifecycle import Lifecycle, absence
from coverage_lifecycle_tests import Backend
from coverage_custody_tests import row, snapshot
from coverage_acquisition import plain
from coverage_symbols import require
from coverage_b40_tests import saved, initial_setup


def gone_backend():
    backend=Backend([])
    backend.samples={}
    return backend


def gone_snapshot(tick=10):
    value=plain(snapshot([],tick))
    value.update(rows=[],raw='')
    return value


def observer_tests(suite):
    sample={'pid':123,'pgid':123,'ppid':100,'path':str(BINARY),
            'start_sec':1000,'start_usec':123,'status':4}
    for field in ('initial_sample','before','after','registration-before','registration-after',
                  'round-before','round-after','nested-round-ps'):
        def discovery(field=field):
            inv=Inventory(99,BINARY)
            if field.startswith('registration-'):
                value={'native':{'records':{'123':{'registration':{field.split('-')[1]:sample}}}}}
            elif field.startswith('round-'):
                value={'native':{'rounds':[{field.split('-')[1]:{'123':sample}}]}}
            elif field=='nested-round-ps':
                value={'native':{'rounds':[{'snapshot':snapshot([row(100,99,100,'/lldb'),row(123,100,123,str(BINARY))])}]}}
            else:value={'native':{'records':{'123':{field:sample}}}}
            inv.collect(value)
            require(123 in inv.pids and 123 in inv.groups and inv.candidates,
                    'native-only/round-only group omitted')
            restored=Inventory(99,BINARY);restored.load(inv.document())
            require(restored.groups==inv.groups and restored.candidates==inv.candidates,
                    'producer-to-supervisor provenance lost')
        suite.check('B40-inventory-' + field,discovery)

    for mode in ('absent','live-group','unknown-group','missing-group-proof','missing-pid-proof'):
        def proof(mode=mode):
            inv=Inventory(99,BINARY);inv.collect({'native':{'records':{'123':{'initial_sample':sample}}}})
            receipt=absence(inv,gone_backend(),gone_snapshot,
                            lambda g: True if mode=='live-group' else None if mode=='unknown-group' else False)
            require(not receipt['signals'],'retirement signaled a rejected identity')
            if mode in ('live-group','unknown-group'):
                require(receipt['state']=='FAIL','absent PID hid live/unknown group')
                return
            if mode=='missing-group-proof':receipt['groups'].pop('123')
            if mode=='missing-pid-proof':receipt['after'].pop('123')
            validate_absence(receipt,inv)
        suite.check('B40-absent-PID-retirement-' + mode,proof,mode in ('missing-group-proof','missing-pid-proof'))

    def foreign():
        inv=Inventory(99,BINARY)
        inv.collect({'native':{'records':{'123':{'initial_sample':dict(sample,pgid=99)},
                                        '124':{'before':dict(sample,pid=124,pgid=777)},
                                        '888':{'before':dict(sample,pid=888,pgid=888,path='/foreign')}}}})
        require(not inv.groups and len(inv.candidates)==3 and not any(c['required_absence'] for c in inv.candidates),
                'foreign/shared group became an owned obligation')
        from coverage_custody import Custody
        c=Custody(99,99,BINARY,run_id(0))
        require(not c.groups and not c.cleanup_allowed(777,snapshot([],10)),
                'inventory granted signal authority')
    suite.check('B40-arbitrary-foreign-shared-groups-never-signal-authority',foreign)

    for mode in ('handshake-only','terminal-only','late-handshake','wrong-run','partial-json'):
        def launch(mode=mode):
            out=suite.root/('launch-'+mode);out.mkdir()
            d=out/'target-0';d.mkdir()
            save(d/'launch-permit.json',{'index':0,'run_id':run_id(0)})
            inv=Inventory(99,BINARY)
            if mode=='late-handshake':
                inv.collect_launch_records(out,run_id)
                require(inv.pids=={99},'unwritten launch invented')
            path=d/('observer-result.json' if mode=='terminal-only' else 'inferior.json')
            record={'pid':123,'run_id':run_id(0),'flags':134,'pgid':123,'state':'INCONCLUSIVE'}
            if mode=='terminal-only':record.pop('pgid')
            if mode=='wrong-run':record['run_id']='foreign'
            if mode=='partial-json':path.write_text('{')
            else:save(path,record)
            inv.collect_launch_records(out,run_id)
            if mode in ('wrong-run','partial-json'):
                require(inv.errors and not inv.groups,'unproved launch record accepted')
            else:
                require(inv.groups=={123} and inv.pids=={99,123} and inv.launch_records,
                        'already-written target record missing without ACK')
        suite.check('B40-owned-launch-record-' + mode,launch)

    for mode in ('registration-exception','after-exception','round-exception','consumer-exception'):
        def partial(mode=mode):
            c,h,p,state,b=initial_setup('partial' if mode=='round-exception' else 'ordinary')
            inv=Inventory(99,c.binary);p.inventory=inv
            if mode=='registration-exception':
                b.on_register=lambda pid: (_ for _ in ()).throw(ValueError('register failed')) if pid==123 else None
            if mode=='after-exception':
                orig=b.sample;calls=[0]
                def read(pid):
                    if pid==123:
                        calls[0]+=1
                        if calls[0]==3:raise ValueError('native after failed')
                    return orig(pid)
                b.sample=read
            s=p()
            if mode=='consumer-exception':s['native']['error']='injected consumer rejection'
            c.observe(s)
            require(c.rejections and not c.targets and 123 in inv.groups,
                    'exception discarded target-own group or accepted target')
            restore=Inventory(99,c.binary);restore.load(inv.document())
            require(123 in restore.groups and restore.candidates,'partial producer flush lost provenance')
        suite.check('B40-pre-handshake-' + mode,partial)

    def frozen35():
        doc=saved()
        binary=doc['rejections'][0]['observation']['native']['records']['49460']['after']['path']
        inv=Inventory(doc['root_pid'],binary)
        inv.collect(doc)
        producer=json.loads((E/'prior-b39-host/boundaries/producer-lifecycle.json').read_text())
        inv.load(producer['inventory'])
        inv.groups.add(inv.root) # exact root handle belongs to the supervisor
        original=json.loads((E/'prior-b39-host/complete-process-release.json').read_text())
        inv.collect(original['snapshot'])
        release=json.loads((E/'prior-b39-host/supplemental-complete-release.json').read_text())
        inv.collect(release['snapshot'])
        require(inv.groups=={49351,49424,49460},'frozen native-only third group omitted')
        require(inv.pids==set(release['pids']) and len(inv.pids)==35,
                'frozen35 subject/probe inventory differs: extra=' + str(inv.pids-set(release['pids']))
                + ' missing=' + str(set(release['pids'])-inv.pids))
        validate_absence(release,inv)
        try:validate_absence(original,inv)
        except ValueError:return
        raise ValueError('original limited34PID/2group release retroactively accepted')
    suite.check('B40-frozen35subjects-three-groups-original-limited-proof-rejected',frozen35)
    supervisor_tests(suite)


def supervisor_tests(suite):
    import boundaries_supervisor as supervisor
    for mode in ('all-six', 'live-sixth-group', 'missing-join', 'over-budget', 'before-normal-read-exception'):
        def execute(mode=mode):
            owned=suite.root/('supervisor-'+mode);owned.mkdir();out=owned/'boundaries'
            clock=[1.0]
            child_inv=Inventory(99,BINARY)
            groups={99}|set(range(123,129))
            class Process:
                pid=99
                returncode=None
                def __init__(self,*a,**kw):
                    out.mkdir()
                    for index,pid in enumerate(range(123,129)):
                        d=out/('target-'+str(index));d.mkdir()
                        save(d/'launch-permit.json',{'index':index,'run_id':run_id(index)})
                        save(d/'inferior.json',{'pid':pid,'pgid':pid,'flags':134,'run_id':run_id(index)})
                        child_inv.candidate({'pid':pid,'pgid':pid},'exact-owned-target',rooted=True)
                    save(out/'producer-lifecycle.json',{'inventory':child_inv.document()})
                    save(out/'process-release.json',{'watcher_alive':mode=='missing-join','monitor_alive':False})
                    save(out/'outcome.json',{'state':'PASS'})
                def communicate(self,timeout):
                    require(timeout>0,'unbounded supervisor wait')
                    self.returncode=0
                    clock[0]=542 if mode=='over-budget' else 10
                    if mode=='before-normal-read-exception':raise ValueError('injected after root exit before normal read')
                    return 'saved output',''
                def poll(self):return self.returncode
                def kill(self):raise ValueError('offline supervisor unexpectedly signaled')
            class Life(Lifecycle):
                def __init__(self,inv,deadline):super().__init__(inv,deadline,clock=lambda:clock[0])
                def capture(self):return gone_snapshot(clock[0])
            with patch.object(supervisor,'E',owned),patch.object(supervisor,'OUT',out),\
                 patch.object(supervisor.subprocess,'Popen',Process),patch.object(supervisor,'Lifecycle',Life),\
                 patch.object(supervisor,'Darwin',gone_backend),\
                 patch.object(supervisor.time,'monotonic',lambda:clock[0]),\
                 patch.object(supervisor,'group_exists',lambda g:mode=='live-sixth-group' and g==128):
                code=supervisor.execute()
            outcome=json.loads((owned/'allocation-outcome.json').read_text())
            inv=json.loads((owned/'supervisor-inventory.json').read_text())
            require(set(inv['groups'])==groups and set(inv['pids'])==groups,'supervisor lost cumulative all-six/root inventory')
            if mode=='all-six':
                release=json.loads((owned/'complete-process-release.json').read_text())
                require(code==0 and release['state']=='PASS' and outcome['elapsed_seconds']<=540
                        and release['watcher_join_recorded'] and release['monitor_join_recorded'],
                        'complete within-budget root/group/join proof failed')
            else:require(code==1 and outcome['state']=='INCONCLUSIVE','failed root/group/join/deadline promoted')
            require((owned/'launcher-exit.json').exists(),'actual root exit receipt omitted')
        suite.check('B40-supervisor-'+mode,execute)
