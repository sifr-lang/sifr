"""Explicit local reuse across audited non-measurement changes; never reseal data."""
from __future__ import annotations
import hashlib
import json
from pathlib import Path
import sys
import uuid

from .execution_evidence import write_evidence
from .errors import VerificationError
from .process_execution import execute
from .graph_retirement import plain_path

# This narrow recipe has no general docs/directory wildcard. Unknown changes
# require fresh qualification. These files are outside compiler, corpus and
# the audited measurement runtime/tooling closure.
EXCLUDED_PATHS=frozenset({
    'AGENTS.md', 'verification/README.md',
    'plans/issues/active/ad-hoc-validation-contracts-and-resource-aware-execution.md',
    '.github/workflows/published-native-qualification.yml',
    'verification/areas/sysroot_release/native_capacity.py',
    'verification/areas/sysroot_release/native_capacity_tests.py',
    'verification/runner/sifr_verify/cloud_receipt_reuse.py',
    'verification/runner/sifr_verify/cloud_receipt_reuse_tests.py',
    'verification/runner/sifr_verify/cloud_profile.py',
    'verification/runner/sifr_verify/selftest.py',
})


def digest(path):
    result=hashlib.sha256()
    with Path(path).open('rb') as stream:
        while block:=stream.read(1024**2):result.update(block)
    return result.hexdigest()


def command(argv,root,env,*,trim=True):
    result=execute(argv,cwd=root,env=env,deadline_seconds=30,limit_bytes=16*1024**2)
    if result.cause!='exit' or result.returncode or result.truncated:
        raise VerificationError('cloud reuse identity command did not complete')
    text=result.stdout.decode('utf-8',errors='strict')
    return text.strip() if trim else text


def live_inputs(root):
    """Bind actual runtime/corpus trees too, including Git-ignored source files."""
    inputs={}
    roots=('Cargo.toml','Cargo.lock','rust-toolchain.toml','.cargo','crates',
           'third_party','vendor','stdlib','demos','verification/areas/performance')
    def visit(path):
        relative=str(path.relative_to(root))
        if path.is_symlink():
            raise VerificationError('cloud reuse cannot audit a linked source input: '+relative)
        if path.is_dir():
            inputs[relative]='directory'
            for child in sorted(path.iterdir()):
                if child.name not in {'.git','target','__pycache__','.venv'}:visit(child)
        elif path.is_file():inputs[relative]=digest(path)
        elif not path.exists():inputs[relative]='absent'
        else:raise VerificationError('cloud reuse encountered an unknown source input')
    for relative in roots:visit(root/relative)
    for name in ('__init__.py','process_execution.py','process_supervisor.py','process_disk_budget.py'):
        visit(root/'verification/runner/sifr_verify'/name)
    return inputs


def equivalence(source,candidate,observed_commit,env):
    source,candidate=(Path(root).resolve(strict=True) for root in (source,candidate))
    def git(root,*args):return command(['git',*args],root,env)
    for root in (source,candidate):
        if git(root,'status','--porcelain','--untracked-files=all','--ignore-submodules=none'):
            raise VerificationError('cloud reuse requires clean source worktrees')
        # Sparse trees cannot establish the live closure of compiler inputs.
        if git(root,'config','--get','--default','false','core.sparseCheckout')=='true':
            raise VerificationError('cloud reuse requires full source worktrees')
    if source==candidate or git(source,'rev-parse','HEAD')!=observed_commit:
        raise VerificationError('cloud reuse observed source worktree differs')
    common=lambda root:Path(git(root,'rev-parse','--path-format=absolute','--git-common-dir')).resolve(strict=True)
    if common(source)!=common(candidate):
        raise VerificationError('cloud reuse requires the same owned Git repository')
    head=git(candidate,'rev-parse','HEAD')
    git(candidate,'merge-base','--is-ancestor',observed_commit,head)
    names=command(['git','diff','--name-only','-z',observed_commit,head],candidate,env,trim=False)
    changed=set(names.split('\0'))-{''}
    unknown=changed-EXCLUDED_PATHS
    if unknown:
        raise VerificationError('cloud reuse dependency closure changed: '+', '.join(sorted(unknown)))
    actual=live_inputs(source)
    if live_inputs(candidate)!=actual:
        raise VerificationError('cloud reuse live compiler/runtime/corpus inputs differ')
    live_digest=hashlib.sha256(json.dumps(actual,sort_keys=True).encode()).hexdigest()
    return {'observed_commit':observed_commit,'candidate_commit':head,
            'observed_worktree':str(source),'candidate_worktree':str(candidate),
            'excluded_changes':sorted(changed),'live_inputs_sha256':live_digest}


def consume(receipt,source,candidate,compiler,env,run_checker):
    """Original checker remains authoritative; bind its pass to current inputs."""
    receipt,source,candidate,compiler=(Path(path).resolve(strict=True) for path in
                                       (receipt,source,candidate,compiler))
    data=json.loads(receipt.read_text());endpoint=data['specification']['endpoints']['candidate']
    if Path(endpoint['cloud_repo']).resolve(strict=True)!=source:
        raise VerificationError('explicit cloud source differs from paired receipt')
    proof=equivalence(source,candidate,endpoint['cloud_source'],env)
    probe=("import json,sys; from pathlib import Path; "
           "sys.path.insert(0,str(Path.cwd()/'verification/areas/performance')); "
           "from cloud_worker import python_context; "
           "from benchmark_manifest import load_manifest,validate_manifest; "
           "print(json.dumps(python_context(Path.cwd(),validate_manifest(load_manifest("
           "Path.cwd()/'verification/areas/performance/data/benchmark_manifest.json')))))")
    context=json.loads(command([sys.executable,'-c',probe],candidate,env))
    if context!=endpoint['cloud_identity']['execution']['cloud_python_context']:
        raise VerificationError('current benchmark Python context differs from measured context')
    identity=json.loads(command([str(compiler),'--print','compiler-identity','--json'],candidate,env))
    if (identity!=endpoint['embedded_compatibility_identity']
            or identity.get('identity_kind')!='product'
            or len(identity.get('compiler_build_id',''))!=64):
        raise VerificationError('current compiler build identity differs from measured compiler')
    if digest(compiler)!=endpoint['artifact']['sha256']:
        raise VerificationError('current compiler bytes differ from measured compiler')
    before={'receipt_sha256':digest(receipt),'compiler_sha256':digest(compiler)}
    destination=plain_path(candidate,candidate/'target/validation_lane_reports/cloud-performance-reuse'/str(uuid.uuid4()))
    destination.mkdir(parents=True,exist_ok=False)
    record={'schema_version':1,'protocol':'local-cloud-non-measurement-reuse-v1',
            'claim':'checked-performance-reuse-observation','status':'incomplete',
            'runtime_assertions_executed':0,'proof':proof,'inputs':before,
            'recipe_sha256':digest(Path(__file__)),'compiler_identity':identity,
            'measured_compiler_sha256':endpoint['artifact']['sha256']}
    write_evidence(destination/'started.json',record)
    try:
        checker=source/'verification/areas/performance/cloud_benchmarks.py'
        # All measurement Python, data, runtime, compiler/stdlib and submodule
        # inputs are unchanged by the tree proof; no old result is rewritten.
        run_checker([sys.executable,str(checker),'check','--receipt',str(receipt)],env=env)
        if equivalence(source,candidate,endpoint['cloud_source'],env)!=proof:
            raise VerificationError('cloud reuse inputs changed during checking')
        if json.loads(command([sys.executable,'-c',probe],candidate,env))!=context:
            raise VerificationError('cloud reuse Python context changed during checking')
        if before!={'receipt_sha256':digest(receipt),'compiler_sha256':digest(compiler)}:
            raise VerificationError('cloud reuse artifact or receipt changed during checking')
        record['status']='passed'
        write_evidence(destination/'receipt.json',record)
        print('Cloud performance reused: observed='+proof['observed_commit']+' candidate='+proof['candidate_commit'])
        return record
    except BaseException as error:
        record.update(status='failed',failure=type(error).__name__+': '+str(error))
        write_evidence(destination/'failure.json',record)
        raise
