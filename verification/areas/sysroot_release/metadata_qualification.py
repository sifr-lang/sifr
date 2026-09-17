"""Exact installed metadata, public emit, relocation and live-generation qualification."""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shlex
import shutil
import sys

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "verification/runner"))
sys.path.insert(0, str(ROOT / "verification/areas/developer_tooling"))
from sifr_verify.process_execution import execute
from lsp_protocol import LspClient, file_uri
from lsp_protocol_smoke import initialize, open_document


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


class Qualification:
    def __init__(self, binary, corpus, output):
        self.binary, self.corpus, self.output = binary.resolve(), corpus, output
        output.mkdir(parents=True, exist_ok=False)
        self.env = os.environ.copy()
        self.env.pop("SIFR_SYSROOT", None)
        self.rows = []
        self.report = {"binary": str(self.binary), "binary_sha256": digest(binary), "rows": self.rows}
        self.save()

    def save(self):
        (self.output / "report.json").write_text(json.dumps(self.report, indent=2) + "\n")

    def run(self, name, arguments, success=True, env=None):
        command = [str(value) for value in arguments]
        result = execute(command, cwd=self.output, env=env or self.env,
                         deadline_seconds=900, limit_bytes=16 * 1024 * 1024)
        (self.output / (name + ".stdout")).write_bytes(result.stdout)
        (self.output / (name + ".stderr")).write_bytes(result.stderr)
        self.rows.append({"id": name, "command": command, "cwd": str(self.output),
                          "exit_code": result.returncode, "cause": result.cause,
                          "elapsed_seconds": result.elapsed_seconds,
                          "stdout_sha256": hashlib.sha256(result.stdout).hexdigest(),
                          "stderr_sha256": hashlib.sha256(result.stderr).hexdigest()})
        self.save()
        assert not result.truncated and result.cause == "exit", (name, result)
        assert (result.returncode == 0) == success, (name, result.stderr.decode(errors="replace")[-3000:])
        return result

    def installed(self):
        original = self.binary.parent.parent
        first = self.output / "generation-a"
        shutil.copytree(original, first)
        moved = self.output / "moved-generation-a"
        first.rename(moved)
        binary = moved / "bin/sifr"
        metadata = moved / "lib/sifr/stdlib.sifrmeta"
        before = digest(metadata)
        full = json.loads(self.run("installed-structural", [binary, "sysroot", "validate-metadata"]).stdout)
        assert full["metadata_id"] == before
        doctor = json.loads(self.run("installed-integrity", [binary, "doctor", "--json", "--verify-integrity"]).stdout)
        assert doctor["status"] == "ok" and doctor["package_integrity_verified"]
        assert doctor["metadata"]["metadata_id"] == before
        coverage = json.loads((self.corpus / "coverage.json").read_text())
        assert not coverage["failures"] and coverage.get("selection") is None
        assert coverage["portable_payload_sha256"] == full["portable_payload_sha256"], "packaged portable payload differs from the corpus provider"
        self.report["source_corpus_sha256"] = digest(self.corpus / "coverage.json")
        self.report["installed_metadata_id"] = before
        self.report["portable_payload_sha256"] = full["portable_payload_sha256"]
        for row in coverage["rows"]:
            name = row["fixture"]
            source = self.corpus / (name + ".sifr")
            expected = (self.corpus / (name + ".source.rs")).read_bytes()
            result = self.run("emit-" + name, [binary, "emit", source])
            assert result.stdout == expected, ("public emit bytes differ", name)
            assert result.stdout.endswith(b"\n") == row["final_newline"]
            start = result.stdout.find(b"// --- stdlib:")
            end = result.stdout.find(b"\n// --- end stdlib ---")
            assert (None if start < 0 else start) == row["preamble_start"], name
            assert (None if end < 0 else end) == row["preamble_end"], name
            for mapping in row["generated_source_maps"]:
                mapped = result.stdout
                if mapping["path"].endswith("#stdlib-preamble"):
                    assert start >= 0 and end >= start, name
                    mapped = result.stdout[start:end].decode().rstrip().encode() + b"\n"
                assert len(mapped) == mapping["source_bytes"], name
                assert hashlib.sha256(mapped).hexdigest() == mapping["source_sha256"], name
        assert digest(metadata) == before, "qualification replaced the consumed metadata"
        self.report["public_emit_cases"] = len(coverage["rows"])
        self.save()
        self.live_generations(moved)
        self.rejection_recovery(moved)
        self.report["status"] = "passed"
        self.save()

    def rejection_recovery(self, root):
        binary = root / "bin/sifr"
        descriptor = root / "lib/sifr/stdlib.metadata.json"
        metadata = root / "lib/sifr/stdlib.sifrmeta"
        original = descriptor.read_bytes()
        value = json.loads(original)
        value["compiler_identity"] = "0" * 64
        try:
            descriptor.write_text(json.dumps(value))
            report = json.loads(self.run("incompatible-doctor", [binary, "doctor", "--json"], False).stdout)
            assert report["status"] == "error" and "reinstall" in report["message"]
        finally:
            descriptor.write_bytes(original)
        held = metadata.with_suffix(".held")
        metadata.rename(held)
        try:
            self.run("missing-metadata-doctor", [binary, "doctor", "--json"], False)
            metadata.write_bytes(b"incomplete")
            self.run("corrupt-metadata-doctor", [binary, "doctor", "--json"], False)
        finally:
            metadata.unlink(missing_ok=True)
            held.rename(metadata)
        self.run("restored-doctor", [binary, "doctor", "--json", "--verify-integrity"])
        # Ordinary checking consumes semantic records without reading navigation sources.
        source = root / "lib/sifr/stdlib/_sifr/math.sifr"
        held = source.with_suffix(".held")
        source.rename(held)
        case = self.output / "tiny.sifr"
        case.write_text("from sifr.math import sqrt\n\ndef main():\n    assert sqrt(4.0) == 2.0\n")
        try:
            self.run("lazy-check-without-source", [binary, "check", case])
        finally:
            held.rename(source)

    def live_generations(self, first):
        second = self.output / "generation-b"
        shutil.copytree(first, second)
        snapshot = self.output / "producer-source"
        snapshot.mkdir()
        shutil.copytree(ROOT / "stdlib", snapshot / "stdlib")
        for relative in ("sysroot.toml", "Cargo.toml", "Cargo.lock", ".cargo/config.toml",
                         "crates/sifr_runtime/Cargo.toml", "crates/sifr_stdlib/Cargo.toml",
                         "crates/sifr_structural_identity/Cargo.toml"):
            target = snapshot / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(ROOT / relative, target)
        (snapshot / "vendor").mkdir()
        source = snapshot / "stdlib/_sifr/math.sifr"
        source.write_text("# generation B navigation marker\n" + source.read_text())
        shutil.rmtree(second / "lib/sifr/stdlib")
        shutil.copytree(snapshot / "stdlib", second / "lib/sifr/stdlib")
        descriptor_path = second / "lib/sifr/stdlib.metadata.json"
        descriptor = json.loads(descriptor_path.read_text())
        metadata = second / "lib/sifr/stdlib.sifrmeta"
        report = json.loads(self.run("generation-b-production", [
            self.binary, "sysroot", "build-metadata", "--source-root", snapshot,
            "--output", metadata, "--target", descriptor["semantic_target"]]).stdout)
        descriptor["metadata_id"] = report["metadata_id"]
        descriptor["stdlib_inputs_id"] = metadata.read_bytes()[80:112].hex()
        descriptor_path.write_text(json.dumps(descriptor, sort_keys=True, indent=2) + "\n")
        files = []
        for relative in ("Cargo.toml", "Cargo.lock", ".cargo/config.toml", "crates", "lib", "vendor"):
            path = second / relative
            files.extend([path] if path.is_file() else [p for p in path.rglob("*") if p.is_file()])
        receipt = "".join(str(p.relative_to(second)) + "\n" + digest(p) + "\n" for p in sorted(files))
        manifest = second / "sysroot.toml"
        text = re.sub(r'("sysroot-content-sha256"\s*=\s*")[^"]+',
                      lambda m: m[1] + hashlib.sha256(receipt.encode()).hexdigest(), manifest.read_text())
        manifest.write_text(text)
        self.run("generation-b-integrity", [second / "bin/sifr", "doctor", "--json", "--verify-integrity"])
        selector = self.output / "current"
        selector.symlink_to(first, target_is_directory=True)
        workspace = self.output / "workspace"
        workspace.mkdir()
        source = workspace / "main.sifr"
        source.write_text("from sifr.math import sqrt\n\ndef main():\n    value: float = sqrt(4.0)\n")
        old_env = {key: os.environ.get(key) for key in ("SIFR_SYSROOT", "SIFR_LSP_COMMAND")}
        os.environ["SIFR_SYSROOT"] = str(selector)
        os.environ["SIFR_LSP_COMMAND"] = shlex.join([str(first / "bin/sifr"), "lsp", "--stdio"])
        client = LspClient()
        def switch(root):
            pending = self.output / "pending"
            pending.symlink_to(root, target_is_directory=True)
            pending.replace(selector)
        def definition(current):
            response = current.request("textDocument/definition", {
                "textDocument": {"uri": file_uri(source)}, "position": {"line": 0, "character": 23}})
            item = response[0] if isinstance(response, list) else response
            return item.get("targetUri", item.get("uri")), item.get("targetSelectionRange", item.get("range"))
        try:
            initialize(client, workspace)
            open_document(client, source, source.read_text())
            switch(second)
            old_uri, old_range = definition(client)
            assert first.as_uri() in old_uri, old_uri
            newer = LspClient()
            try:
                initialize(newer, workspace)
                open_document(newer, source, source.read_text())
                new_uri, new_range = definition(newer)
                assert second.as_uri() in new_uri, new_uri
                assert new_range["start"]["line"] == old_range["start"]["line"] + 1
            finally:
                newer.close()
            switch(first)
            assert definition(client) == (old_uri, old_range)
            self.report["lsp_generations"] = {"old": old_uri, "new": new_uri,
                                             "old_range": old_range, "new_range": new_range,
                                             "rollback_preserved": True}
            self.save()
        finally:
            client.close()
            for key, value in old_env.items():
                if value is None:
                    os.environ.pop(key, None)
                else:
                    os.environ[key] = value


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--corpus", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    Qualification(args.binary, args.corpus, args.output).installed()


if __name__ == "__main__":
    main()
