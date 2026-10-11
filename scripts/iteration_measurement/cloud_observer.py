"""Bounded, nonqualifying timing decomposition of the unchanged cloud workloads."""
import argparse
import json
import os
import shutil
import sys
import time
from pathlib import Path

from cloud_detail import save

REFERENCE = '5fbeee50c2f70abd47e5bfcb98d1f1cd0b978042'
CANDIDATE = 'bdcf3ce90d09e29f128f3f547728b426358ac753'
LIMIT = 1795  # Leave five seconds for canonical descendant cleanup.
RESERVE = 8 * 1024**3


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--candidate', type=Path, required=True)
    parser.add_argument('--baseline', type=Path, required=True)
    parser.add_argument('--evidence', type=Path, required=True)
    parser.add_argument('--case-prefix', default='')
    args = parser.parse_args()
    candidate, baseline, evidence = (x.resolve() for x in (args.candidate, args.baseline, args.evidence))
    evidence.mkdir(parents=True, exist_ok=False)
    logs = evidence / 'logs'
    logs.mkdir()
    sys.path.insert(0, str(candidate / 'verification/areas/performance'))
    sys.path.insert(0, str(candidate / 'verification/runner'))
    from sifr_verify.process_execution import execute
    from benchmark_manifest import load_manifest, validate_manifest
    from cloud_precision import pairs_for_case
    from cloud_statistics import schedule
    from cloud_benchmarks import tooling_digest
    import subprocess

    manifest = load_manifest(candidate / 'verification/areas/performance/data/benchmark_manifest.json')
    cases = validate_manifest(manifest)
    chosen = [c for c in cases if c.id.startswith(args.case_prefix)]
    if not chosen:
        raise RuntimeError('Empty requested workload selection')
    if subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=candidate, text=True).strip() != CANDIDATE:
        raise RuntimeError('Candidate source changed')
    run_started = time.monotonic()
    report = {
        'qualification_eligible': False, 'purpose': 'cost decomposition only',
        'candidate_source': CANDIDATE, 'reference_compiler': REFERENCE,
        'tooling_sha256': tooling_digest(candidate), 'per_category_limit_seconds': 1800,
        'selection': [c.id for c in chosen], 'commands': [], 'cases': [],
        'case_plan': [{'case_id': c.id, 'kind': c.kind, 'group': c.raw['group'],
                       'outer_warmups_per_endpoint': c.warmups,
                       'pairs_required': pairs_for_case(c.id), 'schedule': schedule(c.id),
                       'internal_measured': c.measured, 'inner_repetitions': c.raw.get('inner_repetitions', 1),
                       'sample_timeout_ms': c.timeout_ms} for c in chosen],
        'host': {'uname': list(os.uname()), 'affinity_cpus': sorted(os.sched_getaffinity(0)),
                 'meminfo': Path('/proc/meminfo').read_text(), 'free_disk_bytes': shutil.disk_usage(evidence).free,
                 'planned_cargo_jobs': os.environ.get('CARGO_BUILD_JOBS'), 'reserve_bytes': RESERVE},
    }
    save(evidence / 'report.json', report)

    def run(name, command, cwd, extra=None):
        free = shutil.disk_usage(evidence).free
        if free < RESERVE:
            row = {'name': name, 'status': 'blocked-capacity', 'seconds': None,
                   'available_bytes': free, 'reserve_bytes': RESERVE}
            report['commands'].append(row)
            save(evidence / 'report.json', report)
            return row
        env = os.environ.copy()
        env.update(extra or {})
        env['SIFR_VERIFY_DISK_FLOOR_PATH'] = str(evidence)
        env['SIFR_VERIFY_DISK_FLOOR_BYTES'] = str(RESERVE)
        print(f'OBSERVATION START {name}; limit=1800s; free_disk={free}', flush=True)
        save(evidence / 'active.json', {'name': name, 'started_unix': time.time(), 'command': command})
        started = time.perf_counter()
        outcome = execute(command, cwd=cwd, env=env, deadline_seconds=LIMIT, limit_bytes=32 * 1024**2)
        elapsed = time.perf_counter() - started
        (logs / (name + '.stdout')).write_bytes(outcome.stdout)
        (logs / (name + '.stderr')).write_bytes(outcome.stderr)
        status = 'completed' if outcome.returncode == 0 and outcome.cause == 'exit' and not outcome.truncated else 'failed-incomplete'
        if outcome.cause == 'safety_deadline':
            status = 'cancelled-30-minute-limit'
        row = {'name': name, 'seconds': elapsed, 'returncode': outcome.returncode,
               'cause': outcome.cause, 'status': status, 'command': command,
               'truncated': outcome.truncated, 'cwd': str(cwd)}
        report['commands'].append(row)
        save(evidence / 'report.json', report)
        print('OBSERVATION END ' + json.dumps(row), flush=True)
        return row

    # Fail before long preparation if this runner cannot satisfy the real host contract.
    from reference_host import host_details
    report['admitted_host_preflight'] = host_details()
    save(evidence / 'report.json', report)

    ready = {}
    detail = str(Path(__file__).with_name('cloud_detail.py').resolve())
    for role, repo in [('baseline', baseline), ('candidate', candidate)]:
        role_env = {'CARGO_TARGET_DIR': str(repo / 'target')}
        fetch = run(role + '-dependency-acquisition', ['cargo', 'fetch', '--locked'], repo, role_env)
        if fetch['status'] != 'completed':
            ready[role] = False
            continue
        output = evidence / 'endpoints' / role
        compiler = run(role + '-compiler-preparation', [sys.executable,
                       str(repo / 'verification/areas/performance/prepare_compiler_lane.py'),
                       '--lane', 'contributor-dev', '--output', str(output)], repo, role_env)
        helper = run(role + '-frontend-helper-preparation', ['cargo', 'build', '--locked', '--offline',
                     '-p', 'sifr_frontend', '--bin', 'frontend_query_bench',
                     '--message-format=json-render-diagnostics'], repo, role_env)
        ready[role] = compiler['status'] == helper['status'] == 'completed'
        if ready[role]:
            final = run(role + '-artifact-copy-and-identity', [sys.executable, detail, 'finalize',
                        '--repo', str(repo), '--output', str(output), '--cargo-messages',
                        str(logs / (role + '-frontend-helper-preparation.stdout'))], repo, role_env)
            ready[role] = final['status'] == 'completed'
    if all(ready.values()):
        admitted = run('paired-endpoint-admission', [sys.executable, detail, 'validate',
                       '--repo', str(candidate), '--evidence', str(evidence)], candidate)
        ready['admission'] = admitted['status'] == 'completed'
    else:
        ready['admission'] = False
    for case in chosen:
        if not all(ready.values()):
            report['cases'].append({'case_id': case.id, 'status': 'blocked-prerequisite',
                                    'seconds': None, 'completed_pairs': 0,
                                    'pairs_required': pairs_for_case(case.id), 'readiness': ready})
            continue
        # Leave time for artifact upload rather than lose raw evidence to the job deadline.
        if time.monotonic() - run_started > 5.5 * 3600:
            report['cases'].append({'case_id': case.id, 'status': 'not-started-job-window',
                                    'seconds': None, 'completed_pairs': 0,
                                    'pairs_required': pairs_for_case(case.id)})
            continue
        destination = evidence / 'cases' / case.id
        row = run('case-' + case.id, [sys.executable, detail, 'case', '--repo', str(candidate),
                  '--case', case.id, '--output', str(destination), '--evidence', str(evidence)], candidate)
        progress = destination / 'progress.json'
        completed = json.loads(progress.read_text()) if progress.exists() else {}
        report['cases'].append({'case_id': case.id, **row,
                                'completed_pairs': completed.get('completed_pairs', 0),
                                'pairs_required': pairs_for_case(case.id),
                                'outer_warmup_seconds': completed.get('outer_warmup_seconds'),
                                'paired_execution_seconds': completed.get('paired_execution_seconds'),
                                'statistics_seconds': completed.get('statistics_seconds'),
                                'workers': completed.get('workers', [])})
        save(evidence / 'report.json', report)
    run('raw-evidence-hash-index', [sys.executable, detail, 'index', '--repo', str(candidate),
        '--evidence', str(evidence)], candidate)
    run('raw-evidence-integrity-verification', [sys.executable, detail, 'check-index',
        '--repo', str(candidate), '--evidence', str(evidence)], candidate)
    report['collection_status'] = 'finished-observational-attempts'
    report['whole_gate_status'] = 'not-qualified'
    report['whole_gate_evaluation'] = 'Not run: this segmented instrumented campaign is not one qualifying invocation.'
    report['total_observation_seconds'] = time.monotonic() - run_started
    save(evidence / 'report.json', report)
    print('OBSERVATION COLLECTION FINISHED; qualification_eligible=false', flush=True)


if __name__ == '__main__':
    main()
