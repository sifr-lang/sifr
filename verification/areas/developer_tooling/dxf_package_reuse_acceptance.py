"""Package-cwd importer reuse with prepared paired fresh references."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import statistics
import subprocess
import time

def fixture(root):
    root.mkdir(parents=True)
    (root / "Cargo.toml").write_text('[workspace]\nmembers = ["app", "dep"]\nresolver = "3"\n')
    for name in ("app", "dep"):
        p = root / name
        (p / "src").mkdir(parents=True)
        extra = '\n[dependencies]\ndep = { path = "../dep", package = "dxf-dep" }\n' if name == "app" else ""
        (p / "Cargo.toml").write_text(f'[package]\nname = "dxf-{name}"\nversion = "0.1.0"\nedition = "2024"\n[package.metadata.sifr]\nmanifest = "sifr.toml"\n' + extra)
        (p / "sifr.toml").write_text(f'[package]\nname = "{name}"\nedition = "2026"\nsifr-version = ">=0.3,<0.4"\n[source]\nroot = "src"\n')
        (p / "src/lib.rs").write_text("")
    (root / "dep/src/__init__.sifr").write_text("def value() -> int:\n    return 1\n")
    (root / "app/src/helper.sifr").write_text("def value() -> int:\n    return 1\n")
    imports = []
    for index in range(8):
        name = f"layer{index}"
        source = "from app.helper import value\nfrom dep import value as dep_value\n"
        source += "".join(f"def call{i}() -> int:\n    return value() + dep_value()\n" for i in range(20))
        (root / f"app/src/{name}.sifr").write_text(source)
        imports.append(f"from app.{name} import call0 as call{index}\n")
    (root / "app/src/main.sifr").write_text("".join(imports) + "from app.helper import value\ndef main() -> None:\n    print(value())\n" + "".join(f"    x{i}: int = call{i}()\n" for i in range(8)))
    return root / "app"

def run(binary, output, samples, baseline):
    output.mkdir(parents=True, exist_ok=False)
    cwd = fixture(output / "workspace")
    env = dict(os.environ, SIFR_CACHE_DIR=str(output / "cache"))
    report = {"binary": str(binary), "sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
              "baseline": baseline, "samples": samples, "warmups_excluded": 1, "cwd": str(cwd), "rows": []}
    def invoke(label, fresh=False, native=False, expected=0):
        args = [str(binary)]
        if not native:
            args += ["--diagnostic-format", "json", "--timings"]
        if fresh:
            args += ["--no-incremental"]
        args += ["run" if native else "check", "src/main.sifr"]
        rss = output / (label + ".rss")
        command = ["/usr/bin/time", "-f", "%M", "-o", str(rss), *args]
        started = time.monotonic()
        proc = subprocess.run(command, cwd=cwd, env=env, text=True, capture_output=True)
        elapsed = time.monotonic() - started
        (output / (label + ".stdout")).write_text(proc.stdout)
        (output / (label + ".stderr")).write_text(proc.stderr)
        cache = next((json.loads(x.split("] ", 1)[1]) for x in proc.stderr.splitlines() if x.startswith("[sifr-project-cache] ")), None)
        row = {"label": label, "command": args, "returncode": proc.returncode,
               "wall_seconds": elapsed, "rss_kib": int(rss.read_text().splitlines()[-1]), "cache": cache}
        if cache and cache["modules"]:
            row["module_counts"] = {action: sum(m["action"] == action for m in cache["modules"]) for action in ("computed", "restored")}
        report["rows"].append(row)
        (output / "report.json").write_text(json.dumps(report, indent=2) + "\n")
        assert proc.returncode == expected, (label, proc.stdout, proc.stderr)
        if native:
            return proc.stdout.strip(), row
        stream = proc.stdout.strip() or "\n".join(x for x in proc.stderr.splitlines() if not x.startswith(("[sifr-project-cache] ", "[sifr-metadata] ", "[sifr-timing] ")))
        return json.loads(stream), row
    invoke("prepare")
    for scenario, path in (("helper", cwd / "src/helper.sifr"), ("dependency", cwd.parent / "dep/src/__init__.sifr")):
        for sample in range(-1, samples):
            path.write_text(f"def value() -> int:\n    return {sample + 101}\n")
            pair = {}
            for fresh in ([False, True] if sample % 2 else [True, False]):
                name = "fresh" if fresh else "incremental"
                pair[name] = invoke(f"{scenario}-{sample}-{name}", fresh)
            assert pair["fresh"][0] == pair["incremental"][0]
            row = pair["incremental"][1]
            if not baseline:
                assert row["cache"]["status"] == "interface-restored", row
                assert row["module_counts"]["restored"] >= 8 and row["module_counts"]["computed"] >= 1, row
    summary = {}
    for scenario in ("helper", "dependency"):
        summary[scenario] = {}
        for lane in ("fresh", "incremental"):
            rows = [r for r in report["rows"] if r["label"].startswith(scenario + "-") and "--1-" not in r["label"] and r["label"].endswith(lane)]
            summary[scenario][lane] = {}
            for metric in ("wall_seconds", "rss_kib"):
                values = sorted(r[metric] for r in rows)
                summary[scenario][lane][metric] = {"median": statistics.median(values), "p95": values[int(.95 * (len(values) - 1))], "min": values[0], "max": values[-1], "cv": statistics.pstdev(values) / statistics.mean(values)}
            summary[scenario][lane]["validation_us"] = [r["cache"]["validation_us"] for r in rows]

    if not baseline:
        helper = cwd / "src/helper.sifr"
        good = "def value() -> int:\n    return 1\n"
        for label, before, after, error in [
            ("default", "def value(x: int = 1) -> int:\n    return x\n", "def value(x: int = 2) -> int:\n    return x\n", False),
            ("constant", "N: int = 1\ndef value() -> int:\n    return N\n", "N: int = 2\ndef value() -> int:\n    return N\n", False),
            ("signature", good, 'def value() -> str:\n    return "bad"\n', True),
            ("body-error", good, 'def value() -> int:\n    return "bad"\n', True),
        ]:
            helper.write_text(before)
            invoke(label + "-before")
            helper.write_text(after)
            actual, row = invoke(label + "-after", expected=int(error))
            fresh, _ = invoke(label + "-fresh", fresh=True, expected=int(error))
            assert actual == fresh and row["cache"]["status"] != "interface-restored", row
        helper.write_text(good)
        invoke("native-one-check")
        actual, _ = invoke("native-one", native=True)
        assert actual == "1", actual
        helper.write_text("def value() -> int:\n    return 2\n")
        invoke("native-two-check")
        actual, _ = invoke("native-two", native=True)
        assert actual == "2", actual
        report["native_output"] = ["1", "2"]
        manifest = cwd / "sifr.toml"
        original = manifest.read_text()
        manifest.write_text(original + '\n[trust]\nsecurity-capabilities = ["sql.unsafe-syntax"]\n')
        actual, row = invoke("trust-change")
        fresh, _ = invoke("trust-change-fresh", fresh=True)
        assert actual == fresh and row["cache"]["restored_checks"] == 0, row
        manifest.write_text(original)
        shadow = cwd / "src/helper"
        shadow.mkdir()
        (shadow / "__init__.sifr").write_text(good)
        actual, _ = invoke("resolver-ambiguity", expected=1)
        fresh, _ = invoke("resolver-ambiguity-fresh", fresh=True, expected=1)
        assert actual == fresh and actual
        (shadow / "__init__.sifr").unlink()
        shadow.rmdir()
        # A malformed current lock is rejected by Cargo before cache lookup.
        lock = cwd.parent / "Cargo.lock"
        lock_before = lock.read_text()
        lock.write_text("not valid toml [")
        actual, row = invoke("invalid-lock", expected=1)
        fresh, _ = invoke("invalid-lock-fresh", fresh=True, expected=1)
        assert actual == fresh and actual and row["cache"] is None
        lock.write_text(lock_before)

    report["summary"] = summary
    (output / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(summary, indent=2))

if __name__ == "__main__":
    p = argparse.ArgumentParser()
    p.add_argument("--binary", type=Path, required=True)
    p.add_argument("--output", type=Path, required=True)
    p.add_argument("--samples", type=int, default=21)
    p.add_argument("--baseline", action="store_true")
    a = p.parse_args()
    run(a.binary.resolve(), a.output.resolve(), a.samples, a.baseline)

