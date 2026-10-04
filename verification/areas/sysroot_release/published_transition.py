"""Qualify actual published flat payload -> exact native candidate, without publishing."""
from datetime import datetime,timezone
import hashlib
import json
import os
from pathlib import Path
import shutil
import sys
import time
import uuid

import native_candidate
import legacy_transition_inventory
import native_dependency_preparation
from native_capacity import resources
from published_predecessor import POLICY,digest
from published_installation import payload,rehearse,verify_installed
from sifr_verify.execution_evidence import write_evidence,validate_records
from sifr_verify.graph_retirement import GraphLease
from sifr_verify.process_execution import execute
from sifr_verify.resource_admission import admit

CASES=('published-install','persisted-before','migration-failure-rollback','upgrade',
       'persisted-after','reinstall','persisted-reinstall','candidate-failure-rollback')
USER_FILES=('sifr.toml','Cargo.toml','src/main.sifr','src/lib.rs','user-state.json')


def command_inventory(candidate,output):
    binary=output/'published/managed/bin/sifr';project=output/'user-project'
    installer=Path(candidate['installer']['path']);run=[binary,'run','src/main.sifr','--offline','--quiet']
    rows=[('published-init',[binary,'init','--bin','--name','persisted_upgrade',project]),
          ('published-user-run',run),('failed-migration',['sh','-x',installer,'--no-modify-path']),
          ('published-after-rollback',run),('upgrade',['sh',installer,'--no-modify-path']),
          ('upgraded-version',[binary,'--version']),('upgraded-doctor',[binary,'doctor','--json','--verify-integrity']),
          ('candidate-user-run',run),('reinstall',['sh',installer,'--force','--no-modify-path']),
          ('reinstalled-version',[binary,'--version']),('reinstalled-doctor',[binary,'doctor','--json','--verify-integrity']),
          ('reinstalled-user-run',run),('failed-reinstall',['sh','-x',installer,'--force','--no-modify-path']),
          ('rollback-version',[binary,'--version']),('rollback-doctor',[binary,'doctor','--json','--verify-integrity']),
          ('rollback-user-run',run)]
    return [(name,[str(item) for item in argv]) for name,argv in rows]


def check_runtime(report,output,candidate):
    validate_records(report['cases'],list(CASES),complete=True,required_kinds={name:'runtime' for name in CASES})
    if type(report['runtime_assertions']) is not int or report['runtime_assertions']!=len(CASES):
        raise ValueError('transition runtime case accounting differs')
    expected=command_inventory(candidate,output)
    if [(row['id'],row['argv']) for row in report['commands']]!=expected:
        raise ValueError('transition native command inventory differs')
    for row in report['commands']:
        negative=row['id'] in {'failed-migration','failed-reinstall'}
        if (row['expected_success'] is not (not negative) or row['cause']!='exit' or row['truncated']
                or type(row['returncode']) is not int or (row['returncode']==0)==negative
                or set(row['output_sha256'])!={'stdout','stderr'}):
            raise ValueError('transition required process outcome is incomplete or differs')
        for stream,sha in row['output_sha256'].items():
            if digest(output/(row['id']+'.'+stream))!=sha: raise ValueError('transition raw process output differs')
    for name in ('published-user-run','published-after-rollback','candidate-user-run','reinstalled-user-run','rollback-user-run'):
        if (output/(name+'.stdout')).read_bytes().strip()!=b'9': raise ValueError('persisted native program result differs')
    for prefix in ('upgraded','reinstalled','rollback'):
        if (output/(prefix+'-version.stdout')).read_bytes().strip()!=('sifr '+candidate['version']).encode():
            raise ValueError('candidate runtime version differs')
        doctor=json.loads((output/(prefix+'-doctor.stdout')).read_text())
        if doctor['status']!='ok' or doctor['package_integrity_verified'] is not True:
            raise ValueError('candidate runtime package integrity was not verified')
    for name in ('failed-migration','failed-reinstall'):
        trace=(output/(name+'.stderr')).read_text()
        if not all(point in trace for point in ('atomic_generation_switch','write_install_manifest','rollback_install_transaction')):
            raise ValueError('failure did not exercise post-switch transaction rollback')


def check(path,*,_pending=False):
    path=Path(path).resolve(strict=True);output=path.parent;report=json.loads(path.read_text())
    if not _pending and (path.name!='receipt.json' or json.loads((output/'state.json').read_text())!=report):
        raise ValueError('transition needs its completed officially published receipt')
    candidate_path=Path(report['candidate_receipt']['path']);candidate=native_candidate.check(candidate_path)
    source=Path(candidate['source_root'])
    if (report['schema_version']!=1 or report['protocol']!='published-native-transition-v1' or report['status']!='passed'
            or report['source_commit']!=candidate['source_commit'] or report['version']!=candidate['version']
            or report['target']!=candidate['target'] or digest(candidate_path)!=report['candidate_receipt']['sha256']
            or report['producer_sha256']!=digest(Path(__file__)) or report['policy_sha256']!=digest(POLICY)
            or digest(source/'verification/areas/sysroot_release/published_transition.py')!=report['producer_sha256']
            or report['inventory_producer_sha256']!=digest(Path(legacy_transition_inventory.__file__))
            or digest(source/'verification/areas/sysroot_release/legacy_transition_inventory.py')!=report['inventory_producer_sha256']
            or report['dependency_producer_sha256']!=digest(Path(native_dependency_preparation.__file__))
            or digest(source/'verification/areas/sysroot_release/native_dependency_preparation.py')!=report['dependency_producer_sha256']):
        raise ValueError('transition source/producer/native input identity differs')
    start,finish=(datetime.fromisoformat(report[field]) for field in ('started_utc','finished_utc'))
    now=datetime.now(timezone.utc)
    if start.utcoffset() is None or finish.utcoffset() is None or not start<=finish<=now or (now-finish).total_seconds()>86400:
        raise ValueError('transition evidence is stale or incomplete')
    check_runtime(report,output,candidate)
    if user_identity(output/'user-project')!=report['user_identity']:
        raise ValueError('persisted user-owned files differ')
    for name in ('published_archive','published_installer'):
        if digest(Path(report[name]['path']))!=report[name]['sha256']: raise ValueError('published input bytes differ')
    policy=json.loads(POLICY.read_text());_,rows=payload(policy,Path(report['published_archive']['path']),report['target'],report['version'])
    published=output/'published/receipt.json'
    if digest(published)!=report['published_receipt_sha256']: raise ValueError('published installation receipt differs')
    previous=json.loads(published.read_text())
    if previous['status']!='legacy-installed' or previous['published_version']!=policy['version'] or previous['target']!=report['target']:
        raise ValueError('actual published predecessor was not installed')
    legacy=Path(report['retained_predecessor']);managed=output/'published/managed'
    if legacy.parent!=managed/'.sifr-generations' or not legacy.name.startswith('legacy.'):
        raise ValueError('retained predecessor path differs')
    if legacy_transition_inventory.predecessor(managed,Path(report['rolled_back_legacy']),
            report['published_install_manifest_sha256'],rows)!=legacy:
        raise ValueError('retained published predecessor inventory differs')
    native_dependency_preparation.check(report['dependency_preparation'],output/'dependency-preparation',managed,legacy)
    if (managed/'.sifr-current').resolve()!=Path(report['reinstalled_generation']) or digest(managed/'bin/sifr')!=candidate['binary_sha256']:
        raise ValueError('selected rollback generation or compiler differs')
    if not Path(report['upgraded_generation']).is_dir() or report['upgraded_generation']==report['reinstalled_generation']:
        raise ValueError('reinstall did not preserve its distinct predecessor generation')
    for name,file in (('transport_sha256',output/'transport/curl'),('fault_tool_sha256',output/'fault-tools/cp')):
        if digest(file)!=report[name]: raise ValueError('qualification transport or failure fixture changed')
    return report


def user_identity(root):
    return {name:digest(root/name) for name in USER_FILES}


def admission_requirements():
    # This fixed rehearsal runs one Cargo worker and one small persisted math
    # program. Three GiB is a prospective combined process estimate; observed
    # full source preparation had a 1.8-GiB maximum child and structural doctor
    # execution 247 MiB. This is not a measured transition memory claim.
    return dict(disk_growth_bytes=3*1024**3,retained_copy_bytes=0,
        disk_reserve_bytes=2*1024**3,memory_peak_bytes=3*1024**3,
        tmpfs_growth_bytes=512*1024**2,memory_reserve_bytes=2*1024**3)


def qualify(candidate_receipt,archive,installer,output):
    candidate_receipt,archive,installer=(Path(item).resolve(strict=True) for item in (candidate_receipt,archive,installer))
    candidate=native_candidate.check(candidate_receipt)
    source=Path(candidate['source_root']);target=candidate['target'];version=candidate['version']
    if digest(source/'verification/areas/sysroot_release/published_transition.py')!=digest(Path(__file__)):
        raise ValueError('transition producer differs from prepared candidate source')
    policy=json.loads(POLICY.read_text());asset,rows=payload(policy,archive,target,version)
    output=output.absolute()
    if output.is_relative_to(source) or any(item.is_symlink() for item in (output,*output.parents)):
        raise ValueError('published transition requires owned canonical external output')
    output.mkdir(mode=0o700,parents=True,exist_ok=False)
    report={'schema_version':1,'protocol':'published-native-transition-v1','status':'incomplete',
        'source_commit':candidate['source_commit'],'target':target,'version':version,
        'candidate_receipt':{'path':str(candidate_receipt),'sha256':digest(candidate_receipt)},
        'published_archive':{'path':str(archive),'sha256':digest(archive)},
        'published_installer':{'path':str(installer),'sha256':digest(installer)},
        'producer_sha256':digest(Path(__file__)),'policy_sha256':digest(POLICY),
        'inventory_producer_sha256':digest(Path(legacy_transition_inventory.__file__)),
        'dependency_producer_sha256':digest(Path(native_dependency_preparation.__file__)),
        'started_utc':datetime.now(timezone.utc).isoformat(),'cases':[],'commands':[],'runtime_assertions':0}
    def save(): (output/'state.json').write_text(json.dumps(report,indent=2)+'\n')
    case_clock=time.monotonic()
    def passed(name):
        nonlocal case_clock
        if name!=CASES[len(report['cases'])]: raise ValueError('transition case order differs')
        now=time.monotonic()
        report['cases'].append({'id':name,'state':'passed','phases':['selected','validated','executed','passed'],
            'execution_kind':'runtime','infrastructure':'none','executed_count':1,'elapsed_seconds':now-case_clock})
        case_clock=now;save()
    save();lease=None
    try:
        # Estimates include two retained native generations and both user build
        # configurations. Existing source/compiler preparation is not repeated.
        capacity=resources(output)
        report['admission']=admit(capacity,admission_requirements())
        legacy=rehearse(policy,archive=archive,installer=installer,target=target,candidate_version=version,
            output=output/'published',resource_discovery=lambda *,disk_path:resources(disk_path))
        report['published_receipt_sha256']=digest(output/'published/receipt.json')
        managed=Path(legacy['managed_root']);binary=managed/'bin/sifr'
        report['managed_root']=str(managed);passed('published-install')
        env=os.environ.copy()
        for name in ('SIFR_SYSROOT','SIFR_SYSROOT_MODE','SIFR_CARGO','SIFR_RUSTC','SIFR_INSTALL_LOCK_HELD',
                     'SIFR_INSTALL_MANIFEST_DIR','SIFR_TEST_CHANNEL_METADATA_PATH','CARGO_TARGET_DIR'):
            env.pop(name,None)
        env.update(SIFR_INSTALL_DIR=str(managed/'bin'),SIFR_NO_MODIFY_PATH='1',SIFR_TARGET=target,
            SIFR_CACHE_DIR=str(output/'cache'),CARGO_NET_OFFLINE='true',CARGO_INCREMENTAL='0',
            CARGO_PROFILE_DEV_DEBUG='0',CARGO_BUILD_JOBS='1',
            SIFR_VERIFY_DISK_FLOOR_PATH=str(output),SIFR_VERIFY_DISK_FLOOR_BYTES=str(3*1024**3))
        graph=output/'target/sysroot_release/source-cargo-target'
        if sys.platform.startswith('linux'):
            lease=GraphLease(output,'target/sysroot_release/source-cargo-target',str(uuid.uuid4())).acquire({'published-transition'})
        env['CARGO_TARGET_DIR']=str(graph)
        transport=output/'transport';transport.mkdir()
        (transport/'routes.json').write_text(json.dumps({
            'https://github.com/sifr-lang/sifr/releases/download/'+version+'/sifr-'+version+'-'+target+'.tar.gz':candidate['archive']['path']}))
        curl=transport/'curl'
        curl.write_text('#!'+sys.executable+'\n'+'''import json,pathlib,shutil,sys
args=sys.argv[1:];routes=json.loads(pathlib.Path(__file__).with_name('routes.json').read_text())
urls=[arg for arg in args if arg.startswith(('https://','file://'))]
if len(urls)!=1 or urls[0] not in routes or '-o' not in args: raise SystemExit('unlisted transition URL')
shutil.copyfile(routes[urls[0]],args[args.index('-o')+1])
''');curl.chmod(0o700)
        env['PATH']=str(transport)+os.pathsep+env['PATH']
        report['transport_sha256']=digest(curl)
        def run(name,argv,*,extra=None,success=True,cwd=output):
            result=execute([str(item) for item in argv],cwd=cwd,env=env|(extra or {}),deadline_seconds=2400,limit_bytes=32*1024**2)
            raw={}
            for stream in ('stdout','stderr'):
                path=output/(name+'.'+stream);path.write_bytes(getattr(result,stream));raw[stream]=digest(path)
            report['commands'].append({'id':name,'argv':[str(item) for item in argv],'cause':result.cause,
                'returncode':result.returncode,'truncated':result.truncated,'expected_success':success,
                'output_sha256':raw,'elapsed_seconds':result.elapsed_seconds});save()
            if result.cause!='exit' or result.truncated or (result.returncode==0)!=success:
                raise ValueError(name+': incomplete or unexpected native outcome')
            return result.stdout
        report['dependency_preparation']=native_dependency_preparation.prepare(managed,output/'dependency-preparation');save()
        project=output/'user-project'
        run('published-init',[binary,'init','--bin','--name','persisted_upgrade',project])
        (project/'src/main.sifr').write_text('from sifr.math import sqrt\n\ndef main():\n    print(int(sqrt(81.0)))\n')
        (project/'user-state.json').write_text('{"user_owned":true,"revision":1}\n')
        report['user_identity']=user_identity(project)
        def user_run(label):
            actual=run(label,[binary,'run','src/main.sifr','--offline','--quiet'],cwd=project)
            if actual.strip()!=b'9' or user_identity(project)!=report['user_identity']:
                raise ValueError('persisted source/configuration or program result differs')
        user_run('published-user-run');passed('persisted-before')
        initial_receipt=digest(managed/'install.json')
        report['published_install_manifest_sha256']=initial_receipt
        fault=output/'fault-tools';fault.mkdir();cp=fault/'cp';real_cp=shutil.which('cp')
        cp.write_text('#!'+sys.executable+'\n'+'''import os,pathlib,sys
args=sys.argv[1:]
if len(args)==2 and pathlib.Path(args[0]).name=='install.json' and pathlib.Path(args[1]).name=='install.json' and '/.sifr-generations/'+os.environ['TRANSITION_VERSION']+'-' in args[1]:
    raise SystemExit(71)
os.execv(os.environ['TRANSITION_REAL_CP'],[os.environ['TRANSITION_REAL_CP'],*args])
''');cp.chmod(0o700);report['fault_tool_sha256']=digest(cp)
        new_installer=Path(candidate['installer']['path'])
        run('failed-migration',['sh','-x',new_installer,'--no-modify-path'],success=False,
            extra={'SIFR_MIGRATE_LEGACY':'1','PATH':str(fault)+os.pathsep+env['PATH'],
                   'TRANSITION_VERSION':version,'TRANSITION_REAL_CP':real_cp})
        if ('rollback_install_transaction' not in (output/'failed-migration.stderr').read_text()
                or (managed/'.sifr-current').exists() or (managed/'.sifr-current').is_symlink()
                or digest(managed/'install.json')!=initial_receipt):
            raise ValueError('failed migration did not restore the exact published installation')
        verify_installed(managed,rows)
        rollback_roots=list((managed/'.sifr-generations').glob('legacy.*'))
        if len(rollback_roots)!=1: raise ValueError('failed migration legacy inventory differs')
        rolled_back=legacy_transition_inventory.rollback_residue(managed,rollback_roots[0],initial_receipt)
        report['rolled_back_legacy']=str(rolled_back)
        user_run('published-after-rollback');passed('migration-failure-rollback')
        run('upgrade',['sh',new_installer,'--no-modify-path'],extra={'SIFR_MIGRATE_LEGACY':'1'})
        def selected(label):
            generation=(managed/'.sifr-current').resolve(strict=True)
            if digest(binary)!=candidate['binary_sha256'] or run(label+'-version',[binary,'--version']).strip()!=('sifr '+version).encode():
                raise ValueError('selected candidate binary differs from qualified bundle')
            doctor=json.loads(run(label+'-doctor',[binary,'doctor','--json','--verify-integrity']))
            if doctor['status']!='ok' or not doctor['package_integrity_verified']:
                raise ValueError('candidate installed package integrity was not verified')
            return generation
        upgraded=selected('upgraded');report['upgraded_generation']=str(upgraded)
        legacy_root=legacy_transition_inventory.predecessor(managed,rolled_back,initial_receipt,rows)
        report['retained_predecessor']=str(legacy_root);passed('upgrade')
        user_run('candidate-user-run');passed('persisted-after')
        run('reinstall',['sh',new_installer,'--force','--no-modify-path'])
        reinstalled=selected('reinstalled')
        if reinstalled==upgraded or not upgraded.is_dir(): raise ValueError('reinstall mutated the previous generation')
        report['reinstalled_generation']=str(reinstalled);passed('reinstall')
        user_run('reinstalled-user-run');passed('persisted-reinstall')
        bad=output/'receipt-parent-is-file';bad.write_text('owned transaction failure fixture\n')
        run('failed-reinstall',['sh','-x',new_installer,'--force','--no-modify-path'],success=False,
            extra={'SIFR_INSTALL_MANIFEST_DIR':str(bad)})
        if (managed/'.sifr-current').resolve()!=reinstalled or 'rollback_install_transaction' not in (output/'failed-reinstall.stderr').read_text():
            raise ValueError('candidate transaction failure did not roll back the current generation')
        selected('rollback');user_run('rollback-user-run');passed('candidate-failure-rollback')
        legacy_transition_inventory.predecessor(managed,rolled_back,initial_receipt,rows)
        if native_candidate.check(candidate_receipt)!=candidate or digest(archive)!=report['published_archive']['sha256'] or digest(installer)!=report['published_installer']['sha256']:
            raise ValueError('native qualification inputs changed')
        if lease: lease.passed_consumer('published-transition')
        report.update(status='passed',runtime_assertions=len(CASES),finished_utc=datetime.now(timezone.utc).isoformat(),
            rollback_scope='failed migration restores published flat payload; failed reinstall restores exact candidate generation',
            transport_scope='allowlisted local bytes; actual published installer/archive; no publication')
        save();check(output/'state.json',_pending=True);write_evidence(output/'receipt.json',report)
    except BaseException as error:
        report.update(status='failed',failure=type(error).__name__+': '+str(error),finished_utc=datetime.now(timezone.utc).isoformat());save();raise
    finally:
        if lease: lease.close()
    return report


if __name__=='__main__':
    import argparse
    parser=argparse.ArgumentParser(description=__doc__)
    commands=parser.add_subparsers(dest='command',required=True)
    prepare=commands.add_parser('qualify')
    for name in ('candidate-receipt','archive','installer','output'): prepare.add_argument('--'+name,type=Path,required=True)
    verify=commands.add_parser('check');verify.add_argument('--receipt',type=Path,required=True)
    args=vars(parser.parse_args());command=args.pop('command')
    print(json.dumps(check(args['receipt']) if command=='check' else qualify(**args),indent=2))
