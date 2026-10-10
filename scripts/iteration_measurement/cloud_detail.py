"""Observational cloud worker timing; never emits a qualification receipt."""
import argparse
import hashlib
import json
import os
import shutil
import sys
import time
from pathlib import Path


def save(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_name(path.name + '.tmp')
    tmp.write_text(json.dumps(value, indent=2) + '\n')
    tmp.replace(path)


def install_imports(repo):
    sys.path.insert(0, str(repo / 'verification/areas/performance'))
    sys.path.insert(0, str(repo / 'verification/runner'))
    os.chdir(repo)


def finalize(repo, output, cargo_messages):
    import cloud_worker as worker
    import compiler_lanes
    receipt = json.loads((output / 'receipt.json').read_text())
    rows = [json.loads(line) for line in cargo_messages.read_text().splitlines() if line.strip()]
    artifacts = [r for r in rows if r.get('reason') == 'compiler-artifact'
                 and r.get('target', {}).get('name') == 'frontend_query_bench'
                 and r.get('executable')]
    if len(artifacts) != 1 or artifacts[0]['profile']['opt_level'] != '1':
        raise RuntimeError('Helper must be the actual measured dev artifact')
    started = time.perf_counter()
    executable = output / 'sifr'
    shutil.copy2(receipt['artifact']['path'], executable)
    receipt['artifact']['path'] = str(executable)
    helper = output / 'frontend_query_bench'
    shutil.copy2(artifacts[0]['executable'], helper)
    copy_seconds = time.perf_counter() - started
    started = time.perf_counter()
    receipt['cloud_frontend_helper'] = {
        'path': str(helper), 'sha256': compiler_lanes.digest(helper),
        'profile': artifacts[0]['profile'],
        'cargo_messages_sha256': compiler_lanes.digest(cargo_messages),
    }
    receipt['cloud_source'] = worker.clean_source()
    receipt['cloud_repo'] = str(repo)
    receipt['cloud_identity'] = worker.measured_identity()
    compiler_lanes.validate_receipt(repo, 'contributor-dev', receipt)
    save(output / 'receipt.json', receipt)
    save(output / 'preparation-detail.json', {
        'copy_seconds': copy_seconds,
        'hash_identity_validation_and_receipt_seconds': time.perf_counter() - started,
        'qualification_eligible': False,
        'note': 'Same actual artifact fields as cloud_worker.prepare; separately timed preparation stages.',
    })


def observe_worker(receipt, case, output, artifacts, warmup):
    import cloud_worker as worker
    import compiler_lanes
    detail = {'qualification_eligible': False, 'hashes': [], 'source_checks': []}
    original_digest = compiler_lanes.digest
    original_source = worker.clean_source
    original_endpoint = worker.endpoint

    def digest(path):
        started = time.perf_counter()
        result = original_digest(path)
        detail['hashes'].append({'path': str(path), 'bytes': Path(path).stat().st_size,
                                 'seconds': time.perf_counter() - started})
        return result

    def source():
        started = time.perf_counter()
        result = original_source()
        detail['source_checks'].append(time.perf_counter() - started)
        return result

    def endpoint(path):
        started = time.perf_counter()
        try:
            return original_endpoint(path)
        finally:
            detail['endpoint_admission_seconds'] = time.perf_counter() - started

    compiler_lanes.digest = digest
    worker.clean_source = source
    worker.endpoint = endpoint
    started = time.perf_counter()
    try:
        worker.measure(receipt, case, output, artifacts, warmup)
    finally:
        detail['measure_function_seconds'] = time.perf_counter() - started
        detail['note'] = 'Hash/source timings are nested in admission/measurement, not additional wall time.'
        save(output.parent / (output.name + '-observer.json'), detail)


def observe_case(args):
    from benchmark_manifest import load_manifest, validate_manifest
    from cloud_statistics import schedule, median_decision
    from cloud_precision import pairs_for_case
    from cloud_contract import threshold, empirical_p95
    from sifr_verify.process_execution import execute
    import statistics

    manifest = load_manifest(args.repo / 'verification/areas/performance/data/benchmark_manifest.json')
    case = next(c for c in validate_manifest(manifest) if c.id == args.case)
    output = args.output
    output.mkdir(parents=True, exist_ok=False)
    endpoints = {role: json.loads((args.evidence / 'endpoints' / role / 'receipt.json').read_text())
                 for role in ('baseline', 'candidate')}
    orders = schedule(case.id)
    progress = {'case_id': case.id, 'pairs_required': pairs_for_case(case.id),
                'completed_pairs': 0, 'workers': [], 'qualification_eligible': False,
                'status': 'running', 'schedule': orders}
    save(output / 'progress.json', progress)
    pairs = []

    def worker(role, index, warmup=False):
        sample = output / ('warmups' if warmup else 'pairs') / str(index) / role
        sample.parent.mkdir(parents=True, exist_ok=True)
        receipt = args.evidence / 'endpoints' / role / 'receipt.json'
        repo = Path(endpoints[role]['cloud_repo'])
        command = [sys.executable, __file__, 'worker', '--repo', str(repo),
                   '--receipt', str(receipt), '--case', case.id, '--output', str(sample),
                   '--artifacts', str(args.evidence / 'artifacts' / role)]
        if warmup:
            command.append('--warmup')
        started = time.perf_counter()
        outcome = execute(command, cwd=repo, deadline_seconds=600, limit_bytes=16 * 1024**2)
        elapsed = time.perf_counter() - started
        sample.with_name(sample.name + '.stdout').write_bytes(outcome.stdout)
        sample.with_name(sample.name + '.stderr').write_bytes(outcome.stderr)
        row = {'endpoint': role, 'warmup': warmup, 'index': index,
               'worker_wall_seconds': elapsed, 'returncode': outcome.returncode,
               'cause': outcome.cause, 'truncated': outcome.truncated}
        evidence = sample / 'samples' / case.id / '0.json'
        if evidence.exists():
            raw = json.loads(evidence.read_text())
            row['measured_child_seconds'] = raw['result']['duration_ms'] / 1000
            row['outer_minus_child_seconds'] = elapsed - row['measured_child_seconds']
        observer = sample.parent / (sample.name + '-observer.json')
        if observer.exists():
            observed = json.loads(observer.read_text())
            row['hash_seconds_nested'] = sum(x['seconds'] for x in observed['hashes'])
            row['hashed_bytes'] = sum(x['bytes'] for x in observed['hashes'])
            row['endpoint_admission_seconds_nested'] = observed.get('endpoint_admission_seconds')
        progress['workers'].append(row)
        save(output / 'progress.json', progress)
        if outcome.returncode or outcome.cause != 'exit' or outcome.truncated:
            raise RuntimeError(f'{role} worker failed: {outcome.cause}, {outcome.returncode}')
        return json.loads((sample / 'summary.json').read_text())

    started = time.perf_counter()
    try:
        for role in endpoints:
            for index in range(case.warmups):
                worker(role, index, True)
        progress['outer_warmup_seconds'] = time.perf_counter() - started
        paired_start = time.perf_counter()
        for index, order in enumerate(orders):
            pair = {'order': order, 'pair_index': index}
            for letter in order:
                role = 'baseline' if letter == 'A' else 'candidate'
                pair[role] = worker(role, index)
            pairs.append(pair)
            save(output / 'pairs' / str(index) / 'pair.json', pair)
            progress['completed_pairs'] = len(pairs)
            save(output / 'progress.json', progress)
        progress['paired_execution_seconds'] = time.perf_counter() - paired_start
        budgets = json.loads((args.repo / 'verification/areas/performance/data/budgets.json').read_text())
        budget = next(b for b in budgets['budgets'] if b['benchmark_id'] == case.id)
        stats_start = time.perf_counter()
        excess = [statistics.median(p['candidate']['latencies_ms']) -
                  threshold(budget, statistics.median(p['baseline']['latencies_ms'])) for p in pairs]
        decision = median_decision(excess, orders, len(manifest['cases']), len(pairs))
        tails = {role: empirical_p95([value for p in pairs for value in p[role]['latencies_ms']])
                 for role in endpoints}
        progress['statistics_seconds'] = time.perf_counter() - stats_start
        progress['diagnostic_median_decision_only'] = decision
        progress['descriptive_p95_ms'] = tails
        progress['status'] = 'complete-observation-not-qualification'
    except Exception as error:
        progress['status'] = 'failed-incomplete'
        progress['error'] = str(error)
        raise
    finally:
        progress['case_function_seconds'] = time.perf_counter() - started
        save(output / 'progress.json', progress)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('mode', choices=['finalize', 'validate', 'worker', 'case', 'index', 'check-index'])
    parser.add_argument('--repo', type=Path, required=True)
    for name in ('output', 'receipt', 'artifacts', 'evidence', 'cargo-messages'):
        parser.add_argument('--' + name, type=Path)
    parser.add_argument('--case')
    parser.add_argument('--warmup', action='store_true')
    args = parser.parse_args()
    args.repo = args.repo.resolve()
    install_imports(args.repo)
    if args.mode == 'finalize':
        finalize(args.repo, args.output, args.cargo_messages)
    elif args.mode == 'worker':
        observe_worker(args.receipt, args.case, args.output, args.artifacts, args.warmup)
    elif args.mode == 'case':
        observe_case(args)
    elif args.mode == 'index':
        root = args.evidence.resolve()
        records = []
        for path in sorted(root.rglob('*')):
            relative = path.relative_to(root)
            if not path.is_file() or path.is_symlink() or relative.parts[0] == 'artifacts':
                continue
            if path.name in {'sifr', 'frontend_query_bench', 'report.json', 'active.json', 'raw-index.json'}:
                continue
            records.append({'path': str(relative), 'bytes': path.stat().st_size,
                            'sha256': hashlib.sha256(path.read_bytes()).hexdigest()})
        save(root / 'raw-index.json', {'qualification_eligible': False, 'files': records})
        print(json.dumps({'files': len(records), 'bytes': sum(r['bytes'] for r in records)}))
    elif args.mode == 'check-index':
        root = args.evidence.resolve()
        records = json.loads((root / 'raw-index.json').read_text())['files']
        for record in records:
            path = (root / record['path']).resolve()
            if not path.is_relative_to(root) or hashlib.sha256(path.read_bytes()).hexdigest() != record['sha256']:
                raise RuntimeError('Changed diagnostic raw evidence')
        print(json.dumps({'verified_files': len(records), 'qualification_eligible': False}))
    else:
        from cloud_benchmarks import check_endpoints
        paths = {role: args.evidence / 'endpoints' / role / 'receipt.json'
                 for role in ('baseline', 'candidate')}
        check_endpoints(paths, '5fbeee50c2f70abd47e5bfcb98d1f1cd0b978042')


if __name__ == '__main__':
    main()
