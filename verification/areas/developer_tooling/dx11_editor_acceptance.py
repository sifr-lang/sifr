"""DX.11 E01-E06 through the installed LSP transport; never builds a compiler."""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess

from lsp_protocol import LspClient
from lsp_protocol_smoke import initialize

GOOD = 'def main():\n    value: int = 1\n'
BAD = 'def main():\n    value: int = "bad"\n'


def file_uri(path):
    # Preserve the client's logical symlink URI; Path.resolve would hide E03.
    return path.absolute().as_uri()


def open_source(client, path, source, version=1):
    client.notify("textDocument/didOpen", {"textDocument": {
        "uri": file_uri(path), "languageId": "sifr", "version": version, "text": source}})


def diagnostics(client, path):
    return client.request("textDocument/diagnostic", {"textDocument": {"uri": file_uri(path)}})["items"]


def close_source(client, path):
    client.notify("textDocument/didClose", {"textDocument": {"uri": file_uri(path)}})
    client.request("workspace/diagnostic", {})


def stats(client):
    trace = client.request("sifr/debugTrace", {})
    return json.loads(trace.rsplit("\nmetadata_stats=", 1)[1]) if "\nmetadata_stats=" in trace else None


def finish(client):
    try:
        client.request("shutdown")
    finally:
        client.close()


def run(binary, output, no_incremental=False):
    output.mkdir(parents=True, exist_ok=False)
    flags = ["--no-incremental"] if no_incremental else []
    os.environ["SIFR_LSP_COMMAND"] = shlex.join([str(binary), *flags, "lsp", "--stdio"])
    os.environ["SIFR_CACHE_DIR"] = str(output / "cache")
    report = {"binary": str(binary), "sha256": hashlib.sha256(binary.read_bytes()).hexdigest(), "cases": {}}
    def record(name, detail):
        report["cases"][name] = detail
        (output / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    root = output / "workspace"
    root.mkdir()
    path = root / "main.sifr"
    path.write_text(GOOD)
    seed = subprocess.run([str(binary), *flags, "--timings", "check", str(path)],
                          cwd=root, capture_output=True)
    assert seed.returncode == 0, (seed.stdout, seed.stderr)
    (output / "saved-seed.stderr").write_bytes(seed.stderr)
    client = LspClient(cwd=root)
    try:
        initialize(client, root)
        open_source(client, path, GOOD)
        assert not diagnostics(client, path)
        trace = client.request("sifr/debugTrace", {})
        restored = int(trace.split("\nproject_restored_checks=", 1)[1].splitlines()[0])
        assert (restored > 0) != no_incremental, trace
        record("DX14-saved", {"restored_checks": restored, "persistence": not no_incremental})
        close_source(client, path)
        path.write_text(BAD)
        open_source(client, path, GOOD)
        assert not diagnostics(client, path)
        cli = subprocess.run([str(binary), *flags, "check", path.name], cwd=root, capture_output=True)
        (output / "cli.stdout").write_bytes(cli.stdout)
        (output / "cli.stderr").write_bytes(cli.stderr)
        assert cli.returncode != 0 and b"SIFR-TYPE-0002" in cli.stdout + cli.stderr
        assert not diagnostics(client, path)
        # didSave without text preserves editor authority even if disk is stale.
        client.notify("textDocument/didSave", {"textDocument": {"uri": file_uri(path)}})
        assert not diagnostics(client, path)
        close_source(client, path)
        assert stats(client) is None
        open_source(client, path, path.read_text())
        assert diagnostics(client, path)
        record("E01", {"overlay_clean": True, "saved_cli_exit": cli.returncode, "close_reopen_sees_disk": True})
        # Burst requests followed by changes, with no cancellation notification.
        # Every request must have exactly one success-before-edit or ContentModified outcome.
        stale = 0
        for version in range(2, 22):
            rid = client.next_id
            client.send_request(rid, "textDocument/formatting", {"textDocument": {"uri": file_uri(path)}, "options": {}})
            client.notify("textDocument/didChange", {"textDocument": {"uri": file_uri(path), "version": version}, "contentChanges": [{"text": GOOD if version % 2 else BAD}]})
            response = client.wait_for_response(rid)
            if "error" in response:
                assert response["error"]["code"] == -32801, response
                stale += 1
            assert bool(diagnostics(client, path)) == (version % 2 == 0)
        assert stale > 0, "transport did not exercise a superseded request"
        assert not client.pending_requests
        record("E02", {"terminal_responses": 20, "stale_without_cancel": stale})
        close_source(client, path)
        # Alias roots preserve physical ownership and the client's logical URI.
        alias = output / "alias"
        alias.symlink_to(root, target_is_directory=True)
        alias_path = alias / path.name
        unicode = 'def main():\r\n    text: str = "🦀"\r\n    value: int = "bad"\r\n'
        open_source(client, alias_path, unicode)
        result = diagnostics(client, alias_path)
        assert result and result[0]["range"]["start"]["line"] == 2, result
        client.notify("textDocument/didChange", {"textDocument": {"uri": file_uri(alias_path), "version": 2}, "contentChanges": [{"range": {"start": {"line": 1, "character": 17}, "end": {"line": 1, "character": 19}}, "text": "ok"}]})
        result = diagnostics(client, alias_path)
        assert result and all(item["range"]["start"]["line"] == 2 for item in result), result
        close_source(client, alias_path)
        capital = root / "Main.sifr"
        capital.write_text(GOOD)
        open_source(client, path, BAD)
        open_source(client, capital, GOOD)
        assert diagnostics(client, path) and not diagnostics(client, capital)
        close_source(client, path)
        close_source(client, capital)
        record("E03", {"symlink_logical_uri": file_uri(alias_path), "utf16_surrogate_edit_crlf": True, "linux_case_distinct": True})
        for index in range(12):
            project = root / f"switch-{index}"
            project.mkdir()
            other = project / "main.sifr"
            other.write_text(GOOD)
            open_source(client, other, GOOD)
            assert not diagnostics(client, other)
            close_source(client, other)
            assert stats(client) is None
        record("E04", {"open_close_switch_cycles": 12, "unreferenced_metadata_released": True})
    finally:
        finish(client)

    for encoding, end in [("utf-8", 21), ("utf-32", 18)]:
        client = LspClient(cwd=root)
        try:
            initialized = client.request("initialize", {"processId": None, "rootUri": file_uri(root), "capabilities": {"general": {"positionEncodings": [encoding]}}})
            assert initialized["capabilities"]["positionEncoding"] == encoding
            client.notify("initialized", {})
            open_source(client, path, unicode)
            assert diagnostics(client, path)[0]["range"]["start"]["line"] == 2
            client.notify("textDocument/didChange", {"textDocument": {"uri": file_uri(path), "version": 2}, "contentChanges": [{"range": {"start": {"line": 1, "character": 17}, "end": {"line": 1, "character": end}}, "text": "ok"}]})
            assert all(item["range"]["start"]["line"] == 2 for item in diagnostics(client, path))
            record("E03-" + encoding, {"negotiated": encoding, "crlf_unicode_edit": True})
        finally:
            finish(client)

    # Damage an owned installed copy. Formatting and syntax still work with no
    # semantic owner; explicit reconfiguration recovers the same live process.
    installed = output / "installed"
    shutil.copytree(binary.parent.parent, installed)
    os.environ["SIFR_LSP_COMMAND"] = shlex.join([str(installed / "bin/sifr"), *flags, "lsp", "--stdio"])
    descriptor = installed / "lib/sifr/stdlib.metadata.json"
    metadata = installed / "lib/sifr/stdlib.sifrmeta"
    metadata_bytes, descriptor_bytes = metadata.read_bytes(), descriptor.read_bytes()
    metadata.unlink()
    client = LspClient(cwd=root)
    try:
        initialize(client, root, {"diagnosticsMode": "off"})
        open_source(client, path, GOOD)
        for method, extra in [("textDocument/formatting", {"options": {}}), ("textDocument/foldingRange", {}), ("textDocument/selectionRange", {"positions": [{"line": 1, "character": 5}]})]:
            client.request(method, {"textDocument": {"uri": file_uri(path)}, **extra})
            assert stats(client) is None
        record("E05", {"missing_metadata_syntax_requests": 3, "semantic_owner_created": False})
        failures = diagnostics(client, path)
        assert failures and "metadata" in str(failures).lower(), failures
        metadata.write_bytes(metadata_bytes)
        client.notify("workspace/didChangeConfiguration", {"settings": {"diagnosticsMode": "open-files"}})
        assert not diagnostics(client, path)
        value = json.loads(descriptor_bytes)
        value["compiler_identity"] = "0" * 64
        descriptor.write_text(json.dumps(value))
        client.notify("workspace/didChangeConfiguration", {"settings": {}})
        failures = diagnostics(client, path)
        assert failures and "reinstall" in str(failures).lower(), failures
        descriptor.write_bytes(descriptor_bytes)
        client.notify("workspace/didChangeConfiguration", {"settings": {}})
        assert not diagnostics(client, path)
        record("E06", {"same_process_missing_and_incompatible_recovery": True, "actionable_setup_diagnostics": True})
    finally:
        descriptor.write_bytes(descriptor_bytes)
        metadata.write_bytes(metadata_bytes)
        finish(client)
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--no-incremental", action="store_true")
    args = parser.parse_args()
    run(args.binary.resolve(), args.output.resolve(), args.no_incremental)
