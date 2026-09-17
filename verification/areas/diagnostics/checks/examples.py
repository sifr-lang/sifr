#!/usr/bin/env python3
"""Execute declared diagnostic-document examples and the corresponding explain help."""
import json
from pathlib import Path
import re
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / "verification/runner"))
from sifr_verify.fixture_execution import compiler_binary, run_process

# Each row declares check-fail(code), then check-pass. These self-contained
# examples are checked, not indiscriminately run (many other docs require host
# packages or illustrate runtime failures).
EXAMPLES = ("SIFR-ASYNC-0001", "SIFR-ASYNC-0002", "SIFR-CALL-0003",
            "SIFR-IMPORT-0001", "SIFR-PROTO-0006")

def main():
    binary = compiler_binary()
    failures = []
    for code in EXAMPLES:
        fragment = ROOT / f"crates/sifr_diagnostics/error_page_examples/{code}.md"
        snippets = re.findall(r"```(?:python|sifr)\n(.*?)```", fragment.read_text(), re.S)
        if len(snippets) != 2:
            raise RuntimeError(f"{code}: expected erroneous/fixed pair")
        page = (ROOT / f"docs/errors/{code}.mdx").read_text()
        if fragment.read_text().strip() not in page:
            raise RuntimeError(f"{code}: published example differs from tested source")
        with tempfile.TemporaryDirectory(prefix="sifr-doc-example-") as temporary:
            source = Path(temporary) / "main.sifr"
            for index, text in enumerate(snippets):
                source.write_text(text)
                outcome = run_process([str(binary), "--diagnostic-format", "json",
                                       "check", str(source)], cwd=source.parent)
                expected = 1 if index == 0 else 0
                if outcome.cause != "exit" or outcome.truncated or outcome.returncode != expected:
                    failures.append(f"{code}/{index}: expected exit {expected}: {outcome.stderr}")
                    continue
                if index == 0:
                    diagnostics = json.loads(outcome.stderr or outcome.stdout)
                    if code not in {diagnostic["code"] for diagnostic in diagnostics}:
                        failures.append(f"{code}: intended diagnostic absent: {diagnostics}")
        explained = run_process([str(binary), "--explain", code], cwd=ROOT)
        if explained.returncode or code not in explained.stdout + explained.stderr:
            failures.append(f"{code}: explain failed")
    help_result = run_process([str(binary), "--help"], cwd=ROOT)
    if help_result.returncode or "--explain" not in help_result.stdout:
        failures.append("explain is missing from CLI help")
    if failures:
        raise RuntimeError("\n".join(failures))
    print(f"error-document examples: {len(EXAMPLES)} check-fail/check-pass/explain pairs pass")

if __name__ == "__main__":
    main()
