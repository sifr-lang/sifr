"""Prepare an optimized actual native candidate bundle; no release publication."""
from datetime import datetime,timezone
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tomllib
import uuid

ROOT=Path(__file__).resolve().parents[3]
sys.path.insert(0,str(ROOT/'verification/runner'))
sys.path.insert(0,str(ROOT/'scripts/distribution'))
from sifr_verify.execution_evidence import write_evidence
from sifr_verify.graph_retirement import GraphLease
from sifr_verify.process_execution import execute
from sifr_verify.resource_admission import admit,worker_limit
from sifr_verify.process_disk_budget import DiskBudget
from qualify_stable_target import current_host_target
from native_capacity import resources
from published_predecessor import digest
from verify_release_archive import verify_archive


def tool_identity():
    def selected(name):
        executable=Path(subprocess.check_output(['rustup','which',name],text=True,timeout=30).strip()).resolve(strict=True)
        return {'path':str(executable),'sha256':digest(executable)}
    return {'cargo':selected('cargo'),'rustc':selected('rustc'),
            'rustc_version':subprocess.check_output(['rustc','-vV'],text=True,timeout=30),
            'python':{'path':str(Path(sys.executable).resolve()),'sha256':digest(Path(sys.executable).resolve())}}


def commands(root,directory,version,target):
    return {'build-package':[str(root/'scripts/distribution/build_release_artifacts.sh'),'--version',version,
            '--output-dir',str(directory/'artifacts'),'--cargo-build','--target',target,'--sysroot-root',str(root)],
            'installer':[str(root/'scripts/distribution/generate_version_installer.sh'),'--version',version,
            '--artifact-dir',str(directory/'artifacts'),'--out',str(directory/f'sifr-installer-{version}'),
            '--qualification-target',target]}


def clean_source(root):
    if subprocess.check_output(['git','status','--porcelain','--untracked-files=all','--ignore-submodules=none'],cwd=root,text=True).strip():
        raise ValueError('native candidate source must be committed and clean')
    return subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip()


def prepare(root,output):
    root=root.resolve(strict=True)
    source=clean_source(root)
    for name in ('native_candidate.py','native_capacity.py'):
        if digest(root/'verification/areas/sysroot_release'/name)!=digest(Path(__file__).parent/name):
            raise ValueError('native preparation tooling differs from the source candidate')
    target=current_host_target()
    version=tomllib.loads((root/'crates/sifr/Cargo.toml').read_text())['package']['version']
    cargo=shutil.which('cargo')
    if not cargo: raise ValueError('native Cargo unavailable')
    if any(os.environ.get(name) for name in ('RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','RUSTC','RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER','CARGO_BUILD_TARGET','CARGO_BUILD_RUSTFLAGS','SIFR_SYSROOT','SIFR_SYSROOT_MODE','SIFR_CARGO','SIFR_RUSTC','LD_PRELOAD','DYLD_INSERT_LIBRARIES')):
        raise ValueError('native candidate cannot inherit compiler/sysroot/loader overrides')
    if any(name.startswith('CARGO_PROFILE_RELEASE_') for name in os.environ):
        raise ValueError('native candidate requires the canonical release profile')
    output=output.absolute()
    if output.is_relative_to(root) or any(path.is_symlink() for path in (output,*output.parents)):
        raise ValueError('native candidate needs private canonical external output')
    output.mkdir(mode=0o700,parents=True,exist_ok=False)
    report={'schema_version':1,'protocol':'native-candidate-preparation-v1','status':'incomplete',
        'kind':'preparation-output','runtime_assertions':0,'source_root':str(root),'source_commit':source,
        'version':version,'target':target,'cargo_lock_sha256':digest(root/'Cargo.lock'),
        'producer_sha256':{name:digest(Path(__file__).parent/name) for name in ('native_candidate.py','native_capacity.py')},
        'tools':None,'started_utc':datetime.now(timezone.utc).isoformat(),'commands':[]}
    def save(): (output/'state.json').write_text(json.dumps(report,indent=2)+'\n')
    save();lease=None
    try:
        report['tools']=tool_identity();cargo=report['tools']['cargo']['path'];save()
        capacity=resources(output)
        report['admission']=admit(capacity,dict(disk_growth_bytes=2684354560,retained_copy_bytes=268435456,
            disk_reserve_bytes=2147483648,memory_peak_bytes=6442450944,tmpfs_growth_bytes=1073741824,memory_reserve_bytes=2147483648))
        owner=str(uuid.uuid4())
        lease=GraphLease(output,'target/sysroot_release/cargo-target',owner).acquire({'native-bundle'}) if sys.platform.startswith('linux') else None
        graph=output/'target/sysroot_release/cargo-target';graph.mkdir(parents=True,exist_ok=True)
        wrappers=output/'transport';wrappers.mkdir()
        events=output/'cargo.jsonl'
        wrapper=wrappers/'cargo'
        wrapper.write_text('#!'+sys.executable+'\n'+'''import os,pathlib,subprocess,sys
args=sys.argv[1:]
if args and args[0]=='build':
    process=subprocess.Popen([os.environ['NATIVE_REAL_CARGO'],*args,'--message-format=json-render-diagnostics'],stdout=subprocess.PIPE)
    with open(os.environ['NATIVE_CARGO_EVENTS'],'ab') as stream:
        while data:=process.stdout.read(65536):
            stream.write(data);stream.flush();sys.stdout.buffer.write(data);sys.stdout.buffer.flush()
    sys.exit(process.wait())
os.execv(os.environ['NATIVE_REAL_CARGO'],[os.environ['NATIVE_REAL_CARGO'],*args])
''');wrapper.chmod(0o700)
        inherited=DiskBudget.from_environment(os.environ)
        if inherited is not None and inherited.path.stat().st_dev!=output.stat().st_dev:
            raise ValueError('inherited native disk floor describes another filesystem')
        floor=max(3*1024**3,inherited.floor if inherited else 0)
        env=os.environ.copy()|{'PATH':str(wrappers)+os.pathsep+os.environ['PATH'],'NATIVE_REAL_CARGO':cargo,
            'NATIVE_CARGO_EVENTS':str(events),'CARGO_TARGET_DIR':str(graph),'CARGO_NET_OFFLINE':'true',
            'RUSTC':report['tools']['rustc']['path'],
            'CARGO_INCREMENTAL':'0','CARGO_BUILD_JOBS':str(worker_limit(capacity,2)),
            'RUSTFLAGS':'-C target-cpu=generic','SIFR_VERIFY_DISK_FLOOR_BYTES':str(floor),
            'SIFR_VERIFY_DISK_FLOOR_PATH':str(output)}
        def run(label,command):
            result=execute(command,cwd=root,env=env,deadline_seconds=7200,limit_bytes=32*1024**2)
            paths={}
            for stream in ('stdout','stderr'):
                path=output/(label+'.'+stream);path.write_bytes(getattr(result,stream))
                paths[stream]={'path':str(path),'sha256':digest(path)}
            report['commands'].append({'id':label,'argv':command,'cause':result.cause,'returncode':result.returncode,
                                      'truncated':result.truncated,'output':paths,'elapsed_seconds':result.elapsed_seconds});save()
            if result.cause!='exit' or result.returncode or result.truncated:
                raise ValueError('native candidate '+label+' failed or is incomplete')
        expected_commands=commands(root,output,version,target)
        run('build-package',expected_commands['build-package'])
        rows=[json.loads(line) for line in events.read_text().splitlines() if line.startswith('{')]
        artifacts=[row for row in rows if row.get('reason')=='compiler-artifact' and row.get('target',{}).get('name')=='sifr' and row.get('executable')]
        if len(artifacts)!=1: raise ValueError('native Cargo must identify one compiler')
        artifact=artifacts[0];profile=artifact['profile'];binary=Path(artifact['executable'])
        if profile['opt_level']!='3' or profile['test'] or profile['debug_assertions'] or binary!=graph/target/'release/sifr':
            raise ValueError('native compiler differs from canonical optimized target')
        archive=output/'artifacts'/f'sifr-{version}-{target}.tar.gz'
        verify_archive(str(archive),version,target)
        with tarfile.open(archive,'r:gz') as package:
            manifest=tomllib.loads(package.extractfile('sysroot.toml').read().decode())
            packaged=hashlib.sha256(package.extractfile('bin/sifr').read()).hexdigest()
        if (manifest['built-by-compiler-commit']!=source or packaged!=digest(binary)
                or manifest['cargo-lock-sha256']!=report['cargo_lock_sha256']):
            raise ValueError('native archive differs from source or actual Cargo executable')
        installer=output/f'sifr-installer-{version}'
        run('installer',expected_commands['installer'])
        report.update(cargo_artifact=artifact,cargo_events_sha256=digest(events),binary_sha256=packaged,
            archive={'path':str(archive),'sha256':digest(archive)},installer={'path':str(installer),'sha256':digest(installer)})
        if clean_source(root)!=source or digest(root/'Cargo.lock')!=report['cargo_lock_sha256'] or tool_identity()!=report['tools']:
            raise ValueError('native source changed during preparation')
        if lease:
            lease.passed_consumer('native-bundle')
            def clean(command,*,env):
                result=execute(command,cwd=root,env=env,deadline_seconds=900,limit_bytes=16*1024**2)
                for stream in ('stdout','stderr'):
                    (output/('retirement.'+stream)).write_bytes(getattr(result,stream))
                report['retirement_command']={'argv':command,'cause':result.cause,'returncode':result.returncode,'truncated':result.truncated,
                    'output':{stream:{'path':str(output/('retirement.'+stream)),'sha256':digest(output/('retirement.'+stream))} for stream in ('stdout','stderr')}}
                save()
                if result.cause!='exit' or result.returncode or result.truncated: raise ValueError('native graph retirement failed')
            report['retirement']=lease.retire(retained=output/'protected',protected=[binary],command_runner=clean,env=env,max_encoded_bytes=128*1024**2)
        report.update(status='prepared',finished_utc=datetime.now(timezone.utc).isoformat())
        candidate=output/'candidate.json';candidate.write_text(json.dumps(report,indent=2)+'\n')
        check(candidate,_pending=True)
        write_evidence(output/'receipt.json',report);save()
    except BaseException as error:
        report.update(status='failed',failure=type(error).__name__+': '+str(error),finished_utc=datetime.now(timezone.utc).isoformat());save();raise
    finally:
        if lease: lease.close()
    return report


def check(path,*,_pending=False):
    path=Path(path).resolve(strict=True)
    report=json.loads(path.read_text())
    if not _pending:
        if path.name!='receipt.json' or json.loads((path.parent/'state.json').read_text())!=report:
            raise ValueError('native candidate needs its completed officially published receipt')
    root=Path(report['source_root'])
    if (report['schema_version']!=1 or report['protocol']!='native-candidate-preparation-v1'
            or report['status']!='prepared' or report['kind']!='preparation-output' or report['runtime_assertions']!=0
            or report['source_commit']!=clean_source(root) or report['target']!=current_host_target()
            or report['version']!=tomllib.loads((root/'crates/sifr/Cargo.toml').read_text())['package']['version']
            or report['cargo_lock_sha256']!=digest(root/'Cargo.lock')
            or report['tools']!=tool_identity()
            or report['producer_sha256']!={name:digest(Path(__file__).parent/name) for name in ('native_candidate.py','native_capacity.py')}):
        raise ValueError('native candidate preparation identity differs')
    for name,value in report['producer_sha256'].items():
        if digest(root/'verification/areas/sysroot_release'/name)!=value:
            raise ValueError('native candidate source producer bytes differ')
    start,finish=(datetime.fromisoformat(report[field]) for field in ('started_utc','finished_utc'))
    now=datetime.now(timezone.utc)
    if start.utcoffset() is None or finish.utcoffset() is None or not start<=finish<=now or (now-finish).total_seconds()>86400:
        raise ValueError('native candidate preparation is stale or incomplete')
    directory=path.parent
    expected_commands=commands(root,directory,report['version'],report['target'])
    for item in report['commands']:
        if item['argv']!=expected_commands.get(item['id']): raise ValueError('native preparation command differs')
        if item['cause']!='exit' or item['returncode'] or item['truncated']:
            raise ValueError('native candidate process did not complete')
        if set(item['output'])!={'stdout','stderr'}: raise ValueError('native raw stream inventory differs')
        for stream,raw in item['output'].items():
            raw_path=directory/(item['id']+'.'+stream)
            if raw['path']!=str(raw_path) or digest(raw_path)!=raw['sha256']: raise ValueError('native preparation raw output differs')
    events=directory/'cargo.jsonl'
    if digest(events)!=report['cargo_events_sha256']: raise ValueError('native Cargo event bytes differ')
    rows=[json.loads(line) for line in events.read_text().splitlines() if line.startswith('{')]
    artifacts=[row for row in rows if row.get('reason')=='compiler-artifact' and row.get('target',{}).get('name')=='sifr' and row.get('executable')]
    if artifacts!=[report['cargo_artifact']]: raise ValueError('native compiler Cargo artifact differs')
    artifact=artifacts[0];profile=artifact['profile']
    if (profile['opt_level']!='3' or profile['test'] or profile['debug_assertions'] or profile['overflow_checks']
            or artifact['manifest_path']!=str(root/'crates/sifr/Cargo.toml')
            or artifact['target']['kind']!=['bin'] or artifact['target']['crate_types']!=['bin']
            or artifact['target']['src_path']!=str(root/'crates/sifr/src/main.rs')
            or artifact['executable']!=str(directory/'target/sysroot_release/cargo-target'/report['target']/'release/sifr')):
        raise ValueError('native compiler optimized source/target profile differs')
    if [item['id'] for item in report['commands']]!=['build-package','installer']:
        raise ValueError('native candidate preparation command inventory differs')
    version=report['version'];target=report['target']
    archive=Path(report['archive']['path']);installer=Path(report['installer']['path'])
    if archive!=directory/'artifacts'/f'sifr-{version}-{target}.tar.gz' or installer!=directory/f'sifr-installer-{version}':
        raise ValueError('native candidate output path differs')
    if digest(archive)!=report['archive']['sha256'] or digest(installer)!=report['installer']['sha256']:
        raise ValueError('native candidate archive or installer bytes differ')
    if Path(str(archive)+'.sha256').read_text().strip()!=report['archive']['sha256']:
        raise ValueError('native candidate archive checksum differs')
    verify_archive(str(archive),version,target)
    with tarfile.open(archive,'r:gz') as package:
        manifest=tomllib.loads(package.extractfile('sysroot.toml').read().decode())
        binary=hashlib.sha256(package.extractfile('bin/sifr').read()).hexdigest()
    if (manifest['built-by-compiler-commit']!=report['source_commit'] or binary!=report['binary_sha256']
            or manifest['cargo-lock-sha256']!=report['cargo_lock_sha256']):
        raise ValueError('native archive differs from prepared source/compiler')
    return report


if __name__=='__main__':
    import argparse
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command',choices=('prepare','check'))
    parser.add_argument('--source-root',type=Path,default=ROOT)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    if args.command=='prepare': prepare(args.source_root,args.output)
    else: check(args.output)
