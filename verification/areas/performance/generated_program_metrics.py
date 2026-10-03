"""Correctness-checked native-program observations, separate from compiler time."""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
import math
import os
from pathlib import Path
import re
import shutil
import statistics
import subprocess
import sys

from benchmark_manifest import BenchmarkError
from benchmark_process import run_owned_process
from compiler_lanes import validate_receipt
from sifr_verify.resource_admission import discover, admit, worker_limit
from measurement_timer import managed_timer_identity
from program_evidence import read_metrics, producer_identity, validate_capture, target_cpu_generic
from program_artifact_identity import application_events

ROOT = Path(__file__).resolve().parents[3]
AREA = Path(__file__).parent
POLICY = AREA/'data/generated_programs.json'
PROBE = AREA/'program_process_probe.py'


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def policy():
    value=json.loads(POLICY.read_text())
    if (value['schema_version'] != 1 or value['protocol'] != 'generated-program-observations-v1'
            or value['optimization'] != 'release' or value['target_cpu'] != 'generic'
            or value['levels'] != {'smoke':{'warmups':0,'measured':1},
                                  'representative':{'warmups':2,'measured':10},'full':{'warmups':2,'measured':25}}
            or [row['id'] for row in value['cases']] != ['integer_throughput','startup']):
        raise BenchmarkError('unsupported generated-program contract')
    for case in value['cases']:
        path=(ROOT/case['source_path']).resolve()
        if not path.is_relative_to(AREA/'programs') or digest(path) != case['source_sha256']:
            raise BenchmarkError('generated-program source changed without registration')
        if type(case['work_units']) is not int or case['work_units'] <= 0:
            raise BenchmarkError('logical program work must be positive')
    if (value['owner'] != 'compiler/generated-program-performance' or value['target'] != 'native-host'
            or value['scope'] != 'descriptive process observations; no numeric regression qualification'
            or value['allocation_metrics']['status'] != 'unavailable'
            or value['cases'][0]['required_metrics'] != ['throughput_units_per_second','peak_rss_bytes','cpu_time_ms']
            or value['cases'][1]['required_metrics'] != ['startup_lifecycle_ms']
            or value['cases'][0]['work_units'] != 100000000 or value['cases'][1]['work_units'] != 1
            or value['cases'][0]['expected_stdout'] != '45150\n' or value['cases'][1]['expected_stdout'] != '42\n'):
        raise BenchmarkError('program observation claims, metrics or oracle differ')
    return value


def dependencies(binary):
    command=['ldd',str(binary)]
    completed,deadline=run_owned_process(command,ROOT,30)
    if deadline or completed.returncode or 'not found' in completed.stdout:
        raise BenchmarkError('runtime library identity unavailable')
    paths=re.findall(r'(?:=>\s+|^\s*)(/\S+)\s+\(',completed.stdout,re.MULTILINE)
    if not paths:
        raise BenchmarkError('runtime loader inspection produced no dependencies')
    return [{'path':path,'sha256':digest(path)} for path in sorted(set(paths))]


def sample(binary: Path, case: dict, output: Path, timer: dict) -> dict:
    before=digest(binary)
    probe_before=digest(PROBE)
    completed,deadline=run_owned_process(
        [sys.executable,'-I','-S','-B',str(PROBE),timer['path'],str(binary),str(output)],ROOT,60)
    if deadline or completed.returncode:
        raise BenchmarkError('generated-program observation failed or timed out')
    (output/'control.json').write_text(completed.stdout)
    values=read_metrics(output,case,binary.stat().st_size)
    if (before != digest(binary) or probe_before != digest(PROBE)
            or timer['sha256'] != digest(timer['path'])):
        raise BenchmarkError('native executable or timer changed during observation')
    if any(values[name] is None for name in case['required_metrics']):
        raise BenchmarkError('required generated-program metric unavailable')
    return {'metrics':values,'binary_sha256':before,'raw_files':[
        {'path':str(output/name),'sha256':digest(output/name)} for name in ('stdout','stderr','timer.txt','control.json')],
        'probe_sha256':probe_before,'observations':1,'inner_timing_samples':1,
        'native_wall_scope':'GNU Time command; 0.01-second resolution',
        'startup_scope':'GNU Time and native launch/wait; no supervisor/probe startup subtraction'}


def require_clean_source():
    if subprocess.check_output(['git','status','--porcelain','--untracked-files=all'],cwd=ROOT):
        raise BenchmarkError('program preparation/capture requires the exact clean committed source')


def write_wrappers(output):
    # The compiler uses this wrapper for application Cargo commands. Its JSON
    # artifact profile is the authority; --release alone is not profile evidence.
    events=output/'cargo-artifacts.jsonl'
    wrapper=output/'record-cargo'
    wrapper.write_text('#!'+sys.executable+'\n'+'''import json,os,sys
from pathlib import Path
from sifr_verify.process_execution import execute
result=execute([os.environ['PROGRAM_CARGO'],*sys.argv[1:]],cwd=Path.cwd(),env=os.environ.copy(),limit_bytes=16777216)
with open(os.environ['PROGRAM_CARGO_EVENTS'],'ab') as out:
    out.write(result.stdout)
sys.stdout.buffer.write(result.stdout)
sys.stderr.buffer.write(result.stderr)
if result.truncated or result.cause != 'exit': raise SystemExit(2)
raise SystemExit(result.returncode)
''')
    wrapper.chmod(0o700)
    rust_events=output/'rustc-invocations.jsonl'
    rust_wrapper=output/'record-rustc'
    rust_wrapper.write_text('#!'+sys.executable+'\n'+r'''import json,os,sys
with open(os.environ['PROGRAM_RUSTC_EVENTS'],'a') as out:
    out.write(json.dumps(sys.argv[1:])+'\n')
os.execv(os.environ['PROGRAM_RUSTC'],[os.environ['PROGRAM_RUSTC'],*sys.argv[1:]])
''')
    rust_wrapper.chmod(0o700)
    return wrapper,rust_wrapper,events,rust_events


def prepare(compiler_receipt: Path, output: Path, resource_profile: str = "standard"):
    require_clean_source()
    contract=policy()
    compiler=json.loads(compiler_receipt.read_text())
    validate_receipt(ROOT,compiler['lane'],compiler)
    if any(os.environ.get(key) for key in ('RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','LD_PRELOAD','LD_LIBRARY_PATH','SIFR_SYSROOT','SIFR_SYSROOT_MODE')):
        raise BenchmarkError('unregistered compiler/runtime override prevents program preparation')
    output=output.resolve()
    output.parent.mkdir(parents=True,exist_ok=True)
    resources=discover(disk_path=output.parent)
    allocation=contract['resource_profiles'][resource_profile]
    admission=admit(resources,allocation)
    output.mkdir(parents=True,exist_ok=False,mode=0o700)
    rows=[]
    cargo=shutil.which('cargo');rustc=shutil.which('rustc')
    if not cargo or not rustc: raise BenchmarkError('Cargo/rustc executable unavailable')
    wrapper,rust_wrapper,events,rust_events=write_wrappers(output)
    previous={key:os.environ.get(key) for key in (
        'SIFR_CARGO','SIFR_RUSTC','PROGRAM_CARGO','PROGRAM_CARGO_EVENTS','PROGRAM_RUSTC',
        'PROGRAM_RUSTC_EVENTS','RUSTFLAGS','SIFR_CACHE_DIR','CARGO_TARGET_DIR','CARGO_BUILD_JOBS',
        'SIFR_VERIFY_DISK_FLOOR_BYTES','SIFR_VERIFY_DISK_FLOOR_PATH','SIFR_SYSROOT','SIFR_SYSROOT_MODE','CARGO_NET_OFFLINE')}

    os.environ.update(SIFR_CARGO=str(wrapper),SIFR_RUSTC=str(rust_wrapper),PROGRAM_CARGO=cargo,
                      PROGRAM_CARGO_EVENTS=str(events),RUSTFLAGS='-C target-cpu=generic',
                      PROGRAM_RUSTC=rustc,PROGRAM_RUSTC_EVENTS=str(rust_events),
                      SIFR_CACHE_DIR=str(output/'cache'),CARGO_TARGET_DIR=str(output/'private-target'),
                      CARGO_BUILD_JOBS=str(worker_limit(resources,2)),
                      SIFR_VERIFY_DISK_FLOOR_BYTES=str(max(allocation['disk_reserve_bytes']+1024**3,int(previous['SIFR_VERIFY_DISK_FLOOR_BYTES'] or '0'))),
                      SIFR_VERIFY_DISK_FLOOR_PATH=previous['SIFR_VERIFY_DISK_FLOOR_PATH'] or str(output.parent))
    os.environ['CARGO_NET_OFFLINE']='true'
    if compiler['lane']=='contributor-dev':
        os.environ.update(SIFR_SYSROOT=compiler['sysroot']['path'],SIFR_SYSROOT_MODE='source')
    (output/'preparation-state.json').write_text(json.dumps({'status':'incomplete','runtime_assertions':0,'admission':admission}))
    try:
        for case in contract['cases']:
            copied=output/'sources'/Path(case['source_path']).name
            copied.parent.mkdir(exist_ok=True,mode=0o700)
            shutil.copyfile(ROOT/case['source_path'],copied)
            if digest(copied)!=case['source_sha256']: raise BenchmarkError('program source copy differs')
            destination=output/case['id']
            os.environ['SIFR_CACHE_DIR']=str(output/'cache'/case['id'])
            os.environ['CARGO_TARGET_DIR']=str(output/'private-target'/case['id'])
            cargo_begin=events.stat().st_size if events.exists() else 0
            rustc_begin=rust_events.stat().st_size if rust_events.exists() else 0
            completed,deadline=run_owned_process([compiler['artifact']['path'],'--isolated','build',str(copied),
                                                  '--release','--output',str(destination)],output,2400)
            (output/(case['id']+'.compile.stdout')).write_text(completed.stdout)
            (output/(case['id']+'.compile.stderr')).write_text(completed.stderr)
            if deadline or completed.returncode:
                raise BenchmarkError('native program compilation failed')
            binary=destination/'sifr_output/target/final/sifr_output'
            cargo_end=events.stat().st_size;rustc_end=rust_events.stat().st_size
            artifact_rows=[json.loads(line) for line in events.read_bytes()[cargo_begin:cargo_end].decode().splitlines() if line.startswith('{')]
            invocations=[json.loads(line) for line in rust_events.read_bytes()[rustc_begin:rustc_end].decode().splitlines()]
            artifact,actual_rustc=application_events(artifact_rows,invocations,binary)
            profile=artifact['profile']
            if (profile['test'] or profile['opt_level']!='3' or profile['debug_assertions']
                    or not profile['overflow_checks'] or digest(artifact['executable']) != digest(binary)):
                raise BenchmarkError('native application profile/artifact differs from release contract')
            if not target_cpu_generic(actual_rustc):
                raise BenchmarkError('actual application rustc command lacks registered CPU target')
            protected=output/'binaries'/case['id'];protected.parent.mkdir(exist_ok=True,mode=0o700)
            shutil.copyfile(binary,protected);protected.chmod(0o500)
            if digest(protected)!=digest(binary): raise BenchmarkError('retained native program bytes changed')
            rows.append({'id':case['id'],'binary':str(protected),'binary_sha256':digest(protected),
                         'rustc_command':actual_rustc,
                         'cargo_events_range':[cargo_begin,cargo_end],'rustc_events_range':[rustc_begin,rustc_end],
                         'source_sha256':case['source_sha256'],'compiled_source_path':str(copied),'cargo_artifact':artifact,
                         'runtime_dependencies':dependencies(binary)})
    except BaseException as error:
        (output/'preparation-state.json').write_text(json.dumps({'status':'failed','runtime_assertions':0,
                                                               'failure':type(error).__name__+': '+str(error)}))
        raise
    finally:
        for key,value in previous.items():
            if value is None: os.environ.pop(key,None)
            else: os.environ[key]=value
    try:
        validate_receipt(ROOT,compiler['lane'],compiler)
        require_clean_source()
    except BaseException as error:
        (output/'preparation-state.json').write_text(json.dumps({'status':'failed','runtime_assertions':0,
                                                               'failure':type(error).__name__+': '+str(error)}))
        raise
    receipt={'schema_version':1,'kind':'preparation-output','runtime_assertions':0,
             'policy_sha256':digest(POLICY),'compiler_receipt':str(compiler_receipt.resolve()),
             'compiler_receipt_sha256':digest(compiler_receipt),'programs':rows,
             'compile_time_scope':'separate preparation logs; excluded from all program observations',
             'target_cpu':'generic','optimization':'release','cargo_events_sha256':digest(events),
             'rustc_events_sha256':digest(rust_events),'admission':admission,'resource_profile':resource_profile}
    (output/'preparation-state.json').write_text(json.dumps({'status':'prepared','runtime_assertions':0}))
    (output/'prepared.json').write_text(json.dumps(receipt,indent=2)+'\n')
    return receipt


def capture(prepared: Path, output: Path, level: str):
    require_clean_source()
    contract=policy()
    receipt=json.loads(prepared.read_text())
    compiler=json.loads(Path(receipt['compiler_receipt']).read_text())
    validate_receipt(ROOT,compiler['lane'],compiler)
    if (receipt['kind']!='preparation-output' or receipt['policy_sha256']!=digest(POLICY)
            or receipt['compiler_receipt_sha256']!=digest(receipt['compiler_receipt'])
            or receipt['target_cpu']!='generic' or receipt['optimization']!='release'
            or [row['id'] for row in receipt['programs']] != [case['id'] for case in contract['cases']]):
        raise BenchmarkError('generated-program preparation identity differs')
    output=output.resolve();output.mkdir(parents=True,exist_ok=False,mode=0o700)
    timer=managed_timer_identity()
    result={'schema_version':1,'protocol':contract['protocol'],'level':level,'status':'incomplete',
            'numeric_regression_qualification':False,'policy_sha256':digest(POLICY),'prepared_sha256':digest(prepared),
            'started_utc':datetime.now(timezone.utc).isoformat(),'timer':timer,'rows':[],
            'prepared_path':str(prepared.resolve()),'producer':producer_identity(ROOT)}
    (output/'receipt.json').write_text(json.dumps(result,indent=2,allow_nan=False)+'\n')
    try:
        count=contract['levels'][level]
        for case,program in zip(contract['cases'],receipt['programs'],strict=True):
            binary=Path(program['binary'])
            if (digest(binary)!=program['binary_sha256'] or program['source_sha256']!=case['source_sha256']
                    or dependencies(binary)!=program['runtime_dependencies']):
                raise BenchmarkError('generated-program artifact/runtime drift')
            observations=[]
            for index in range(count['warmups']+count['measured']):
                observed=sample(binary,case,output/case['id']/str(index),timer)
                observed.update(warmup=index<count['warmups'],process_index=index)
                observations.append(observed)
            measured=[row for row in observations if not row['warmup']]
            summaries={}
            for metric in case['required_metrics']:
                values=[row['metrics'][metric] for row in measured]
                summaries[metric]={'median':statistics.median(values),
                                   'empirical_p95':sorted(values)[math.ceil(.95*len(values))-1]}
            result['rows'].append({'id':case['id'],'observations':observations,'process_count':len(measured),
                                   'summary':summaries,'binary_sha256':program['binary_sha256']})
            if digest(binary)!=program['binary_sha256'] or dependencies(binary)!=program['runtime_dependencies']:
                raise BenchmarkError('artifact/runtime changed during capture')
        if digest(prepared)!=result['prepared_sha256'] or digest(POLICY)!=result['policy_sha256']:
            raise BenchmarkError('preparation/policy changed during capture')
        validate_receipt(ROOT,compiler['lane'],compiler)
        require_clean_source()
        if producer_identity(ROOT)!=result['producer']:
            raise BenchmarkError('measurement producer/runtime changed during capture')
        result['status']='captured'
        result['finished_utc']=datetime.now(timezone.utc).isoformat()
        (output/'receipt.json').write_text(json.dumps(result,indent=2,allow_nan=False)+'\n')
        validate_capture(output/'receipt.json',contract,ROOT)
    except BaseException as error:
        result.update(status='failed',failure=type(error).__name__+': '+str(error))
        raise
    finally:
        if result['status']!='captured':
            result['finished_utc']=datetime.now(timezone.utc).isoformat()
            (output/'receipt.json').write_text(json.dumps(result,indent=2,allow_nan=False)+'\n')
    return result


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command',choices=('prepare','capture','check'))
    parser.add_argument('--compiler-receipt',type=Path)
    parser.add_argument('--prepared',type=Path)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--resource-profile',choices=('standard','constrained'),default='standard')
    parser.add_argument('--level',choices=('smoke','representative','full'),default='smoke')
    args=parser.parse_args()
    if args.command=='prepare':
        if not args.compiler_receipt: parser.error('--compiler-receipt required')
        prepare(args.compiler_receipt,args.output,args.resource_profile)
    elif args.command=='check':
        validate_capture(args.output,policy(),ROOT)
    else:
        if not args.prepared: parser.error('--prepared required')
        capture(args.prepared,args.output,args.level)


if __name__=='__main__':
    main()
