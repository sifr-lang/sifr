"""Separate instrumented allocation observations for registered generated programs."""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import sys
import tomllib

from allocation_instrumentation import FIELDS, MODULE, SCOPE, instrument, validate_counts
from allocation_recipe import copy_package, package_files, require_same_recipe
from benchmark_manifest import BenchmarkError
from compiler_lanes import validate_receipt
from generated_program_metrics import ROOT, POLICY, policy, require_clean_source, digest, dependencies
from program_artifact_identity import application_events
from program_evidence import producer_identity, target_cpu_generic, verify_preparation_events
from sifr_verify.process_execution import execute
from sifr_verify.execution_evidence import write_evidence
from sifr_verify.resource_admission import admit, discover, worker_limit

PROTOCOL = 'rust-globalalloc-main-v1'
ALLOCATION_POLICY = Path(__file__).parent/'data/generated_program_allocations.json'


def allocation_policy():
    value=json.loads(ALLOCATION_POLICY.read_text())
    expected={'schema_version':1,'owner':'compiler/generated-program-performance','protocol':PROTOCOL,
              'scope':SCOPE,'samples_per_case':1,'numeric_regression_qualification':False,
              'optimization':'release','target_cpu':'generic','instrumentation_sha256':digest(MODULE),
              'program_policy_sha256':digest(POLICY),'fields':list(FIELDS)}
    if any(type(value.get(key)) is not type(item) or value.get(key)!=item for key,item in expected.items()):
        raise BenchmarkError('allocation contract or instrumentation bytes changed without registration')
    return value


def load_prepared(path):
    prepared = json.loads(path.read_text())
    compiler_path = Path(prepared['compiler_receipt'])
    compiler = json.loads(compiler_path.read_text())
    validate_receipt(ROOT, compiler['lane'], compiler)
    contract = policy()
    allocation_policy()
    if (prepared['schema_version'] != 1 or prepared['kind'] != 'preparation-output'
            or prepared['runtime_assertions'] != 0 or prepared['optimization'] != 'release'
            or prepared['target_cpu'] != 'generic' or prepared['policy_sha256'] != digest(POLICY)
            or prepared['compiler_receipt_sha256'] != digest(compiler_path)
            or [row['id'] for row in prepared['programs']] != [case['id'] for case in contract['cases']]):
        raise BenchmarkError('allocation input is not the exact current generated program preparation')
    verify_preparation_events(path, prepared)
    for row, case in zip(prepared['programs'], contract['cases'], strict=True):
        if (digest(row['binary']) != row['binary_sha256'] or row['source_sha256'] != case['source_sha256']
                or digest(row['compiled_source_path']) != case['source_sha256']):
            raise BenchmarkError('allocation input program bytes differ')
    return prepared, contract


def checked(command, *, output, label, env):
    result = execute(command, cwd=output, env=env, deadline_seconds=2400, limit_bytes=16*1024**2)
    paths = {}
    for stream in ('stdout', 'stderr'):
        path = output/(label+'.'+stream)
        path.write_bytes(getattr(result, stream))
        paths[stream] = {'path':str(path),'sha256':digest(path)}
    record = {'argv':command,'cause':result.cause,'returncode':result.returncode,
              'truncated':result.truncated,'elapsed_seconds':result.elapsed_seconds,'output':paths}
    (output/(label+'.process.json')).write_text(json.dumps(record,indent=2)+'\n')
    if result.cause != 'exit' or result.returncode or result.truncated:
        raise BenchmarkError('allocation '+label+' process failed or is incomplete')
    return result, record


def collect(prepared_path, output):
    require_clean_source()
    prepared_path = prepared_path.resolve(strict=True)
    prepared, contract = load_prepared(prepared_path)
    if any(os.environ.get(key) for key in ('RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','RUSTC_WRAPPER',
                                          'RUSTC_WORKSPACE_WRAPPER','LD_PRELOAD','LD_LIBRARY_PATH')):
        raise BenchmarkError('unregistered allocation compiler or loader override')
    output = output.absolute()
    if output.is_relative_to(ROOT) or any(path.is_symlink() for path in (output,*output.parents)):
        raise BenchmarkError('allocation output must be a private canonical external directory')
    output.parent.mkdir(parents=True,exist_ok=True)
    resources = discover(disk_path=output.parent)
    admission = admit(resources,dict(disk_growth_bytes=1024**3,retained_copy_bytes=128*1024**2,
        disk_reserve_bytes=2*1024**3,memory_peak_bytes=4*1024**3,tmpfs_growth_bytes=0,memory_reserve_bytes=2*1024**3))
    output.mkdir(mode=0o700,exist_ok=False)
    producer = producer_identity(ROOT)
    report = {'schema_version':1,'protocol':PROTOCOL,'scope':SCOPE,'status':'incomplete',
              'numeric_regression_qualification':False,'samples_per_case':1,'allocation_policy_sha256':digest(ALLOCATION_POLICY),
              'prepared_path':str(prepared_path),'prepared_sha256':digest(prepared_path),
              'instrumentation_sha256':digest(MODULE),'producer':producer,'admission':admission,
              'started_utc':datetime.now(timezone.utc).isoformat(),'rows':[]}
    receipt = output/'receipt.json'
    state = output/'state.json'
    def save(): state.write_text(json.dumps(report,indent=2)+'\n')
    save()
    env = os.environ.copy()|{'CARGO_NET_OFFLINE':'true','CARGO_BUILD_JOBS':str(worker_limit(resources,2)),
          'RUSTFLAGS':'-C target-cpu=generic','SIFR_VERIFY_DISK_FLOOR_BYTES':str(max(3*1024**3,int(os.environ.get('SIFR_VERIFY_DISK_FLOOR_BYTES','0')))),
          'SIFR_VERIFY_DISK_FLOOR_PATH':os.environ.get('SIFR_VERIFY_DISK_FLOOR_PATH',str(output.parent))}
    try:
        for original, case in zip(prepared['programs'],contract['cases'],strict=True):
            directory = output/case['id'];directory.mkdir()
            manifest = Path(original['cargo_artifact']['manifest_path'])
            project = directory/'project';(project/'src').mkdir(parents=True)
            source = Path(original['cargo_artifact']['target']['src_path']).read_bytes()
            (directory/'original.rs').write_bytes(source)
            modified = instrument(source)
            (project/'src/main.rs').write_bytes(modified)
            copy_package(manifest.parent,project)
            declared = tomllib.loads((project/'Cargo.toml').read_text())
            if len(declared.get('bin',[])) != 1 or declared['bin'][0]['path'] != 'src/main.rs':
                raise BenchmarkError('allocation project requires its exact standalone main manifest')
            events = directory/'rustc-events.jsonl'
            wrapper = directory/'record-rustc'
            wrapper.write_text('#!'+sys.executable+'\n'+'''import json,os,sys
with open(os.environ['SIFR_ALLOCATION_RUSTC_EVENTS'],'a') as output:
    output.write(json.dumps(sys.argv[2:])+'\\n')
os.execv(sys.argv[1],sys.argv[1:])
''')
            wrapper.chmod(0o700)
            current = env|{'CARGO_TARGET_DIR':str(directory/'target'),'RUSTC_WRAPPER':str(wrapper),
                          'SIFR_ALLOCATION_RUSTC_EVENTS':str(events)}
            command = ['cargo','build','--locked','--offline','--release','--manifest-path',str(project/'Cargo.toml'),
                       '--message-format=json-render-diagnostics']
            built, build_record = checked(command,output=directory,label='build',env=current)
            cargo_rows = [json.loads(line) for line in built.stdout.decode().splitlines() if line.startswith('{')]
            target = declared['bin'][0]['name']
            binary = directory/'target/release'/target
            invocations = [json.loads(line) for line in events.read_text().splitlines()]
            artifact, invocation = application_events(cargo_rows,invocations,binary)
            if Path(artifact['manifest_path']) != project/'Cargo.toml' or Path(artifact['target']['src_path']) != project/'src/main.rs':
                raise BenchmarkError('allocation Cargo events identify another source project')
            require_same_recipe(original['rustc_command'],invocation)
            if artifact['profile']!=original['cargo_artifact']['profile']:
                raise BenchmarkError('instrumented Cargo profile differs from preparation')
            profile = artifact['profile']
            if (profile['opt_level']!='3' or profile['test'] or profile['debug_assertions']
                    or not profile['overflow_checks'] or not target_cpu_generic(invocation)):
                raise BenchmarkError('instrumented application differs from release/generic contract')
            counts = directory/'counts.json'
            measured, run_record = checked([str(binary)],output=directory,label='observation',
                                          env=current|{'SIFR_ALLOCATION_RECEIPT':str(counts)})
            if measured.stdout != case['expected_stdout'].encode() or measured.stderr:
                raise BenchmarkError('instrumented generated program violates its independent oracle')
            counters = validate_counts(json.loads(counts.read_text()))
            report['rows'].append({'id':case['id'],'original_binary_sha256':original['binary_sha256'],
                'original_rust_sha256':hashlib.sha256(source).hexdigest(),'instrumented_rust_sha256':digest(project/'src/main.rs'),
                'binary':str(binary),'binary_sha256':digest(binary),'cargo_artifact':artifact,'rustc_command':invocation,
                'rustc_events_sha256':digest(events),'runtime_dependencies':dependencies(binary),'counts':counters,
                'counts_sha256':digest(counts),'build_process':build_record,'observation_process':run_record})
            save()
        load_prepared(prepared_path)
        if digest(prepared_path)!=report['prepared_sha256'] or digest(MODULE)!=report['instrumentation_sha256'] or producer_identity(ROOT)!=producer:
            raise BenchmarkError('allocation producer/input bytes changed')
        require_clean_source()
        report.update(status='captured',finished_utc=datetime.now(timezone.utc).isoformat())
        candidate=output/'candidate.json'
        candidate.write_text(json.dumps(report,indent=2)+'\n')
        check(candidate)
        write_evidence(receipt,report)
        save()
    except BaseException as error:
        report.update(status='failed',failure=type(error).__name__+': '+str(error),finished_utc=datetime.now(timezone.utc).isoformat());save();raise
    return report


def check(path):
    require_clean_source()
    value = json.loads(path.read_text())
    prepared, contract = load_prepared(Path(value['prepared_path']))
    now = datetime.now(timezone.utc)
    start, finish = (datetime.fromisoformat(value[key]) for key in ('started_utc','finished_utc'))
    if (value['schema_version']!=1 or value['protocol']!=PROTOCOL or value['scope']!=SCOPE
            or value['status']!='captured' or value['numeric_regression_qualification'] is not False
            or value['allocation_policy_sha256']!=digest(ALLOCATION_POLICY) or value['samples_per_case']!=1 or value['prepared_sha256']!=digest(value['prepared_path'])
            or value['instrumentation_sha256']!=digest(MODULE) or value['producer']!=producer_identity(ROOT)
            or start.utcoffset() is None or finish.utcoffset() is None or not start<=finish<=now
            or (now-finish).total_seconds()>86400
            or [row['id'] for row in value['rows']] != [case['id'] for case in contract['cases']]):
        raise BenchmarkError('allocation observation identity/status/freshness differs')
    for row, original, case in zip(value['rows'],prepared['programs'],contract['cases'],strict=True):
        directory = path.parent/case['id'];project = directory/'project'
        source = (directory/'original.rs').read_bytes()
        if (hashlib.sha256(source).hexdigest()!=original['generated_rust_sha256']
                or row['original_rust_sha256']!=original['generated_rust_sha256']
                or (project/'src/main.rs').read_bytes()!=instrument(source)
                or row['instrumented_rust_sha256']!=digest(project/'src/main.rs')
                or row['original_binary_sha256']!=original['binary_sha256'] or digest(row['binary'])!=row['binary_sha256']):
            raise BenchmarkError('allocation source transformation/artifact differs')
        if package_files(project)!=package_files(Path(original['cargo_artifact']['manifest_path']).parent):
            raise BenchmarkError('allocation generated package inputs differ')
        expected_build=['cargo','build','--locked','--offline','--release','--manifest-path',str(project/'Cargo.toml'),
                        '--message-format=json-render-diagnostics']
        if row['build_process']['argv']!=expected_build or row['observation_process']['argv']!=[row['binary']]:
            raise BenchmarkError('allocation commands differ from the registered recipe')
        for label in ('build','observation'):
            process = row[label+'_process']
            if process['cause']!='exit' or process['returncode'] or process['truncated']:
                raise BenchmarkError('allocation native process did not complete')
            for stream in ('stdout','stderr'):
                if process['output'][stream]['sha256']!=digest(directory/(label+'.'+stream)):
                    raise BenchmarkError('allocation raw process output differs')
        if (directory/'observation.stdout').read_bytes()!=case['expected_stdout'].encode() or (directory/'observation.stderr').read_bytes():
            raise BenchmarkError('allocation independent program oracle differs')
        events = directory/'rustc-events.jsonl'
        if digest(events)!=row['rustc_events_sha256']: raise BenchmarkError('allocation rustc events differ')
        cargo_rows = [json.loads(line) for line in (directory/'build.stdout').read_text().splitlines() if line.startswith('{')]
        artifact, invocation = application_events(cargo_rows,[json.loads(line) for line in events.read_text().splitlines()],row['binary'])
        if Path(artifact['manifest_path']) != project/'Cargo.toml' or Path(artifact['target']['src_path']) != project/'src/main.rs':
            raise BenchmarkError('allocation Cargo events identify another source project')
        require_same_recipe(original['rustc_command'],invocation)
        if artifact['profile']!=original['cargo_artifact']['profile']:
            raise BenchmarkError('instrumented Cargo profile differs from preparation')
        profile = artifact['profile']
        if (artifact!=row['cargo_artifact'] or invocation!=row['rustc_command'] or not target_cpu_generic(invocation)
                or profile['opt_level']!='3' or profile['test'] or profile['debug_assertions'] or not profile['overflow_checks']
                or dependencies(row['binary'])!=row['runtime_dependencies']):
            raise BenchmarkError('allocation actual build profile/CPU/runtime differs')
        counts = directory/'counts.json'
        if digest(counts)!=row['counts_sha256'] or validate_counts(json.loads(counts.read_text()))!=row['counts']:
            raise BenchmarkError('allocation raw counters differ')
    return value


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command',choices=('collect','check'))
    parser.add_argument('--prepared',type=Path)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    if args.command=='collect':
        if args.prepared is None: parser.error('collect requires --prepared')
        collect(args.prepared,args.output)
    else: check(args.output)
