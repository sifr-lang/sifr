"""Fixed harmless Darwin teardown measurement; no runtime or qualification work."""
import argparse
import json
import os
from pathlib import Path
import platform
import signal
import sys
import time

import native_runtime_diagnostic as common
import native_teardown_process as process
from native_teardown_checker import check_case
from sifr_verify.process_disk_budget import DiskBudget

ROOT = common.ROOT
PROTOCOL = 'darwin-production-executor-harmless-v1'
PRODUCTION_COMMIT = '75d6d4f7f83d7de2979d7eca98250e483626bd0f'
PRODUCTION_RUNTIME = {
    'verification/runner/sifr_verify/__init__.py': 'e3081f3db9e711e6b4201ddc2b8b6b4c65efd896c72fd0dd37dbddeda8a76091',
    'verification/runner/sifr_verify/process_execution.py': 'eb7fdef427352ead7d997fadd7eae116ef23fd4e76be31ad834c24680c98b750',
    'verification/runner/sifr_verify/process_supervisor.py': '306917639782c13acf88b36c25a83c29860670c72effe9e6792f3921ac1dcf95',
    'verification/runner/sifr_verify/process_disk_budget.py': '97db8045e181ea0268706fc9feb99a900d8e10c1d8e985d80ac75b5bcbe08cec'}
WORKFLOWS = ('.github/workflows/native-teardown-diagnostic.yml',
             '.github/workflows/published-native-qualification.yml')
SPEC = {'fixture_kinds': list(process.KINDS), 'fixture_lifetime_seconds': 8,
        'term_interval_seconds': .25, 'deadline_seconds': 60,
        'evidence_limit_bytes': 8*1024**2, 'ps_output_limit_bytes': process.MAX_CAPTURE,
        'memory_peak_bytes': 256*1024**2, 'memory_reserve_bytes': 2*1024**3,
        'tmpfs_growth_bytes': 0, 'disk_reserve_bytes': 8*1024**3,
        'disk_growth_bytes': 256*1024**2, 'retained_copy_bytes': 8*1024**2}
REQUIREMENTS = {key: SPEC[key] for key in ('memory_peak_bytes', 'memory_reserve_bytes',
    'tmpfs_growth_bytes', 'disk_reserve_bytes', 'disk_growth_bytes', 'retained_copy_bytes')}


def identity():
    result = common.source_identity(ROOT)
    if any(common.digest(ROOT/name) != digest for name, digest in PRODUCTION_RUNTIME.items()):
        raise ValueError('reviewed production runtime bytes differ')
    result['production'] = {'commit': PRODUCTION_COMMIT, 'runtime_sha256': PRODUCTION_RUNTIME}
    result['producer_sha256'].update({name: common.digest(ROOT/name) for name in WORKFLOWS})
    return result


def workflow_document():
    invocation = '"$GITHUB_WORKSPACE/verification/.venv/bin/python" -B verification/areas/sysroot_release/native_teardown_diagnostic.py '
    return {'name': 'native-production-executor-diagnostic', 'on': {'workflow_dispatch': None},
        'permissions': {'contents': 'read'}, 'jobs': {'observe': {
            'name': 'harmless-production-executor-aarch64-apple-darwin', 'runs-on': 'macos-15',
            'timeout-minutes': 10, 'env': {'SIFR_NATIVE_HOST_KIND': 'dedicated-darwin',
                                         'PYTHONDONTWRITEBYTECODE': '1'},
            'steps': [
                {'uses': 'actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1',
                 'with': {'ref': '${{ github.sha }}', 'persist-credentials': False}},
                {'uses': 'astral-sh/setup-uv@20cfd1bf945f4377ade1205e4dbc17946fc9a30d',
                 'with': {'version-file': 'verification/pyproject.toml',
                    'checksum': '51c6170e8e3a01cef9f33b94f582b7b81ac65046f55d40afb35f9cff5a68c179'}},
                {'name': 'Prepare explicit canonical interpreter',
                 'run': 'uv python install 3.14.7\nuv sync --project verification --locked'},
                {'name': 'Observe harmless owned fixtures',
                 'run': invocation+'observe --output "$RUNNER_TEMP/native-teardown-diagnostic"'},
                {'name': 'Check exact diagnostic evidence', 'if': 'always()',
                 'run': invocation+'check --output "$RUNNER_TEMP/native-teardown-diagnostic"'},
                {'name': 'Retain failure and signal evidence', 'if': 'always()',
                 'uses': 'actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a',
                 'with': {'name': 'native-teardown-diagnostic-${{ github.sha }}-aarch64-apple-darwin-${{ github.run_attempt }}',
                    'path': '${{ runner.temp }}/native-teardown-diagnostic/evidence',
                    'if-no-files-found': 'error', 'retention-days': 30}}
            ]}}}


def validate_workflow(document):
    return [] if document == workflow_document() else ['fixed teardown diagnostic workflow differs']


def case_validation(case, report):
    try:
        passed = check_case(case, report)
    except (ValueError, KeyError, TypeError) as error:
        return False, type(error).__name__+': '+str(error)
    return passed, None


def checked_case(case, report):
    passed, error = case_validation(case, report)
    if case.get('validation_error') != error:
        raise ValueError('retained case validation differs')
    return passed


def status(cases, report):
    results = [checked_case(case, report) for case in cases]
    return 'observed' if len(cases) == len(process.KINDS) and all(results) else 'stopped'


def observe(output):
    if (platform.system() != 'Darwin' or platform.machine() != 'arm64'
            or os.environ.get('SIFR_NATIVE_HOST_KIND') != 'dedicated-darwin'
            or signal.getsignal(signal.SIGCHLD) != signal.SIG_DFL):
        raise ValueError('requires dedicated native ARM Darwin with unreaped child ownership')
    source = identity()
    output = output.absolute()
    env, proof = common.storage.prepare(output, os.environ)
    evidence = output/'evidence'
    report = {'protocol': PROTOCOL, 'schema_version': 1, 'source': source, 'specification': SPEC,
        'requirements': REQUIREMENTS, 'runtime_assertions': 0, 'native_qualification': False,
        'target': common.TARGET, 'status': 'stopped', 'original_output': str(output), 'source_root': str(ROOT),
        'host': {'system': platform.system(), 'machine': platform.machine(), 'uname': list(os.uname())},
        'driver': {'pid': os.getpid(), 'uid': os.geteuid(), 'ruid': os.getuid()},
        'tools': {'python': common.python_identity(), 'ps': {'path': '/bin/ps', 'sha256': common.digest('/bin/ps')}},
        'storage': proof, 'cases': [], 'workflow': {key: os.environ[key] for key in
            ('GITHUB_SHA', 'GITHUB_RUN_ID', 'GITHUB_RUN_ATTEMPT', 'GITHUB_WORKFLOW_REF') if key in os.environ}}
    env, deadline = common.deadline_environment(env, SPEC['deadline_seconds'])
    report['deadline'] = deadline
    disk = DiskBudget.from_environment(env)
    report['inherited_disk_floor_bytes'] = disk.floor
    def checkpoint():
        if time.monotonic() >= deadline:
            raise ValueError('diagnostic absolute deadline')
        disk.check()

    def save():
        if sum(len(json.dumps(case).encode()) for case in report['cases']) > SPEC['evidence_limit_bytes']-65536:
            raise ValueError('aggregate diagnostic evidence bound')
        report['files'] = common.file_evidence(evidence)
        encoded = json.dumps(report, separators=(',', ':'))+'\n'
        if len(encoded.encode()) > SPEC['evidence_limit_bytes']:
            raise ValueError('diagnostic state exceeds retained bound')
        (evidence/'state.json').write_text(encoded)

    previous_handlers = {}
    def interrupted(signum, frame):
        raise InterruptedError('diagnostic interrupted by signal '+str(signum))
    try:
        for number in (signal.SIGTERM, signal.SIGINT):
            previous_handlers[number] = signal.signal(number, interrupted)
        if report['workflow'] and report['workflow'].get('GITHUB_SHA') != source['commit']:
            raise ValueError('workflow source differs')
        capacity = common.resources(output)
        report['admission'] = common.admit(capacity, REQUIREMENTS)
        floor = max(disk.floor, capacity.disk_available_bytes-REQUIREMENTS['disk_growth_bytes']-
                    REQUIREMENTS['retained_copy_bytes'])
        env['SIFR_VERIFY_DISK_FLOOR_BYTES'] = str(floor)
        disk = DiskBudget.from_environment(env)
        report['environment'] = common.recorded_environment(env, output)
        if deadline-time.monotonic() < 30:
            raise ValueError('inherited deadline leaves no bounded fixture cleanup window')
        save()
        with common.parent_temporary(proof):
            for kind in process.KINDS:
                checkpoint()
                if deadline-time.monotonic() < 15:
                    raise ValueError('fixture cleanup window unavailable')
                common.storage.check(proof, output, env)
                admission = common.admit(common.resources(output), dict(REQUIREMENTS,
                    disk_growth_bytes=0, retained_copy_bytes=0))
                case = process.observe_case(kind, output/'temporary', report['tools']['python']['path'],
                                            checkpoint, env)
                case['admission'] = admission
                report['cases'].append(case)
                report['finished'] = time.monotonic()
                passed, case['validation_error'] = case_validation(case, report)
                save()
                if not passed:
                    raise ValueError('fixture cleanup or observation incomplete: '+kind)
        common.check_live_tool_identities(report['tools'])
        if source != identity():
            raise ValueError('diagnostic producer changed')
        checkpoint()
        report['finished'] = time.monotonic()
        report['status'] = status(report['cases'], report)
    except Exception as error:
        report['failure'] = type(error).__name__+': '+str(error)
    finally:
        report['finished'] = time.monotonic()
        save()
        for number, handler in previous_handlers.items():
            signal.signal(number, handler)
    return report


def check_retained(evidence, commit):
    evidence = evidence.resolve(strict=True)
    if (evidence/'state.json').is_symlink() or (evidence/'state.json').stat().st_size > SPEC['evidence_limit_bytes']:
        raise ValueError('retained state bound/type')
    report = json.loads((evidence/'state.json').read_text())
    if (report.get('protocol') != PROTOCOL or report.get('schema_version') != 1
            or report.get('source') != identity() or report['source']['commit'] != commit
            or report.get('specification') != SPEC or report.get('requirements') != REQUIREMENTS
            or type(report.get('runtime_assertions')) is not int or report['runtime_assertions'] != 0
            or report.get('native_qualification') is not False or report.get('target') != common.TARGET
            or report.get('files') != common.file_evidence(evidence)):
        raise ValueError('retained microprobe identity or bytes differ')
    host = report.get('host', {})
    if (host.get('system') != 'Darwin' or host.get('machine') != 'arm64'
            or not isinstance(host.get('uname'), list) or len(host['uname']) != 5
            or not all(isinstance(value, str) and value for value in host['uname'])
            or host['uname'][0] != 'Darwin' or host['uname'][4] != 'arm64'):
        raise ValueError('retained actual Darwin host identity missing')
    original = Path(report['original_output'])
    if (not original.is_absolute() or report['storage']['environment'] != common.storage.bindings(original)
            or set(report['storage']['paths']) != {'.', *common.storage.NAMES}
            or any(type(report['driver'][key]) is not int or report['driver'][key] < (1 if key == 'pid' else 0)
                   for key in ('pid', 'uid', 'ruid'))):
        raise ValueError('retained storage or driver authority differs')
    if report['workflow'] and report['workflow'].get('GITHUB_SHA') != commit:
        raise ValueError('retained workflow identity differs')
    if sum(row['size'] for row in report['files'].values()) > SPEC['evidence_limit_bytes']:
        raise ValueError('retained evidence bound')
    if 'admission' in report:
        authority = report['admission']
        capacity = common.Resources(**authority['resources'])
        if (authority != common.admit(capacity, REQUIREMENTS) or capacity.disk_memory_backed is not False
                or capacity.diagnostics.get('capacity_authority') != 'declared-dedicated-darwin-vm-stat'):
            raise ValueError('retained initial admission differs')
        floor = report['inherited_disk_floor_bytes']
        if type(floor) is not int or floor < SPEC['disk_reserve_bytes']:
            raise ValueError('inherited disk floor differs')
        expected_env = common.storage.bindings(original) | {
            'SIFR_VERIFY_DISK_FLOOR_BYTES': str(max(floor, capacity.disk_available_bytes-
                REQUIREMENTS['disk_growth_bytes']-REQUIREMENTS['retained_copy_bytes'])),
            'SIFR_VERIFY_SAFETY_DEADLINE_MONOTONIC': repr(report['deadline'])}
        if report['environment'] != common.recorded_environment(expected_env, original):
            raise ValueError('retained fixture environment differs')
        if any(row['storage'] != common.storage.topology(capacity.diagnostics)
               for row in report['storage']['paths'].values()):
            raise ValueError('retained physical storage topology differs')
    elif report['cases']:
        raise ValueError('fixtures without initial admission')
    if [c['kind'] for c in report['cases']] != list(process.KINDS[:len(report['cases'])]):
        raise ValueError('fixture sequence differs')
    for case in report['cases']:
        authority = case['admission']
        if authority != common.admit(common.Resources(**authority['resources']),
                dict(REQUIREMENTS, disk_growth_bytes=0, retained_copy_bytes=0)):
            raise ValueError('retained fixture admission differs')
        checked_case(case, report)
    expected = 'stopped' if report.get('failure') else status(report['cases'], report)
    if expected == 'observed' and not report['finished'] <= report['deadline']:
        raise ValueError('observed run exceeded inherited deadline')
    if report['status'] != expected:
        raise ValueError('retained result overstates observed evidence')
    return report


def check(output):
    report = check_retained(output/'evidence', identity()['commit'])
    if output.resolve() != Path(report['original_output']):
        raise ValueError('live checker requires original owned paths')
    common.storage.check(report['storage'], output, common.storage.environment(output, os.environ))
    common.check_live_tool_identities(report['tools'])
    return report


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('mode', choices=('observe', 'check', 'check-retained'))
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--commit')
    args = parser.parse_args()
    report = (observe(args.output) if args.mode == 'observe' else check(args.output) if args.mode == 'check'
              else check_retained(args.output, args.commit or ''))
    print(json.dumps({'protocol': PROTOCOL, 'status': report['status'], 'runtime_assertions': 0,
                      'native_qualification': False}))
    if args.mode == 'observe' and report['status'] != 'observed':
        raise SystemExit(1)
