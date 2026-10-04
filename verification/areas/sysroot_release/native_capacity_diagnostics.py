"""Bounded Darwin capacity facts; zero builds or native qualification claims."""
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import platform
import plistlib
import re
import shutil
import sys

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / 'verification/runner'))
from sifr_verify.process_execution import execute
from native_capacity import apfs_store_devices, known_apfs_store

PRODUCERS = ('verification/areas/sysroot_release/native_capacity_diagnostics.py',
             'verification/areas/sysroot_release/native_capacity.py',
             'verification/runner/sifr_verify/process_execution.py',
             'verification/runner/sifr_verify/process_supervisor.py',
             'verification/runner/sifr_verify/process_disk_budget.py')
TARGETS = {'arm64': 'aarch64-apple-darwin', 'x86_64': 'x86_64-apple-darwin'}
FIXED = [('host', ['uname', '-sm']), ('version', ['sw_vers']),
         ('memory-total', ['sysctl', '-n', 'hw.memsize']),
         ('cpu', ['sysctl', '-n', 'hw.logicalcpu']), ('pages', ['vm_stat']),
         ('swap', ['sysctl', 'vm.swapusage']), ('pressure', ['memory_pressure', '-Q'])]


def digest(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def selected_executable(name):
    selected = shutil.which(name)
    if selected is None:
        raise ValueError('capacity tool unavailable: ' + name)
    path = Path(selected).resolve(strict=True)
    return {'path': str(path), 'sha256': digest(path)}


def source_identity(root):
    def git(*args):
        value = execute(['git', *args], cwd=root, deadline_seconds=30, limit_bytes=1024**2)
        if value.cause != 'exit' or value.returncode or value.truncated:
            raise ValueError('diagnostic source identity unavailable')
        return value.stdout.decode('utf-8', errors='strict').strip()
    if git('status', '--porcelain', '--untracked-files=all', '--ignore-submodules=none'):
        raise ValueError('diagnostics require a clean committed source')
    return {'commit': git('rev-parse', 'HEAD'),
            'producer_sha256': {name: digest(root / name) for name in PRODUCERS}}


def parse_pages(text):
    match = re.search(r'page size of (\d+) bytes', text)
    if match is None:
        raise ValueError('Darwin page size unavailable')
    page = int(match[1]); rows = {}
    for line in text.splitlines()[1:]:
        parsed = re.fullmatch(r'([^:]+):\s+(\d+)\.', line)
        if parsed:
            if parsed[1] in rows:
                raise ValueError('duplicate Darwin page counter')
            rows[parsed[1]] = int(parsed[2])
    required = ('Pages free', 'Pages inactive', 'Pages speculative',
                'Pages occupied by compressor', 'Swapins', 'Swapouts')
    if page <= 0 or any(name not in rows for name in required):
        raise ValueError('incomplete Darwin page observations')
    return {'available_bytes': page * sum(rows[n] for n in required[:3]),
            'compressed_physical_bytes': page * rows[required[3]],
            'swapins': rows['Swapins'], 'swapouts': rows['Swapouts'],
            'page_size': page}


def interpret(rows, output, *, original_output=None):
    """Recompute from retained bounded raw bytes; never infer a build budget."""
    by_id = {row['id']: row for row in rows}
    if len(by_id) != len(rows):
        raise ValueError('duplicate diagnostic commands')
    def raw(name):
        row = by_id[name]
        if row['cause'] != 'exit' or row['returncode'] != 0 or row['truncated']:
            raise ValueError('required capacity command failed: ' + name)
        tool = row['executable']
        if (not Path(tool['path']).is_absolute() or Path(tool['path']).name != row['argv'][0]
                or not re.fullmatch(r'[0-9a-f]{64}', tool['sha256'])
                or row['actual_argv'] != [tool['path'], *row['argv'][1:]]):
            raise ValueError('diagnostic executable identity differs')
        for stream in ('stdout', 'stderr'):
            item = row['output'][stream]
            if item['file'] != name + '.' + stream:
                raise ValueError('diagnostic raw path differs')
            path = output / item['file']
            if path.is_symlink() or digest(path) != item['sha256']:
                raise ValueError('diagnostic raw bytes changed')
        return (output / (name + '.stdout')).read_bytes()
    def text(name):
        return raw(name).decode('utf-8', errors='strict')
    host = text('host').strip().split()
    if len(host) != 2 or host[0] != 'Darwin' or host[1] not in TARGETS:
        raise ValueError('unsupported diagnostic host')
    total = int(text('memory-total').strip()); cpu = int(text('cpu').strip())
    pages = parse_pages(text('pages'))
    if total <= 0 or cpu <= 0 or not 0 <= pages['available_bytes'] <= total:
        raise ValueError('contradictory capacity observation')
    df = text('disk').splitlines()
    if len(df) != 2 or len(df[1].split()) < 6:
        raise ValueError('diagnostic disk observation unavailable')
    fields = df[1].split(); device = fields[0]
    if not re.fullmatch(r'/dev/disk[0-9]+(?:s[0-9]+)*', device):
        raise ValueError('unknown diagnostic disk device')
    free = int(fields[3]) * 1024
    if free <= 0:
        raise ValueError('invalid diagnostic disk capacity')
    volume = plistlib.loads(raw('volume'))
    if volume.get('DeviceNode') != device or volume.get('FilesystemType') != 'apfs':
        raise ValueError('diagnostics require identified physical APFS storage')
    devices = apfs_store_devices(volume)
    if len(devices) > 16:
        raise ValueError('diagnostic backing-store inventory exceeds bound')
    for index, store in enumerate(devices):
        if not known_apfs_store(plistlib.loads(raw('store-' + str(index))), store):
            raise ValueError('unknown diagnostic backing-store authority')
    expected = [('disk', ['df', '-k', '-P', str(original_output or output)]),
                ('volume', ['diskutil', 'info', '-plist', device]),
                *[('store-' + str(i), ['diskutil', 'info', '-plist', d]) for i, d in enumerate(devices)],
                *FIXED]
    if [(row['id'], row['argv']) for row in rows] != expected:
        raise ValueError('capacity diagnostic command inventory differs')
    # These remain raw observations. A single swap/pressure snapshot does not
    # establish absence of pressure or a whole-process-tree memory bound.
    for name in ('version', 'swap', 'pressure'):
        if not text(name).strip():
            raise ValueError('required capacity observation empty: ' + name)
    return {'target': TARGETS[host[1]], 'memory_total_bytes': total,
            'logical_cpus': cpu, 'disk_free_bytes': free, 'memory': pages,
            'storage_memory_backed': False, 'apfs_backing_devices': devices}


def observe(root, output, *, target):
    root = Path(root).resolve(strict=True); output = Path(output).absolute()
    if platform.system() != 'Darwin' or os.environ.get('SIFR_NATIVE_HOST_KIND') != 'dedicated-darwin':
        raise ValueError('capacity diagnostics require an explicitly dedicated Darwin host')
    if target not in TARGETS.values():
        raise ValueError('unsupported expected diagnostic target')
    if output.is_relative_to(root) or any(p.is_symlink() for p in (output, *output.parents)):
        raise ValueError('capacity diagnostics require private canonical external output')
    identity = source_identity(root)
    if identity['producer_sha256'] != {name: digest(ROOT / name) for name in PRODUCERS}:
        raise ValueError('diagnostic tooling differs from source')
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    report = {'schema_version': 1, 'protocol': 'darwin-capacity-observation-v1',
              'claim': 'read-only-capacity-observation', 'status': 'incomplete',
              'runtime_assertions': 0, 'native_qualification': False,
              'expected_target': target,
              'source': identity, 'original_output': str(output),
              'python': {'version': platform.python_version(), 'sha256': digest(sys.executable)},
              'started_utc': datetime.now(timezone.utc).isoformat(), 'commands': []}
    def save():
        (output / 'state.json').write_text(json.dumps(report, indent=2) + '\n')
    save()
    def run(name, argv):
        tool = selected_executable(argv[0]); actual = [tool['path'], *argv[1:]]
        value = execute(actual, cwd=output, deadline_seconds=30, limit_bytes=1024**2)
        hashes = {}
        for stream in ('stdout', 'stderr'):
            path = output / (name + '.' + stream)
            path.write_bytes(getattr(value, stream))
            hashes[stream] = {'file': path.name, 'sha256': digest(path)}
        report['commands'].append({'id': name, 'argv': argv, 'cause': value.cause,
            'actual_argv': actual, 'executable': tool,
            'returncode': value.returncode, 'truncated': value.truncated,
            'elapsed_seconds': value.elapsed_seconds, 'output': hashes})
        save()
        if value.cause != 'exit' or value.returncode or value.truncated:
            raise ValueError('required capacity command failed: ' + name)
        if selected_executable(argv[0]) != tool:
            raise ValueError('diagnostic executable changed: ' + name)
        return value.stdout
    try:
        df = run('disk', ['df', '-k', '-P', str(output)]).decode('utf-8', errors='strict').splitlines()
        if len(df) != 2 or not df[1].split():
            raise ValueError('capacity disk device unavailable')
        device = df[1].split()[0]
        if not re.fullmatch(r'/dev/disk[0-9]+(?:s[0-9]+)*', device):
            raise ValueError('capacity disk device unknown')
        volume = plistlib.loads(run('volume', ['diskutil', 'info', '-plist', device]))
        devices = apfs_store_devices(volume)
        if len(devices) > 16:
            raise ValueError('diagnostic backing-store inventory exceeds bound')
        for index, store in enumerate(devices):
            run('store-' + str(index), ['diskutil', 'info', '-plist', store])
        for name, argv in FIXED:
            run(name, argv)
        report['observations'] = interpret(report['commands'], output)
        if report['observations']['target'] != target:
            raise ValueError('observed host differs from expected diagnostic target')
        if source_identity(root) != identity or digest(sys.executable) != report['python']['sha256']:
            raise ValueError('diagnostic source or interpreter changed')
        report.update(status='observed', completed_utc=datetime.now(timezone.utc).isoformat())
        save()
        check(output, root=root, pending=True)
        with (output / 'receipt.json').open('x') as stream:
            stream.write(json.dumps(report, indent=2) + '\n')
        return report
    except BaseException as error:
        report.update(status='failed', failure=type(error).__name__ + ': ' + str(error))
        save()
        raise


def check_retained(output, expected_commit, *, target=None, pending=False):
    """Portable byte audit; original Python/host are not executed or requalified."""
    output = Path(output).resolve(strict=True)
    report = json.loads((output / 'state.json').read_text())
    if (report['schema_version'] != 1 or report['protocol'] != 'darwin-capacity-observation-v1'
            or report['claim'] != 'read-only-capacity-observation' or report['status'] != 'observed'
            or type(report['runtime_assertions']) is not int or report['runtime_assertions'] != 0
            or report['native_qualification'] is not False or report['python']['version'] != '3.14.7'):
        raise ValueError('invalid capacity observation claim')
    if (not re.fullmatch(r'[0-9a-f]{40}', expected_commit)
            or report['source']['commit'] != expected_commit
            or report['source']['producer_sha256'] != {name: digest(ROOT / name) for name in PRODUCERS}
            or not re.fullmatch(r'[0-9a-f]{64}', report['python']['sha256'])):
        raise ValueError('capacity observation source/producer differs')
    original = Path(report['original_output'])
    if (report['expected_target'] not in TARGETS.values()
            or report['observations']['target'] != report['expected_target']
            or (target is not None and target != report['expected_target'])
            or not original.is_absolute() or '..' in original.parts
            or interpret(report['commands'], output, original_output=original) != report['observations']):
        raise ValueError('capacity observation differs from retained raw evidence')
    receipt = output / 'receipt.json'
    if not pending and not receipt.is_file():
        raise ValueError('published capacity receipt missing')
    if receipt.exists() and json.loads(receipt.read_text()) != report:
        raise ValueError('published capacity receipt differs')
    return report


def check(output, *, root=ROOT, target=None, pending=False):
    output = Path(output).resolve(strict=True)
    identity = source_identity(root)
    report = check_retained(output, identity['commit'], target=target, pending=pending)
    if (identity != report['source'] or digest(sys.executable) != report['python']['sha256']
            or report['original_output'] != str(output)):
        raise ValueError('capacity observation live source/interpreter differs')
    if any(selected_executable(row['argv'][0]) != row['executable'] for row in report['commands']):
        raise ValueError('capacity observation live executable differs')
    return report


if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('mode', choices=('observe', 'check', 'check-retained'))
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--target', required=True, choices=tuple(TARGETS.values()))
    parser.add_argument('--commit', help='Externally verified original source SHA for retained-byte audit')
    args = parser.parse_args()
    if args.mode == 'check-retained':
        if args.commit is None:
            parser.error('check-retained requires the externally verified --commit')
        result = check_retained(args.output, args.commit, target=args.target)
    elif args.mode == 'observe':
        result = observe(ROOT, args.output, target=args.target)
    else:
        result = check(args.output, target=args.target)
    print(json.dumps({'claim': result['claim'], 'runtime_assertions': 0,
                      'native_qualification': False, 'audit_mode': args.mode,
                      'observations': result['observations']}))
