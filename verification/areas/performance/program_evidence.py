"""Independently recompute descriptive program observations from retained bytes."""
from datetime import datetime, timezone
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import statistics
import sys

from benchmark_manifest import BenchmarkError
from compiler_lanes import validate_receipt
from measurement_timer import managed_timer_identity


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def producer_identity(root):
    paths = [*sorted((root/'verification/areas/performance').glob('*.py')),
             *sorted((root/'verification/runner/sifr_verify').glob('*.py'))]
    return {'files': {str(path.relative_to(root)): digest(path) for path in paths},
            'python': sys.version, 'python_path': sys.executable,
            'python_sha256': digest(sys.executable),
            'platform': [platform.system(), platform.machine(), platform.release()],
            'environment': {key: hashlib.sha256(os.environ[key].encode()).hexdigest()
                            for key in ('LD_PRELOAD', 'LD_LIBRARY_PATH', 'PATH', 'SIFR_PERFORMANCE_TIME')
                            if key in os.environ}}


def read_metrics(output, case, binary_size):
    control = json.loads((output/'control.json').read_text())
    lines = (output/'timer.txt').read_text().splitlines()
    metrics = dict(line.split('=', 1) for line in lines)
    if (len(lines) != 5 or set(metrics) != {'wall_seconds', 'user_seconds', 'system_seconds',
                                          'peak_rss_kib', 'exit'}
            or control['timed_command_exit'] != 0 or int(metrics['exit']) != 0
            or (output/'stdout').read_bytes() != case['expected_stdout'].encode()
            or (output/'stderr').read_bytes()):
        raise BenchmarkError('generated-program output or timed command status failed')
    wall = float(metrics['wall_seconds'])
    cpu = (float(metrics['user_seconds']) + float(metrics['system_seconds'])) * 1000
    rss = int(metrics['peak_rss_kib']) * 1024
    elapsed = control['launcher_elapsed_ns']
    if type(elapsed) is not int or elapsed <= 0:
        raise BenchmarkError('invalid native lifecycle duration')
    if not all(math.isfinite(value) and value >= 0 for value in (wall, cpu, rss)):
        raise BenchmarkError('invalid native process counters')
    return {'startup_lifecycle_ms': elapsed / 1e6,
            'throughput_units_per_second': case['work_units']/wall if wall > 0 else None,
            'peak_rss_bytes': rss if rss > 0 else None, 'cpu_time_ms': cpu,
            'binary_size_bytes': binary_size, 'allocation_count': None,
            'allocation_reason': 'no reviewed allocation instrumentation',
            'native_wall_seconds': wall, 'timer_resolution_seconds': 0.01}


def validate_capture(path, contract, root):
    from generated_program_metrics import dependencies
    path = path.resolve()
    value = json.loads(path.read_text())
    prepared_path = Path(value['prepared_path'])
    prepared = json.loads(prepared_path.read_text())
    compiler_path = Path(prepared['compiler_receipt'])
    compiler = json.loads(compiler_path.read_text())
    validate_receipt(root, compiler['lane'], compiler)
    now = datetime.now(timezone.utc)
    start, finish = (datetime.fromisoformat(value[key]) for key in ('started_utc', 'finished_utc'))
    if (start.utcoffset() is None or finish.utcoffset() is None or not start <= finish <= now
            or (now-finish).total_seconds() > 86400):
        raise BenchmarkError('program observations are stale or have invalid completion time')
    policy_path = root/'verification/areas/performance/data/generated_programs.json'
    if (value['schema_version'] != 1 or value['status'] != 'captured'
            or value['protocol'] != contract['protocol'] or value['numeric_regression_qualification'] is not False
            or value['producer'] != producer_identity(root) or value['timer'] != managed_timer_identity()
            or value['policy_sha256'] != digest(policy_path)
            or value['prepared_sha256'] != digest(prepared_path)
            or prepared['policy_sha256'] != digest(policy_path)
            or prepared['compiler_receipt_sha256'] != digest(compiler_path)
            or prepared['kind'] != 'preparation-output' or prepared['runtime_assertions'] != 0
            or prepared['optimization'] != 'release' or prepared['target_cpu'] != 'generic'):
        raise BenchmarkError('program capture identity or status differs')
    ids = [case['id'] for case in contract['cases']]
    if [row['id'] for row in value['rows']] != ids or [row['id'] for row in prepared['programs']] != ids:
        raise BenchmarkError('program case inventory differs')
    counts = contract['levels'][value['level']]
    for case, row, program in zip(contract['cases'], value['rows'], prepared['programs'], strict=True):
        binary = Path(program['binary'])
        if (program['binary_sha256'] != digest(binary) or row['binary_sha256'] != digest(binary)
                or program['source_sha256'] != case['source_sha256']
                or dependencies(binary) != program['runtime_dependencies']
                or len(row['observations']) != counts['warmups'] + counts['measured']
                or row['process_count'] != counts['measured']):
            raise BenchmarkError('program bytes, runtime or fixed observation counts differ')
        measured = []
        for index, observation in enumerate(row['observations']):
            folder = path.parent/case['id']/str(index)
            expected = [{'path': str(folder/name), 'sha256': digest(folder/name)}
                        for name in ('stdout', 'stderr', 'timer.txt', 'control.json')]
            metrics = read_metrics(folder, case, binary.stat().st_size)
            if (observation['raw_files'] != expected or observation['metrics'] != metrics
                    or observation['binary_sha256'] != digest(binary)
                    or observation['probe_sha256'] != digest(root/'verification/areas/performance/program_process_probe.py')
                    or observation['process_index'] != index or observation['observations'] != 1
                    or observation['inner_timing_samples'] != 1
                    or observation['warmup'] is not (index < counts['warmups'])
                    or any(metrics[name] is None for name in case['required_metrics'])):
                raise BenchmarkError('raw program observation differs')
            if not observation['warmup']:
                measured.append(metrics)
        summary = {}
        for name in case['required_metrics']:
            observations = [item[name] for item in measured]
            summary[name] = {'median': statistics.median(observations),
                             'empirical_p95': sorted(observations)[math.ceil(.95*len(observations))-1]}
        if row['summary'] != summary:
            raise BenchmarkError('program summary differs from raw fixed-count observations')
    return value
