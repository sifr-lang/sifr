"""Native, nonpublishing qualification of exact installed package generations.

The network transport is an allowlisted local fixture. The packaged updater,
installer, integrity checks and native compiler execute unchanged. The update is
an explicit same-version forced reinstall into a new immutable generation; it
does not claim a published version upgrade.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "verification/runner"))
sys.path.insert(0, str(ROOT / "verification/areas/sysroot_release"))
from sifr_verify.process_execution import execute
from producer_snapshot import prepare_source_snapshot
from metadata_qualification import Qualification
from qualify_stable_target import current_host_target

PUBLIC = "https://github.com/sifr-lang/sifr/releases/download"
TARGETS = ("aarch64-apple-darwin", "x86_64-apple-darwin",
           "x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu")


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


class NativePackage:
    def __init__(self, artifacts, installer, version, target, source, output):
        self.artifacts = artifacts.resolve()
        self.installer = installer.resolve()
        self.version, self.target, self.source = version, target, source
        self.output = output.resolve()
        self.output.mkdir(parents=True, exist_ok=False)
        require(target == current_host_target(), "native host does not match target")
        actual = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
        require(actual == source, "checkout differs from exact qualified source")
        subprocess.run(["git", "diff", "--quiet", "HEAD"], cwd=ROOT, check=True)
        self.env = dict(os.environ)
        for key in ("SIFR_SYSROOT", "SIFR_SYSROOT_MODE", "SIFR_TEST_CHANNEL_METADATA_PATH",
                    "SIFR_INSTALL_MANIFEST_DIR", "SIFR_CARGO", "SIFR_RUSTC"):
            self.env.pop(key, None)
        self.env.update(SIFR_CACHE_DIR=str(self.output / "cache"),
                        CARGO_NET_OFFLINE="true", SIFR_NO_MODIFY_PATH="1",
                        SIFR_ARTIFACT_BASE_URL=self.artifacts.as_uri())
        self.rows = []
        self.report = {"schema_version": 1, "source_commit": source, "target": target,
                       "version": version, "host": platform.uname()._asdict(),
                       "protocol_sha256": digest(__file__),
                       "installer_sha256": digest(installer), "rows": self.rows,
                       "update_scope": "same-version forced reinstall; distinct immutable generations",
                       "transport_scope": "allowlisted local fixture; no publication or live channel mutation",
                       "environment": {key: self.env.get(key) for key in (
                           "CARGO_BUILD_JOBS", "CARGO_TARGET_DIR", "CARGO_NET_OFFLINE",
                           "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTUP_TOOLCHAIN")},
                       "status": "running"}
        self.save()

    def save(self):
        (self.output / "native-package.json").write_text(json.dumps(self.report, indent=2) + "\n")

    def run(self, name, command, *, env=None, success=True, deadline=900, cwd=None):
        merged = self.env | (env or {})
        result = execute([str(v) for v in command], cwd=cwd or self.output, env=merged,
                         deadline_seconds=deadline, limit_bytes=16 * 1024 * 1024)
        (self.output / (name + ".stdout")).write_bytes(result.stdout)
        (self.output / (name + ".stderr")).write_bytes(result.stderr)
        self.rows.append({"id": name, "command": [str(v) for v in command],
                          "elapsed_seconds": result.elapsed_seconds,
                          "returncode": result.returncode, "cause": result.cause,
                          "stdout_sha256": hashlib.sha256(result.stdout).hexdigest(),
                          "stderr_sha256": hashlib.sha256(result.stderr).hexdigest()})
        self.save()
        require(not result.truncated and result.cause == "exit", f"{name}: incomplete execution")
        require((result.returncode == 0) == success,
                f"{name}: unexpected exit {result.returncode}: {result.stderr[-3000:]!r}")
        return result.stdout

    def integrity(self, name, binary):
        report = json.loads(self.run(name, [binary, "doctor", "--json", "--verify-integrity"]))
        require(report["status"] == "ok" and report["package_integrity_verified"],
                f"{name}: package integrity not verified")
        return report

    def prepare_transport(self):
        evidence = {}
        for target in TARGETS:
            path = self.artifacts / f"qualification-{target}.json"
            report = json.loads(path.read_text())
            archive = self.artifacts / f"sifr-{self.version}-{target}.tar.gz"
            require(report["source_commit"] == self.source and report["target"] == target
                    and report["candidate_version"] == self.version
                    and report["smoke_status"] == "pass", f"{target}: mismatched target report")
            require(digest(archive) == report["archive_sha256"], f"{target}: archive digest mismatch")
            require(Path(str(archive) + ".sha256").read_text().strip() == digest(archive),
                    f"{target}: checksum mismatch")
            evidence[target] = {"artifact_sha256": digest(archive),
                                "sysroot_content_sha256": report["sysroot_sha256"]}
        self.report["archives"] = evidence
        # Alpha/beta records only satisfy the channel schema. They are unused
        # schema fixtures, never selected or treated as qualified packages.
        releases = {}
        channels = {"alpha": "0.1.0-alpha.1", "beta": "0.1.0-beta.1", "stable": self.version}
        for channel, version in channels.items():
            releases[version] = {"channel": channel, "status": "active",
                                 "source_commit": self.source,
                                 "installer_sha256": digest(self.installer), "targets": evidence}
        metadata = self.output / "channels-fixture.json"
        metadata.write_text(json.dumps({"schema_version": 2, "generation": 1,
                                        "ga_status": "active", "channels": channels,
                                        "releases": releases}) + "\n")
        mapping = {f"{PUBLIC}/channels/channels.json": str(metadata),
                   f"{PUBLIC}/{self.version}/sifr-installer-{self.version}": str(self.installer)}
        for target in TARGETS:
            archive = self.artifacts / f"sifr-{self.version}-{target}.tar.gz"
            mapping[archive.as_uri()] = str(archive)
        transport = self.output / "transport"
        transport.mkdir()
        routes = transport / "routes.json"
        routes.write_text(json.dumps(mapping))
        curl = transport / "curl"
        curl.write_text("#!" + sys.executable + "\n" + """import json,pathlib,sys
args=sys.argv[1:]
routes=json.loads(pathlib.Path(__file__).with_name("routes.json").read_text())
urls=[value for value in args if value.startswith(("https://","file://"))]
if len(urls)!=1 or urls[0] not in routes:
    raise SystemExit("qualification transport rejected an unlisted URL")
source=pathlib.Path(routes[urls[0]]).read_bytes()
if "-o" in args:
    pathlib.Path(args[args.index("-o")+1]).write_bytes(source)
else:
    sys.stdout.buffer.write(source)
""")
        curl.chmod(0o755)
        self.env["PATH"] = str(transport) + os.pathsep + self.env["PATH"]
        self.report["transport_sha256"] = digest(curl)
        self.report["channels_fixture_sha256"] = digest(metadata)
        self.save()

    def generation_checks(self):
        managed = self.output / "managed"
        install_env = {"SIFR_INSTALL_DIR": str(managed / "bin")}
        self.run("install", ["sh", self.installer, "--no-modify-path"], env=install_env)
        binary = managed / "bin/sifr"
        first = (managed / ".sifr-current").resolve(strict=True)
        archive_report = json.loads((self.artifacts / f"qualification-{self.target}.json").read_text())
        require(digest(binary) == archive_report["binary_sha256"], "installed binary differs from archive")
        first_integrity = self.integrity("install-integrity", binary)
        self.run("same-version-noop", [binary, "self", "update", "--version", self.version])
        require((managed / ".sifr-current").resolve() == first, "no-op replaced generation")
        self.run("forced-update", [binary, "self", "update", "--version", self.version, "--force"])
        second = (managed / ".sifr-current").resolve(strict=True)
        require(first != second and first.is_dir(), "update failed to retain distinct immutable generations")
        require(digest(first / "bin/sifr") == digest(binary), "forced reinstall changed compiler bytes")
        require(self.integrity("update-integrity", binary)["metadata"]["metadata_id"]
                == first_integrity["metadata"]["metadata_id"], "forced reinstall changed metadata identity")
        # Fail after the new selector is installed, at atomic receipt publication.
        # The real installer trap must roll the selector back to generation B.
        bad_manifest = self.output / "receipt-parent-is-file"
        bad_manifest.write_text("owned failure fixture\n")
        self.run("failed-update-rollback", ["sh", "-x", self.installer, "--force", "--no-modify-path"],
                 env=install_env | {"SIFR_INSTALL_MANIFEST_DIR": str(bad_manifest)}, success=False)
        trace = (self.output / "failed-update-rollback.stderr").read_text()
        require("rollback_install_transaction" in trace and str(bad_manifest) in trace,
                "failure did not exercise the installer receipt rollback path")
        require((managed / ".sifr-current").resolve() == second, "failed update did not restore selector")
        self.integrity("rollback-integrity", binary)
        moved = self.output / "relocated"
        managed.rename(moved)
        binary = moved / "bin/sifr"
        require(not managed.exists(), "old installation path still exists")
        self.integrity("relocated-integrity", binary)
        self.report["generations"] = {"first": first.name, "second": second.name,
                                      "rollback": second.name, "relocated": str(moved)}
        self.save()
        return binary

    def native_profiles(self, binary):
        cargo = shutil.which("cargo", path=self.env["PATH"])
        require(cargo is not None, "selected native qualification requires Cargo")
        wrapper = self.output / "cargo-events"
        wrapper.write_text("#!" + sys.executable + "\n" + """import json,os,subprocess,sys
result=subprocess.run([os.environ["QUALIFICATION_CARGO"],*sys.argv[1:]],stdout=subprocess.PIPE)
with open(os.environ["QUALIFICATION_CARGO_EVENTS"],"ab") as out:
    out.write(result.stdout)
sys.stdout.buffer.write(result.stdout)
raise SystemExit(result.returncode)
""")
        wrapper.chmod(0o755)
        source = self.output / "native.sifr"
        for stage, value in (("cold", 42), ("noop", 42), ("edit", 43), ("reused", 43)):
            source.write_text(f"def main():\n    print({value})\n")
            for profile in ("development", "release"):
                label = f"native-{stage}-{profile}"
                events = self.output / (label + ".cargo.jsonl")
                env = {"SIFR_CARGO": str(wrapper), "QUALIFICATION_CARGO": cargo,
                       "QUALIFICATION_CARGO_EVENTS": str(events)}
                args = [binary, "run", source]
                if profile == "release":
                    args.append("--release")
                result = self.run(label, args, env=env, deadline=2400)
                require(result == f"{value}\n".encode(), f"{label}: incorrect native result")
                units = []
                for line in events.read_text().splitlines():
                    try:
                        row = json.loads(line)
                    except ValueError:
                        continue
                    if isinstance(row, dict) and row.get("reason") == "compiler-artifact":
                        units.append(row)
                apps = [row for row in units if row.get("executable")]
                require(bool(apps), f"{label}: missing actual Cargo artifact evidence")
                for app in apps:
                    details = app["profile"]
                    require(details["overflow_checks"] and details["opt_level"] == ("3" if profile == "release" else "0")
                            and details["debug_assertions"] == (profile == "development"),
                            f"{label}: generated application profile mismatch")
                rebuilt = sum(not row["fresh"] for row in units)
                if stage in ("noop", "reused"):
                    require(rebuilt == 0, f"{label}: unchanged work rebuilt")
                elif stage == "edit":
                    require(rebuilt == 1, f"{label}: edit did not isolate application work")
                self.rows[-1]["cargo"] = {"rebuilt": rebuilt, "fresh": sum(row["fresh"] for row in units),
                                         "application_profiles": [row["profile"] for row in apps]}
                self.save()
        loader = ["otool", "-L", binary] if platform.system() == "Darwin" else ["ldd", binary]
        text = self.run("native-loader", loader).decode()
        require("not found" not in text and str(ROOT) not in text,
                "compiler loader refers to missing or source-tree libraries")
        self.report["native_profile_scope"] = "cold/noop/edit/reused development and release; cold work charged, no latency claim"
        self.save()

    def metadata_corpus(self, binary):
        snapshot = prepare_source_snapshot(ROOT, self.output / "producer-source", self.version)
        corpus = self.output / "corpus"
        env = {"SIFR_RELEASE_VERSION": self.version, "SIFR_SYSROOT": str(snapshot),
               "SIFR_DX8_CORPUS_OUTPUT": str(corpus), "SIFR_DX8_SEMANTIC_TARGET": self.target}
        command = ["cargo", "test", "--locked", "--offline", "-p", "sifr_driver", "--lib"]
        self.run("corpus-preparation", command + ["--no-run"], env=env, cwd=ROOT, deadline=2400)
        self.run("source-metadata-full-corpus", command + ["full_corpus_exact_emission", "--", "--ignored", "--nocapture"],
                 env=env, cwd=ROOT, deadline=2400)
        # Qualification consumes the canonical generation, not the facade whose
        # package members are symlinks. All copied bytes remain archive-identical.
        Qualification(binary.resolve(), corpus, self.output / "installed-metadata").installed()
        self.report["metadata_report_sha256"] = digest(self.output / "installed-metadata/report.json")
        self.save()

    def qualify(self):
        try:
            self.run("rustc-identity", ["rustc", "-vV"], deadline=90)
            self.run("cargo-identity", ["cargo", "-vV"], deadline=90)
            self.prepare_transport()
            binary = self.generation_checks()
            self.native_profiles(binary)
            self.metadata_corpus(binary)
            subprocess.run(["git", "diff", "--quiet", "HEAD"], cwd=ROOT, check=True)
            self.report["status"] = "pass"
        except BaseException as error:
            self.report["status"] = "fail"
            self.report["failure"] = str(error)
            raise
        finally:
            self.save()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--artifacts", type=Path, required=True)
    parser.add_argument("--installer", type=Path, required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--target", choices=TARGETS, required=True)
    parser.add_argument("--source-commit", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    NativePackage(args.artifacts, args.installer, args.version, args.target,
                  args.source_commit, args.output).qualify()


if __name__ == "__main__":
    main()

