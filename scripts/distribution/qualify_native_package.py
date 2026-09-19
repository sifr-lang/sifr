"""Native, nonpublishing qualification of exact installed package generations.

The network transport is an allowlisted local fixture. The packaged updater,
installer, integrity checks and native compiler execute unchanged. Same-source fixture packages exercise an actual lower-version installation,
upgrade, forced downgrade, reinstall and receipt-failure rollback. No version
or channel is published.
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
import tomllib

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "verification/runner"))
sys.path.insert(0, str(ROOT / "verification/areas/sysroot_release"))
from sifr_verify.process_execution import execute
from producer_snapshot import prepare_source_snapshot
from metadata_qualification import Qualification
from qualify_stable_target import current_host_target
from qualify_stable_editor import parse_version, range_contains

PUBLIC = "https://github.com/sifr-lang/sifr/releases/download"
TARGETS = ("aarch64-apple-darwin", "x86_64-apple-darwin",
           "x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu")


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def transition_fixture_version(root, candidate, rollback):
    """Admit actual source configuration before any expensive native build."""
    previous = tomllib.loads((root / "crates/sifr/Cargo.toml").read_text())["package"]["version"]
    candidate_tuple = parse_version(candidate, "candidate version")
    require(parse_version(previous, "fixture version") < candidate_tuple,
            "transition fixture must be older than candidate")
    editor = json.loads((root / "editor_integrations/vscode/package.json").read_text())
    compatibility = editor["sifrCompilerCompatibility"]
    require(range_contains(compatibility, candidate_tuple),
            "candidate is outside the checked-in editor compatibility range")
    if rollback != "none":
        require(range_contains(compatibility, parse_version(rollback, "rollback version")),
                "rollback is outside the checked-in editor compatibility range")
    return previous


class NativePackage:
    def __init__(self, artifacts, installer, version, target, source, output,
                 previous_artifacts, previous_installer, previous_version):
        self.artifacts = artifacts.resolve()
        self.installer = installer.resolve()
        self.version, self.target, self.source = version, target, source
        self.previous_artifacts = previous_artifacts.resolve()
        self.previous_installer = previous_installer.resolve()
        self.previous_version = previous_version
        require(parse_version(previous_version, "fixture version")
                < parse_version(version, "candidate version"),
                "transition fixture must be older than candidate")
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
                       "update_scope": "real version upgrade and forced downgrade, same-version reinstall and transaction rollback",
                       "previous_version": previous_version,
                       "previous_installer_sha256": digest(previous_installer),
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
        versions = ((self.version, self.artifacts, self.installer),
                    (self.previous_version, self.previous_artifacts, self.previous_installer))
        packages, mapping, releases = {}, {}, {}
        for version, artifacts, installer in versions:
            evidence = {}
            for target in TARGETS:
                path = artifacts / f"qualification-{target}.json"
                report = json.loads(path.read_text())
                archive = artifacts / f"sifr-{version}-{target}.tar.gz"
                require(report["source_commit"] == self.source and report["target"] == target
                        and report["candidate_version"] == version
                        and report["smoke_status"] == "pass", f"{target}: mismatched target report")
                require(digest(archive) == report["archive_sha256"], f"{target}: archive digest mismatch")
                require(Path(str(archive) + ".sha256").read_text().strip() == digest(archive),
                        f"{target}: checksum mismatch")
                evidence[target] = {"artifact_sha256": digest(archive),
                                    "sysroot_content_sha256": report["sysroot_sha256"]}
                mapping[archive.as_uri()] = str(archive)
            packages[version] = evidence
            mapping[f"{PUBLIC}/{version}/sifr-installer-{version}"] = str(installer)
            releases[version] = {"channel": "stable", "status": "active",
                                 "source_commit": self.source,
                                 "installer_sha256": digest(installer), "targets": evidence}
        self.report["packages"] = packages
        # Unused alpha/beta rows satisfy the existing public channel schema.
        # Only the two exact, locally qualified stable versions are selected.
        channels = {"alpha": "0.1.0-alpha.1", "beta": "0.1.0-beta.1", "stable": self.version}
        for channel in ("alpha", "beta"):
            releases[channels[channel]] = releases[self.version] | {"channel": channel}
        metadata = self.output / "channels-fixture.json"
        metadata.write_text(json.dumps({"schema_version": 2, "generation": 1,
                                        "ga_status": "active", "channels": channels,
                                        "releases": releases}) + "\n")
        mapping[f"{PUBLIC}/channels/channels.json"] = str(metadata)
        transport = self.output / "transport"
        transport.mkdir()
        (transport / "routes.json").write_text(json.dumps(mapping))
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
        previous_env = {"SIFR_ARTIFACT_BASE_URL": self.previous_artifacts.as_uri()}
        self.run("install-previous", ["sh", self.previous_installer, "--no-modify-path"],
                 env=install_env | previous_env)
        binary = managed / "bin/sifr"
        generations = []
        def verify(label, version, artifacts):
            selected = (managed / ".sifr-current").resolve(strict=True)
            report = json.loads((artifacts / f"qualification-{self.target}.json").read_text())
            require(digest(binary) == report["binary_sha256"], f"{label}: binary differs from archive")
            require(self.run(label + "-version", [binary, "--version"]).strip()
                    == f"sifr {version}".encode(), f"{label}: wrong selected version")
            integrity = self.integrity(label + "-integrity", binary)
            generations.append({"step": label, "version": version, "generation": selected.name,
                                "binary_sha256": digest(binary),
                                "metadata_id": integrity["metadata"]["metadata_id"]})
            self.report["generations"] = generations
            self.save()
            return selected
        first = verify("previous", self.previous_version, self.previous_artifacts)
        self.run("upgrade", [binary, "self", "update", "--version", self.version])
        upgraded = verify("upgraded", self.version, self.artifacts)
        require(upgraded != first and first.is_dir(), "upgrade did not retain the old generation")
        self.run("version-rollback", [binary, "self", "update", "--version",
                                     self.previous_version, "--force"], env=previous_env)
        rolled_back = verify("version-rollback", self.previous_version, self.previous_artifacts)
        require(rolled_back != upgraded and upgraded.is_dir(), "downgrade mutated an existing generation")
        self.run("upgrade-after-rollback", [binary, "self", "update", "--version", self.version])
        current = verify("upgrade-after-rollback", self.version, self.artifacts)
        self.run("same-version-noop", [binary, "self", "update", "--version", self.version])
        require((managed / ".sifr-current").resolve() == current, "no-op replaced generation")
        self.run("forced-reinstall", [binary, "self", "update", "--version", self.version, "--force"])
        reinstalled = verify("forced-reinstall", self.version, self.artifacts)
        require(current != reinstalled and current.is_dir(), "reinstall failed to retain distinct generations")
        require(digest(current / "bin/sifr") == digest(binary), "reinstall changed compiler bytes")
        # Fail after the new selector is installed, at atomic receipt publication.
        # The real installer trap must roll back to the previous current generation.
        bad_manifest = self.output / "receipt-parent-is-file"
        bad_manifest.write_text("owned failure fixture\n")
        self.run("failed-update-rollback", ["sh", "-x", self.installer, "--force", "--no-modify-path"],
                 env=install_env | {"SIFR_INSTALL_MANIFEST_DIR": str(bad_manifest)}, success=False)
        trace = (self.output / "failed-update-rollback.stderr").read_text()
        require("rollback_install_transaction" in trace and str(bad_manifest) in trace,
                "failure did not exercise the installer receipt rollback path")
        require((managed / ".sifr-current").resolve() == reinstalled,
                "failed update did not restore selector")
        verify("transaction-rollback", self.version, self.artifacts)
        moved = self.output / "relocated"
        managed.rename(moved)
        binary = moved / "bin/sifr"
        require(not managed.exists(), "old installation path still exists")
        self.integrity("relocated-integrity", binary)
        self.report["relocated"] = str(moved)
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
    parser.add_argument("--previous-artifacts", type=Path, required=True)
    parser.add_argument("--previous-installer", type=Path, required=True)
    parser.add_argument("--previous-version", required=True)
    args = parser.parse_args()
    NativePackage(args.artifacts, args.installer, args.version, args.target,
                  args.source_commit, args.output, args.previous_artifacts,
                  args.previous_installer, args.previous_version).qualify()


if __name__ == "__main__":
    main()

