"""Exercise the actual embedded probe with ordered imports and fresh metadata."""
from __future__ import annotations

import contextlib
import hashlib
import importlib
import importlib.metadata
import io
import json
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import types

CASES = (
    "empty", "overlap", "all-failed", "ownership-error", "ownership-interrupt",
    "version-error", "mutation", "failed-mutation", "namespace",
)


def require(condition, message):
    # These controls must remain active under Python -O.
    if not condition:
        raise AssertionError(message)


def probe_script(repo_root: Path) -> str:
    source = repo_root / "crates/sifr_package/src/cargo/python_probe.rs"
    match = re.search(r'const PROBE_SCRIPT: &str = r#"(.*?)"#;', source.read_text(), re.S)
    require(match is not None, "authoritative embedded Python probe is missing")
    return match[1]


def child(name, work, repo_root):
    script = probe_script(Path(repo_root))
    root = Path(work)
    events = []
    mapping_calls = []
    declared = []
    native = []
    ownership = {}
    versions = {}
    orig_import = importlib.import_module
    if name == 'empty':
        pass
    elif name == 'all-failed':
        declared = ['probe_missing']
        native = ['probe_missing']
    elif name == 'overlap':
        declared = ['probe_good', 'probe_missing', 'probe_good']
        native = ['probe_good', 'probe_native']
        ownership = {'probe_good': ['zeta', 'Alpha', 'zeta'], 'probe_native': []}
        versions = {'Alpha': '1.0', 'zeta': '2.0'}
    elif name in ['ownership-error', 'ownership-interrupt']:
        declared = ['probe_good', 'probe_missing']
        native = ['probe_native']
    elif name == 'version-error':
        declared = ['probe_bad_version', 'probe_good']
        ownership = {'probe_bad_version': ['absent'], 'probe_good': ['good']}
        versions = {'good': '1.0'}
    elif name in ['mutation', 'failed-mutation']:
        declared = ['probe_early', 'probe_mutator']
        native = ['probe_early']
        ownership = {'probe_early': ['old']}
        versions = {'old': '1.0', 'new': '2.0'}
    elif name == 'namespace':
        declared = ['local_probe_ns']
        native = ['local_probe_ns']
        sys.path[:0] = [str(root / 'first'), str(root / 'second')]
    else:
        raise ValueError(name)

    def fake_import(module, *args, **kwargs):
        if not module.startswith('probe_'):
            return orig_import(module, *args, **kwargs)
        events.append(module)
        if module == 'probe_missing':
            raise ModuleNotFoundError('deliberately missing import')
        if module == 'probe_mutator':
            sys.path.append(str(root / 'new-site'))
            ownership['probe_early'] = ['new']
            if name == 'failed-mutation':
                raise RuntimeError('mutation before import failure')
        result = types.ModuleType(module)
        result.__file__ = str(root / (module + '.py'))
        return result

    def mapping():
        mapping_calls.append(list(events))
        if name == 'ownership-error':
            raise RuntimeError('metadata unavailable')
        if name == 'ownership-interrupt':
            raise KeyboardInterrupt('metadata interrupted')
        return ownership

    def version(distribution):
        if distribution not in versions:
            raise importlib.metadata.PackageNotFoundError(distribution)
        return versions[distribution]
    if name != 'namespace':
        importlib.import_module = fake_import
        importlib.metadata.packages_distributions = mapping
        importlib.metadata.version = version
    project = root / 'pyproject.toml'
    lock = root / 'uv.lock'
    sys.argv = ['probe', json.dumps(declared), json.dumps(native), str(project), str(lock)]
    output = io.StringIO()
    with contextlib.redirect_stdout(output):
        exec(compile(script, 'embedded Python environment probe', 'exec'), {})
    payload = json.loads(output.getvalue())
    records = payload['imports'] + payload['native_imports']
    require([r['root'] for r in records] == declared + native, 'Python environment observation control failed')
    require(payload['pyproject_digest'] == hashlib.sha256(project.read_bytes()).hexdigest(), 'Python environment observation control failed')
    require(payload['uv_lock_digest'] == hashlib.sha256(lock.read_bytes()).hexdigest(), 'Python environment observation control failed')
    require(payload['sys_prefix'] == str(Path(sys.prefix).resolve()), 'Python environment observation control failed')
    if name != 'namespace':
        require(events == declared + native, (name, events))
        expected_calls = 0 if name in ['empty', 'all-failed'] else 1
        require(len(mapping_calls) == expected_calls, 'Python environment observation control failed')
        if mapping_calls:
            require(mapping_calls == [declared + native], mapping_calls)
    if name == 'overlap':
        expected = [{'name': 'Alpha', 'version': '1.0'}, {'name': 'zeta', 'version': '2.0'}]
        require([r['distributions'] for r in records if r['root'] == 'probe_good'] == [expected] * 3, 'Python environment observation control failed')
        require(not records[1]['ok'] and records[1]['error'] == 'ModuleNotFoundError: deliberately missing import', 'Python environment observation control failed')
        require(records[-1]['ok'] and records[-1]['distributions'] == [], 'Python environment observation control failed')
    elif name == 'all-failed':
        require(all((not r['ok'] and r['error'].startswith('ModuleNotFoundError:') for r in records)), 'Python environment observation control failed')
    elif name in ['ownership-error', 'ownership-interrupt']:
        require(all((not r['ok'] and r['origin'] is None and (r['distributions'] == []) for r in records)), 'Python environment observation control failed')
        require(records[1]['error'] == 'ModuleNotFoundError: deliberately missing import', 'Python environment observation control failed')
        expected = 'RuntimeError: metadata unavailable' if name == 'ownership-error' else 'KeyboardInterrupt: metadata interrupted'
        require(records[0]['error'] == records[2]['error'] == expected, 'Python environment observation control failed')
    elif name == 'version-error':
        require(not records[0]['ok'] and records[0]['error'].startswith('PackageNotFoundError:'), 'Python environment observation control failed')
        require(records[1]['ok'] and records[1]['distributions'] == [{'name': 'good', 'version': '1.0'}], 'Python environment observation control failed')
    elif name in ['mutation', 'failed-mutation']:
        require(str(root / 'new-site') in payload['sys_path'], 'Python environment observation control failed')
        require(all((r['distributions'] == [{'name': 'new', 'version': '2.0'}] for r in records if r['root'] == 'probe_early')), 'Python environment observation control failed')
        require(records[1]['ok'] == (name == 'mutation'), 'Python environment observation control failed')
        if name == 'failed-mutation':
            require(records[1]['error'] == 'RuntimeError: mutation before import failure', 'Python environment observation control failed')
    elif name == 'namespace':
        expected = [{'name': 'local-probe-first', 'version': (root / 'expected-version').read_text()}, {'name': 'local-probe-second', 'version': '3.0'}]
        require(all((r['ok'] and r['origin'] is None and (r['distributions'] == expected) for r in records)), records)
    print(json.dumps({'name': name, 'status': 'pass', 'mapping_calls': len(mapping_calls) if name != 'namespace' else 'real distribution discovery', 'payload': payload}))

def run_probe_observation_self_tests(paths) -> None:
    # A new selected-interpreter process per observation prevents hidden memoization.
    from env import require_canonical_python

    require_canonical_python(paths.area_root)
    records = []
    with tempfile.TemporaryDirectory(prefix="sifr-python-probe-controls-") as scratch:
        root = Path(scratch)
        (root / "pyproject.toml").write_text('[project]\nname="probe-control"\nversion="1.0"\n')
        (root / "uv.lock").write_text("control-lock-v1\n")
        for side, version in (("first", "1.0"), ("second", "3.0")):
            site = root / side
            (site / "local_probe_ns").mkdir(parents=True)
            info = site / f"local_probe_{side}-{version}.dist-info"
            info.mkdir()
            (info / "METADATA").write_text(
                f"Metadata-Version: 2.1\nName: local-probe-{side}\nVersion: {version}\n"
            )
            (info / "top_level.txt").write_text("local_probe_ns\n")
        (root / "expected-version").write_text("1.0")
        for index, name in enumerate((*CASES, "namespace")):
            if index == len(CASES):
                metadata = root / "first/local_probe_first-1.0.dist-info/METADATA"
                metadata.write_text(
                    "Metadata-Version: 2.1\nName: local-probe-first\nVersion: 2.0\n"
                )
                (root / "expected-version").write_text("2.0")
                (root / "uv.lock").write_text("control-lock-v2\n")
            command = [
                sys.executable, *(["-O"] if sys.flags.optimize else []), "-B", "-I",
                str(Path(__file__).resolve()), "--child",
                name, str(root), str(paths.repo_root),
            ]
            result = subprocess.run(command, capture_output=True, text=True, timeout=120)
            require(result.returncode == 0, f"{name}: {result.stderr}")
            records.append(json.loads(result.stdout))
        require(
            records[-1]["payload"]["uv_lock_digest"] != records[-2]["payload"]["uv_lock_digest"],
            "second invocation reused the previous lock observation",
        )
    print(f"python environment observation controls ok: {len(records)}")


if __name__ == "__main__":
    if len(sys.argv) != 5 or sys.argv[1] != "--child":
        raise SystemExit("run through the canonical Python interop self-test")
    child(sys.argv[2], sys.argv[3], sys.argv[4])
