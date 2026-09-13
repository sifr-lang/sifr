"""Explicit SQLx clean-cache experiment; warm-up and frozen results stay separate."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time

from _scenario_lock_checks import SQLX_QUERY_MACRO_FIXTURE_ONLY_LOCK_PACKAGES

ROOT = Path(__file__).resolve().parents[4]
FIXTURE = ROOT / "verification/areas/rust_interop/fixtures/ecosystem_backend_certification/examples/backend_feature_package/Cargo.toml"


def sparse_path(name: str) -> str:
    if len(name) == 1:
        return f"1/{name}"
    if len(name) == 2:
        return f"2/{name}"
    if len(name) == 3:
        return f"3/{name[0]}/{name}"
    return f"{name[:2]}/{name[2:4]}/{name}"


def summaries(home: Path) -> list[dict[str, object]]:
    result = []
    for name, version, source, checksum in sorted(SQLX_QUERY_MACRO_FIXTURE_ONLY_LOCK_PACKAGES):
        paths = list((home / "registry/index").glob(f"*/.cache/{sparse_path(name)}"))
        if len(paths) > 1:
            raise ValueError(f"ambiguous sparse summary for {name}")
        raw = paths[0].read_bytes() if paths else b""
        matching = []
        for field in raw.split(b"\0"):
            if field.startswith(b"{"):
                row = json.loads(field)
                if row.get("vers") == version:
                    matching.append(row)
        if matching and (len(matching) != 1 or matching[0].get("cksum") != checksum):
            raise ValueError(f"official cached identity differs for {name} {version}")
        result.append({"name": name, "version": version, "source": source,
                       "expected_checksum": checksum, "present": bool(matching),
                       "path": str(paths[0]) if paths else None,
                       "summary_sha256": hashlib.sha256(raw).hexdigest() if raw else None})
    return result


def experiment(cargo: Path, home: Path, result_path: Path) -> int:
    if not cargo.is_absolute() or not cargo.is_file() or not home.is_absolute():
        raise ValueError("absolute existing Cargo and new absolute CARGO_HOME required")
    home.mkdir()  # Existing/ambient caches are never accepted or cleaned.
    environment = os.environ.copy()
    environment.update(CARGO_HOME=str(home), CARGO_NET_OFFLINE="false",
                       CARGO_NET_RETRY="0", CARGO_HTTP_TIMEOUT="30")
    locks = [ROOT / "Cargo.lock", FIXTURE.with_name("Cargo.lock")]
    hashes = {str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in locks}
    report: dict[str, object] = {"cargo": str(cargo), "cargo_sha256": hashlib.sha256(cargo.read_bytes()).hexdigest(),
                               "cargo_home": str(home), "initial_summaries": summaries(home),
                               "locks": hashes, "steps": [], "status": "FAIL"}
    steps = report["steps"]
    assert isinstance(steps, list)
    began = time.monotonic()
    def run(label: str, args: list[str]) -> int:
        command = [str(cargo), *args]
        started = time.monotonic()
        remaining = 600 - (started - began)
        if remaining <= 0:
            raise TimeoutError("clean-cache experiment whole budget")
        output = subprocess.run(command, cwd=ROOT, env=environment, capture_output=True,
                                text=True, timeout=min(300, remaining), check=False)
        steps.append({"label": label, "argv": command, "exit": output.returncode,
                      "seconds": time.monotonic() - started,
                      "stdout": output.stdout, "stderr": output.stderr})
        return output.returncode
    try:
        if run("root-warm-up-fetch", ["fetch", "--locked", "--manifest-path", str(ROOT / "Cargo.toml")]):
            return 1
        report["after_root_fetch_summaries"] = summaries(home)
        probe = ["metadata", "--locked", "--offline", "--frozen", "--format-version", "1", "--manifest-path", str(FIXTURE)]
        first = run("fixture-frozen-after-root-fetch", probe)
        report["root_fetch_alone_suffices"] = first == 0
        if first:
            # This is a distinct changed-input warm-up, not an offline retry.
            if run("fixture-warm-up-fetch", ["fetch", "--locked", "--manifest-path", str(FIXTURE)]):
                return 1
            report["after_fixture_fetch_summaries"] = summaries(home)
            if run("fixture-frozen-after-fixture-warm-up", probe):
                return 1
        report["status"] = "PASS"
        return 0
    finally:
        report["seconds"] = time.monotonic() - began
        report["locks_unchanged"] = all(hashlib.sha256(Path(path).read_bytes()).hexdigest() == digest for path, digest in hashes.items())
        if not report["locks_unchanged"]:
            report["status"] = "FAIL"
        with result_path.open("x") as stream:
            json.dump(report, stream, indent=2, sort_keys=True)
            stream.write("\n")
        if not report["locks_unchanged"]:
            raise ValueError("clean-cache qualification changed an input lock")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--cargo", type=Path)
    parser.add_argument("--cargo-home", type=Path)
    parser.add_argument("--result-json", type=Path)
    args = parser.parse_args()
    if args.self_test:
        assert sparse_path("sqlx-mysql") == "sq/lx/sqlx-mysql"
        assert len(SQLX_QUERY_MACRO_FIXTURE_ONLY_LOCK_PACKAGES) == 5
        assert len({x[0] for x in SQLX_QUERY_MACRO_FIXTURE_ONLY_LOCK_PACKAGES}) == 5
        with tempfile.TemporaryDirectory(prefix="sqlx-cache-contract-") as raw:
            home = Path(raw)
            assert not any(row["present"] for row in summaries(home))
            for name, version, _, checksum in SQLX_QUERY_MACRO_FIXTURE_ONLY_LOCK_PACKAGES:
                path = home / "registry/index/synthetic/.cache" / sparse_path(name)
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(b"header\0" + json.dumps({"vers": version, "cksum": checksum}).encode() + b"\0")
            assert all(row["present"] for row in summaries(home))
            name, version, _, _ = sorted(SQLX_QUERY_MACRO_FIXTURE_ONLY_LOCK_PACKAGES)[0]
            path = home / "registry/index/synthetic/.cache" / sparse_path(name)
            path.write_bytes(json.dumps({"vers": version, "cksum": "wrong"}).encode())
            try:
                summaries(home)
            except ValueError:
                pass
            else:
                raise AssertionError("wrong inactive package checksum accepted")
        print("SQLx clean-cache isolation contract self-test: PASS; no experiment executed")
        return 0
    if not all((args.cargo, args.cargo_home, args.result_json)):
        parser.error("experiment requires --cargo, --cargo-home, --result-json")
    return experiment(args.cargo, args.cargo_home, args.result_json)


if __name__ == "__main__":
    raise SystemExit(main())
