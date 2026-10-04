"""One separately admitted published ARM runtime observation; zero qualification."""
import argparse
import ast
from dataclasses import asdict
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT/'verification/runner'))
sys.path.insert(0, str(ROOT/'scripts/distribution'))
from sifr_verify.process_execution import execute, deadline_environment
from sifr_verify.resource_admission import Resources, admit
from native_capacity import resources
from native_capacity_diagnostics import digest
from native_preparation_storage import parent_temporary
import native_runtime_storage as storage
from native_runtime_observer import Observer, SPEC, replay
import native_dependency_preparation as dependencies
from published_predecessor import POLICY, select, fetch_archive, unpack
from published_installation import payload, verify_installed
from metadata_artifact import require_native_binary

TARGET = 'aarch64-apple-darwin'
SOURCE = 'from sifr.math import isqrt\n\ndef main():\n    print(isqrt(81))\n'
USER_FILES = ('sifr.toml', 'Cargo.toml', 'src/main.sifr', 'src/lib.rs', 'user-state.json')
COLD_PATHS = ('target', 'sifr-cache', 'bridge-cache', 'clang-cache')
PROTOCOL = 'published-arm-runtime-diagnostic-v1'
TOOLCHAIN = '1.98.1'
DEADLINE = 1800
PHASES = ('toolchain', 'tool-identity', 'sdk', 'linker', 'acquire', 'dependency-project',
          'fetch', 'fetch-offline', 'version', 'init', 'source', 'run', 'verify')
DEPENDENCY_ROOTS = ('verification/areas/sysroot_release', 'verification/runner/sifr_verify',
                    'scripts/distribution')
FIXED_INPUTS = ('verification/areas/sysroot_release/published_predecessors.json',
                'verification/areas/sysroot_release/package_build.py',
                'verification/pyproject.toml', 'verification/uv.lock', 'rust-toolchain.toml',
                '.github/workflows/native-runtime-diagnostic.yml',
                '.github/workflows/published-native-qualification.yml')


def source_identity(root):
    def git(*args):
        result = subprocess.run(['git', '--no-optional-locks', '-C', str(root), *args],
                                capture_output=True, timeout=30, check=True)
        return result.stdout.decode().strip()
    if git('status', '--porcelain', '--untracked-files=all', '--ignore-submodules=none'):
        raise ValueError('diagnostic requires clean committed source')
    paths = set(FIXED_INPUTS)
    for directory in DEPENDENCY_ROOTS:
        paths.update(name for name in git('ls-files', directory).splitlines() if name.endswith('.py'))
    return {'commit': git('rev-parse', 'HEAD'),
            'producer_sha256': {name: digest(root/name) for name in sorted(paths)}}


def version(root):
    tree = ast.parse((root/'verification/areas/sysroot_release/package_build.py').read_text())
    values = [node.value.value for node in tree.body if isinstance(node, ast.Assign)
              and any(isinstance(target, ast.Name) and target.id == 'RELEASE_VERSION' for target in node.targets)
              and isinstance(node.value, ast.Constant)]
    if len(values) != 1 or not isinstance(values[0], str):
        raise ValueError('canonical prospective version unavailable')
    return values[0]


def python_identity():
    # Invocation through the venv symlink selects its site-packages. Resolving it
    # is correct for byte identity but wrong for spawning the canonical context.
    invocation = Path(sys.executable).absolute()
    binary = invocation.resolve(strict=True)
    return {'path': str(invocation), 'resolved_path': str(binary), 'sha256': digest(binary),
            'prefix': sys.prefix, 'base_prefix': sys.base_prefix}


def worker(mode, output):
    """No child processes here: the outer executor owns each workload session."""
    policy = json.loads(POLICY.read_text())
    candidate_version = version(ROOT)
    package = output/'package'
    asset = select(policy, TARGET, candidate_version)
    archive = output/asset['name']
    if mode == 'acquire':
        fetch_archive(asset, archive)
        _, rows = payload(policy, archive, TARGET, candidate_version)
        decoded = unpack(archive, package)
        verify_installed(package, rows)
        if require_native_binary(package/'bin/sifr', False) != TARGET:
            raise ValueError('published executable is not ARM Darwin')
        (output/'evidence/package.json').write_text(json.dumps(
            {'asset': asset, 'publication_source': policy['source_commit'], 'rows': rows,
             'decoded': decoded, 'binary_sha256': digest(package/'bin/sifr')}, sort_keys=True)+'\n')
        for name in dependencies.INPUTS:
            destination = output/'evidence/package-inputs'/name
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(package/name, destination)
    elif mode == 'dependency-project':
        project = output/'dependency'
        project.mkdir(mode=0o700)
        (project/'src').mkdir()
        (project/'src/lib.rs').write_text('')
        (project/'Cargo.toml').write_text(dependencies.manifest(package))
        shutil.copyfile(package/'Cargo.lock', project/'Cargo.lock')
        (output/'evidence/dependency-inputs.json').write_text(json.dumps(dependencies.inputs(package), sort_keys=True)+'\n')
    elif mode == 'source':
        (output/'project/src/main.sifr').write_text(SOURCE)
        (output/'project/user-state.json').write_text('{"user_owned":true,"revision":1}\n')
        (output/'evidence/user-inputs.json').write_text(json.dumps(user_inputs(output/'project'), sort_keys=True)+'\n')
        for name in USER_FILES:
            destination = output/'evidence/user-inputs'/name
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(output/'project'/name, destination)
        cold = {name: sorted(str(path.relative_to(output/name)) for path in (output/name).rglob('*'))
                for name in COLD_PATHS}
        if any(cold.values()):
            raise ValueError('runtime diagnostic requires empty generated-code caches')
        (output/'evidence/cold-cache.json').write_text(json.dumps(cold, sort_keys=True)+'\n')
    elif mode == 'verify':
        record = json.loads((output/'evidence/package.json').read_text())
        _, rows = payload(policy, archive, TARGET, candidate_version)
        if rows != record['rows'] or digest(package/'bin/sifr') != record['binary_sha256']:
            raise ValueError('published package identity changed')
        verify_installed(package, rows)
        if dependencies.inputs(package) != json.loads((output/'evidence/dependency-inputs.json').read_text()):
            raise ValueError('published runtime inputs changed')
        if user_inputs(output/'project') != json.loads((output/'evidence/user-inputs.json').read_text()):
            raise ValueError('persisted user input changed')
    else:
        raise ValueError('unknown diagnostic worker')


def user_inputs(project):
    return {name: digest(project/name) for name in USER_FILES}


def command_plan(output, rustup, python):
    script = str(ROOT/'verification/areas/sysroot_release/native_runtime_diagnostic.py')
    cargo = output/'rustup-home/toolchains'/f'{TOOLCHAIN}-{TARGET}'/'bin/cargo'
    binary = output/'package/bin/sifr'
    def work(name):
        return [python, '-B', script, 'worker', '--output', str(output), '--worker', name]
    return {
        'toolchain': [rustup, 'toolchain', 'install', TOOLCHAIN, '--profile', 'minimal', '--no-self-update'],
        'tool-identity': [str(cargo.with_name('rustc')), '-vV'],
        'sdk': ['/usr/bin/xcrun', '--show-sdk-path'],
        'linker': ['/usr/bin/clang', '--version'],
        'acquire': work('acquire'), 'dependency-project': work('dependency-project'),
        'fetch': [str(cargo), *dependencies.commands(output/'dependency')[0][1:]],
        'fetch-offline': [str(cargo), *dependencies.commands(output/'dependency')[1][1:]],
        'version': [str(binary), '--version'],
        'init': [str(binary), 'init', '--bin', '--name', 'persisted_upgrade', str(output/'project')],
        'source': work('source'),
        'run': [str(binary), 'run', 'src/main.sifr', '--offline', '--quiet'],
        'verify': work('verify')}


def file_evidence(directory):
    if any(path.is_symlink() or not (path.is_file() or path.is_dir()) for path in directory.rglob('*')):
        raise ValueError('diagnostic evidence must contain regular owned bytes')
    return {str(path.relative_to(directory)): {'sha256': digest(path), 'size': path.stat().st_size}
            for path in sorted(directory.rglob('*')) if path.is_file() and path.name not in ('state.json', 'receipt.json')}


def recorded_environment(env, output):
    # Never serialize inherited CI credentials or unrelated environment values.
    keys = set(storage.bindings(output)) | {'PATH', 'SIFR_VERIFY_SAFETY_DEADLINE_MONOTONIC'}
    return {key: env[key] for key in sorted(keys) if key in env}


def observe(root, output):
    if (platform.system() != 'Darwin' or platform.machine() != 'arm64'
            or os.environ.get('SIFR_NATIVE_HOST_KIND') != 'dedicated-darwin'):
        raise ValueError('diagnostic requires dedicated native standard ARM Darwin')
    root = root.resolve(strict=True)
    output = output.absolute()
    if output.is_relative_to(root) or any(path.is_symlink() for path in (output, *output.parents)):
        raise ValueError('diagnostic requires private external output')
    identity = source_identity(root)
    if identity != source_identity(ROOT):
        raise ValueError('diagnostic source tooling differs')
    policy = json.loads(POLICY.read_text())
    asset = select(policy, TARGET, version(root))
    interpreter = python_identity()
    python = interpreter['path']
    selected = shutil.which('rustup')
    if not selected:
        raise ValueError('rustup unavailable')
    rustup = str(Path(selected).resolve(strict=True))
    env, proof = storage.prepare(output, os.environ)
    # Explicit actual binaries avoid implicit rustup defaults in generated builds.
    toolbin = output/'rustup-home/toolchains'/f'{TOOLCHAIN}-{TARGET}'/'bin'
    plan = command_plan(output, rustup, python)
    report = {'schema_version': 1, 'protocol': PROTOCOL, 'runtime_assertions': 0,
              'native_qualification': False, 'status': 'incomplete', 'target': TARGET,
              'source': identity, 'publication': policy, 'asset': asset, 'version': version(root),
              'workflow': {name: os.environ[name] for name in ('GITHUB_REPOSITORY', 'GITHUB_RUN_ID',
                           'GITHUB_RUN_ATTEMPT', 'GITHUB_SHA', 'GITHUB_WORKFLOW_REF', 'GITHUB_WORKFLOW_SHA',
                           'RUNNER_OS', 'RUNNER_ARCH') if name in os.environ},
              'specification': SPEC, 'requirements': storage.requirements(asset),
              'original_output': str(output), 'source_root': str(root), 'storage': proof,
              'inherited_disk_floor_bytes': int(env['SIFR_VERIFY_DISK_FLOOR_BYTES']),
              'tools': {'python': interpreter,
                        'rustup': {'path': rustup, 'sha256': digest(rustup)}},
              'commands': [], 'started': datetime.now(timezone.utc).isoformat()}
    report['tools'].update({name: {'path': path, 'sha256': digest(path)} for name, path in
                           (('ps', '/bin/ps'), ('vm_stat', '/usr/bin/vm_stat'),
                            ('xcrun', '/usr/bin/xcrun'), ('clang', '/usr/bin/clang'))})
    evidence = output/'evidence'
    def save():
        report['files'] = file_evidence(evidence)
        (evidence/'state.json').write_text(json.dumps(report, indent=2)+'\n')
    save()
    try:
        env, deadline = deadline_environment(env, DEADLINE)
        report['deadline'] = deadline
        if report['workflow'] and report['workflow'].get('GITHUB_SHA') != identity['commit']:
            raise ValueError('workflow candidate source differs')
        capacity = resources(output)
        report['capacity'] = asdict(capacity)
        report['admission'] = admit(capacity, report['requirements'])
        floor = max(report['inherited_disk_floor_bytes'], capacity.disk_available_bytes -
                    report['requirements']['disk_growth_bytes'] - report['requirements']['retained_copy_bytes'])
        env['SIFR_VERIFY_DISK_FLOOR_BYTES'] = str(floor)
        with parent_temporary(proof):
            for phase in PHASES:
                # Recheck memory and immutable physical write-root authority before each workload.
                storage.check(proof, output, env)
                fresh = resources(output)
                memory_only = dict(report['requirements'], disk_growth_bytes=0, retained_copy_bytes=0)
                admission = admit(fresh, memory_only)
                observer = Observer(evidence/'samples.jsonl', phase=phase, total=capacity.memory_limit_bytes)
                if observer.poll(os.getpid(), __import__('time').monotonic()) is not None:
                    raise ValueError('diagnostic observer preflight refused '+phase)
                cwd = output/'project' if phase == 'run' else output
                child_env = env | {'CARGO_NET_OFFLINE': 'false' if phase in ('toolchain', 'fetch') else 'true',
                                  'SIFR_VERIFY_SAFETY_DEADLINE_MONOTONIC': repr(deadline)}
                before_lock = digest(output/'dependency/Cargo.lock') if phase == 'fetch-offline' else None
                row = {'id': phase, 'argv': plan[phase], 'cwd': str(cwd),
                       'environment': recorded_environment(child_env, output),
                       'admission': admission, 'status': 'incomplete'}
                report['commands'].append(row); save()
                result = execute(plan[phase], cwd=cwd, env=child_env, deadline_seconds=DEADLINE,
                                 limit_bytes=4*1024**2, observer=observer.poll)
                for stream in ('stdout', 'stderr'):
                    (evidence/(phase+'.'+stream)).write_bytes(getattr(result, stream))
                row.update(status='finished', cause=result.cause, returncode=result.returncode,
                           truncated=result.truncated, elapsed_seconds=result.elapsed_seconds,
                           observation=observer.summary())
                if phase == 'fetch-offline':
                    row['lock_sha256'] = digest(output/'dependency/Cargo.lock')
                    if row['lock_sha256'] != before_lock:
                        raise ValueError('offline fetch changed prepared lock')
                    shutil.copyfile(output/'dependency/Cargo.toml', evidence/'dependency.Cargo.toml')
                    shutil.copyfile(output/'dependency/Cargo.lock', evidence/'dependency.Cargo.lock')
                save()
                if result.cause != 'exit' or result.returncode or result.truncated:
                    raise ValueError('diagnostic phase stopped: '+phase)
                if phase == 'tool-identity':
                    if ('release: '+TOOLCHAIN not in result.stdout.decode().splitlines()
                            or 'host: '+TARGET not in result.stdout.decode().splitlines()):
                        raise ValueError('diagnostic toolchain differs')
                    report['tools'].update({name: {'path': str(toolbin/name), 'sha256': digest(toolbin/name)}
                                            for name in ('cargo', 'rustc')})
                if phase == 'version' and result.stdout.strip() != ('sifr '+policy['version']).encode():
                    raise ValueError('published version differs')
                if phase == 'run' and result.stdout.strip() != b'9':
                    raise ValueError('diagnostic program output differs')
        if source_identity(root) != identity:
            raise ValueError('diagnostic source changed')
        if any(digest(tool['path']) != tool['sha256'] for tool in report['tools'].values()):
            raise ValueError('diagnostic selected tool bytes changed')
        storage.check(proof, output, env)
        report['status'] = 'observed'
    except Exception as error:
        report.update(status='stopped', failure=type(error).__name__+': '+str(error))
        raise
    finally:
        report['finished'] = datetime.now(timezone.utc).isoformat()
        save()
    check_retained(evidence, identity['commit'])
    (evidence/'receipt.json').write_text(json.dumps(report, indent=2)+'\n')
    return report


def check_retained(evidence, commit):
    evidence = evidence.resolve(strict=True)
    report = json.loads((evidence/'state.json').read_text())
    if (report.get('protocol') != PROTOCOL or report.get('schema_version') != 1
            or type(report.get('runtime_assertions')) is not int or report['runtime_assertions'] != 0
            or report.get('native_qualification') is not False or report.get('target') != TARGET
            or report.get('source', {}).get('commit') != commit or not re.fullmatch('[0-9a-f]{40}', commit)
            or report.get('specification') != SPEC or report.get('status') not in ('observed', 'stopped')
            or not report.get('finished')):
        raise ValueError('diagnostic identity or claim differs')
    policy = json.loads(POLICY.read_text())
    if report['source'] != source_identity(ROOT):
        raise ValueError('retained diagnostic requires exact committed producer source authority')
    if report.get('workflow') and report['workflow'].get('GITHUB_SHA') != commit:
        raise ValueError('retained workflow source differs')
    if report['publication'] != policy or report['asset'] != select(policy, TARGET, report['version']):
        raise ValueError('diagnostic publication differs')
    if report['requirements'] != storage.requirements(report['asset']) or report['files'] != file_evidence(evidence):
        raise ValueError('diagnostic allocation or raw bytes differ')
    if 'admission' in report and report['admission'] != admit(Resources(**report['capacity']), report['requirements']):
        raise ValueError('diagnostic initial admission differs')
    if report['status'] == 'observed' and 'admission' not in report:
        raise ValueError('diagnostic initial admission missing')
    if (evidence/'receipt.json').exists() and json.loads((evidence/'receipt.json').read_text()) != report:
        raise ValueError('diagnostic receipt differs')
    original = Path(report['original_output'])
    if not original.is_absolute() or report['storage']['environment'] != storage.bindings(original):
        raise ValueError('diagnostic storage/environment identity differs')
    if set(report['storage']['paths']) != {'.', *storage.NAMES}:
        raise ValueError('diagnostic storage path inventory differs')
    if 'capacity' in report:
        capacity = Resources(**report['capacity'])
        if (capacity.disk_memory_backed is not False
                or capacity.diagnostics.get('capacity_authority') != 'declared-dedicated-darwin-vm-stat'
                or any(row['storage'] != storage.topology(capacity.diagnostics)
                       for row in report['storage']['paths'].values())):
            raise ValueError('diagnostic physical topology differs')
    plan = command_plan(original, report['tools']['rustup']['path'], report['tools']['python']['path'])
    # command_plan's script path is the original committed checkout, not this audit host.
    for argv in plan.values():
        for index, value in enumerate(argv):
            if value == str(ROOT/'verification/areas/sysroot_release/native_runtime_diagnostic.py'):
                argv[index] = str(Path(report['source_root'])/'verification/areas/sysroot_release/native_runtime_diagnostic.py')
    phases = [row['id'] for row in report['commands']]
    if phases != list(PHASES[:len(phases)]) or (report['status'] == 'observed' and phases != list(PHASES)):
        raise ValueError('diagnostic command sequence differs')
    samples = replay(evidence/'samples.jsonl', report['admission']['resources']['memory_limit_bytes']) if (evidence/'samples.jsonl').exists() else {}
    if set(samples)-set(phases):
        # A preflight stop may occur before that command is appended.
        following = PHASES[len(phases)] if len(phases) < len(PHASES) else None
        if set(samples)-set(phases) != {following} or report['status'] != 'stopped':
            raise ValueError('unexpected observer phase')
    for row in report['commands']:
        phase = row['id']
        if row['argv'] != plan[phase] or row['cwd'] != str(original/'project' if phase == 'run' else original):
            raise ValueError('diagnostic command differs')
        if row['admission'] != admit(Resources(**row['admission']['resources']),
                                      dict(report['requirements'], disk_growth_bytes=0, retained_copy_bytes=0)):
            raise ValueError('diagnostic phase admission differs')
        desired = storage.bindings(original)
        if (type(report['inherited_disk_floor_bytes']) is not int
                or report['inherited_disk_floor_bytes'] < 8*storage.GIB):
            raise ValueError('invalid inherited diagnostic disk floor')
        floor = max(report['inherited_disk_floor_bytes'], report['capacity']['disk_available_bytes'] -
                    report['requirements']['disk_growth_bytes'] - report['requirements']['retained_copy_bytes'])
        desired['SIFR_VERIFY_DISK_FLOOR_BYTES'] = str(floor)
        desired['CARGO_NET_OFFLINE'] = 'false' if phase in ('toolchain', 'fetch') else 'true'
        desired['SIFR_VERIFY_SAFETY_DEADLINE_MONOTONIC'] = repr(report['deadline'])
        if any(row['environment'].get(key) != value for key, value in desired.items()):
            raise ValueError('diagnostic command environment differs')
        if row['status'] == 'finished':
            actual = samples.get(phase)
            if actual != {key: row['observation'][key] for key in ('samples', 'sampled_maximum_rss_bytes', 'stop')}:
                raise ValueError('diagnostic observation summary differs')
            if actual['stop'] is not None and row['cause'] != actual['stop']:
                raise ValueError('diagnostic observer stop differs from process outcome')
            if report['status'] == 'observed' and (row['cause'] != 'exit' or type(row['returncode']) is not int
                    or row['returncode'] != 0 or row['truncated'] is not False or actual['samples'] <= 0):
                raise ValueError('completed diagnostic contains failed command')
        elif report['status'] == 'observed':
            raise ValueError('diagnostic command incomplete')
    if report['status'] == 'observed':
        required_files = {phase+'.'+stream for phase in PHASES for stream in ('stdout', 'stderr')}
        required_files.update({'package.json', 'dependency-inputs.json', 'user-inputs.json',
                               'dependency.Cargo.toml', 'dependency.Cargo.lock', 'samples.jsonl', 'cold-cache.json'})
        required_files.update('package-inputs/'+name for name in dependencies.INPUTS)
        required_files.update('user-inputs/'+name for name in USER_FILES)
        if set(report['files']) != required_files:
            raise ValueError('diagnostic retained inventory differs')
        rust = (evidence/'tool-identity.stdout').read_text()
        if 'release: '+TOOLCHAIN not in rust.splitlines() or 'host: '+TARGET not in rust.splitlines():
            raise ValueError('retained Rust identity differs')
        if not (evidence/'sdk.stdout').read_text().strip().startswith('/') or not (evidence/'linker.stdout').read_text().strip():
            raise ValueError('retained Apple tools unavailable')
        package = json.loads((evidence/'package.json').read_text())
        if package['asset'] != report['asset'] or package['publication_source'] != policy['source_commit']:
            raise ValueError('retained package publication differs')
        package_files = {row['path']: row['sha256'] for row in package['rows']}
        if len(package_files) != len(package['rows']) or package_files.get('bin/sifr') != package['binary_sha256']:
            raise ValueError('retained package inventory differs')
        if json.loads((evidence/'dependency-inputs.json').read_text()) != {name: package_files[name] for name in dependencies.INPUTS}:
            raise ValueError('retained published dependency identity differs')
        retained_package = evidence/'package-inputs'
        if dependencies.inputs(retained_package) != {name: package_files[name] for name in dependencies.INPUTS}:
            raise ValueError('retained package input bytes differ')
        expected_manifest = dependencies.manifest(retained_package).replace(
            json.dumps(str(retained_package/'crates/sifr_stdlib')), json.dumps(str(original/'package/crates/sifr_stdlib')))
        if (evidence/'dependency.Cargo.toml').read_text() != expected_manifest:
            raise ValueError('retained dependency feature selection differs')
        if (user_inputs(evidence/'user-inputs') != json.loads((evidence/'user-inputs.json').read_text())
                or (evidence/'user-inputs/src/main.sifr').read_text() != SOURCE
                or json.loads((evidence/'cold-cache.json').read_text()) != {name: [] for name in COLD_PATHS}):
            raise ValueError('retained user workload or cold cache identity differs')
        if (evidence/'run.stdout').read_bytes().strip() != b'9':
            raise ValueError('diagnostic output differs')
        if (evidence/'version.stdout').read_bytes().strip() != ('sifr '+policy['version']).encode():
            raise ValueError('diagnostic binary version differs')
        if digest(evidence/'dependency.Cargo.lock') != next(row['lock_sha256'] for row in report['commands'] if row['id'] == 'fetch-offline'):
            raise ValueError('dependency lock evidence differs')
    return report


def check(evidence, root=ROOT):
    identity = source_identity(root)
    report = check_retained(evidence, identity['commit'])
    if report['source'] != identity:
        raise ValueError('diagnostic live producer identity differs')
    output = Path(report['original_output'])
    if evidence.resolve() != output/'evidence':
        raise ValueError('live diagnostic checker requires original paths')
    storage.check(report['storage'], output, storage.environment(output, os.environ))
    if report['tools']['python'] != python_identity() or any(
            digest(tool['path']) != tool['sha256'] for tool in report['tools'].values()):
        raise ValueError('live diagnostic tool identities differ')
    return report


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('mode', choices=('observe', 'check', 'check-retained', 'worker'))
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--commit')
    parser.add_argument('--worker', choices=('acquire', 'dependency-project', 'source', 'verify'))
    args = parser.parse_args()
    if args.mode == 'worker':
        worker(args.worker, args.output)
    else:
        result = (observe(ROOT, args.output) if args.mode == 'observe' else
                  check(args.output/'evidence') if args.mode == 'check' else
                  check_retained(args.output, args.commit or ''))
        print(json.dumps({'protocol': PROTOCOL, 'status': result['status'],
                          'runtime_assertions': 0, 'native_qualification': False}))
