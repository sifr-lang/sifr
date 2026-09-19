#!/usr/bin/env python3
"""Reject semantics-bearing implementation paths inside sifr_lsp."""

from __future__ import annotations

import argparse
import re
import sys
import tempfile
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
LSP_ROOT = REPO_ROOT / "crates" / "sifr_lsp"

FORBIDDEN_PATTERNS = {
    "sifr_python_parser": "LSP must not call the raw parser",
    "ruff_python_parser": "LSP must not call the raw parser",
    "parse_unchecked": "LSP must not call parser internals",
    "parse_module_with_diagnostics": "LSP must route parse diagnostics through analysis/frontend",
    "lower_module(": "LSP must not lower HIR directly",
    "lower_frontend_module": "LSP must not lower HIR directly",
    "type_check": "LSP must not type-check directly",
    "HirModule": "LSP must not traverse HIR for semantic answers",
    "sifr_type_system": "LSP must not construct type-system semantic answers",
    "FunctionType": "LSP must not construct callable signatures",
    "sifr_codegen::": "LSP must not call codegen directly",
}

ALLOWED_SNIPPETS = {
    "AnalysisHost::generated_rust_preview",
}


def rust_files() -> list[Path]:
    if not LSP_ROOT.exists():
        return []
    return sorted(path for path in LSP_ROOT.rglob("*.rs") if "target" not in path.parts)


def violations(paths: list[Path]) -> list[str]:
    failures: list[str] = []
    for path in paths:
        text = path.read_text(encoding="utf-8", errors="replace")
        for line_number, line in enumerate(text.splitlines(), 1):
            if any(snippet in line for snippet in ALLOWED_SNIPPETS):
                continue
            for pattern, reason in FORBIDDEN_PATTERNS.items():
                if pattern in line:
                    failures.append(
                        f"{path.relative_to(REPO_ROOT)}:{line_number} contains {pattern!r}: {reason}"
                    )
                    break
        if path.name == "analysis_workspace.rs":
            failures.extend(project_fallback_violations(path, text))
    return failures


def project_fallback_violations(path: Path, text: str) -> list[str]:
    failures: list[str] = []
    fallback_calls = text.count("LspDocumentAnalysis::open(")
    if fallback_calls != 2:
        failures.append(
            f"{path.relative_to(REPO_ROOT)} has {fallback_calls} standalone document open call(s); expected 2 no-project branches"
        )
    def method_text(name: str) -> str | None:
        start = re.search(r"(?m)^    (?:pub\(crate\) )?fn " + name + r"\(", text)
        if start is None:
            return None
        following = re.search(r"(?m)^    (?:pub\(crate\) )?fn ", text[start.end():])
        return text[start.start():start.end() + following.start()] if following else text[start.start():]

    refresh_text = method_text("refresh_projects")
    if refresh_text is None:
        failures.append(f"{path.relative_to(REPO_ROOT)} missing refresh_projects; cannot verify LSP ownership")
    else:
        # Recognize the reviewed delegation, including the helper's actual body.
        # Unknown delegation must still fail the required removal assertion.
        if "self.synchronize_projects(documents, true);" in refresh_text:
            helper = method_text("synchronize_projects")
            if helper is None:
                failures.append(f"{path.relative_to(REPO_ROOT)} missing synchronize_projects ownership helper")
            else:
                refresh_text += helper
        if "LspDocumentAnalysis::open(" in refresh_text:
            failures.append(f"{path.relative_to(REPO_ROOT)} refresh_projects creates standalone project fallback")
        if "self.documents.remove(document.uri())" not in refresh_text:
            failures.append(f"{path.relative_to(REPO_ROOT)} refresh_projects must drop standalone project entries")
    for method in ["open_document", "update_document"]:
        method_start = text.find(f"pub(crate) fn {method}(")
        if method_start == -1:
            failures.append(f"{path.relative_to(REPO_ROOT)} missing {method}; cannot verify LSP ownership")
            continue
        next_method = text.find("pub(crate) fn", method_start + 1)
        method_text = text[method_start:] if next_method == -1 else text[method_start:next_method]
        # This guard intentionally recognizes only the two reviewed project-routing
        # shapes. If routing is refactored, update this check with the new shape
        # instead of letting project-rooted standalone analysis slip through.
        project_match = method_text.find("match self.projects.get_mut(&root)")
        project_if_let = method_text.find("if let Some(project) = self.projects.get_mut(&root)")
        if project_match != -1:
            project_arm_start = project_match
            project_arm_end = method_text.find("None => true", project_match)
            boundary_label = "project match arm"
        elif project_if_let != -1:
            project_arm_start = project_if_let
            project_arm_end = method_text.find("} else {", project_if_let)
            boundary_label = "no-project boundary"
        else:
            failures.append(f"{path.relative_to(REPO_ROOT)} {method} has unverifiable project ownership routing")
            continue
        if project_arm_end == -1:
            failures.append(f"{path.relative_to(REPO_ROOT)} {method} has unverifiable {boundary_label}")
            continue
        project_arm = method_text[project_arm_start:project_arm_end]
        if "LspDocumentAnalysis::open(" in project_arm:
            failures.append(
                f"{path.relative_to(REPO_ROOT)} {method} creates standalone analysis from a project-owned path"
            )
    return failures


def run_self_test() -> None:
    with tempfile.TemporaryDirectory(dir=REPO_ROOT / "target") as tmp:
        seed = Path(tmp) / "handler.rs"
        seed.write_text("use sifr_lowering::HirModule;\nfn handler() { lower_module(&[]); }\n", encoding="utf-8")
        found = violations([seed])
    if not found:
        raise SystemExit("LSP split-brain self-test failed: seeded direct HIR path passed")
    with tempfile.TemporaryDirectory(dir=REPO_ROOT / "target") as tmp:
        seed = Path(tmp) / "analysis_workspace.rs"
        seed.write_text(
            """
impl LspAnalysisWorkspace {
    pub(crate) fn open_document(&mut self, document: &DocumentState) -> bool {
        match self.projects.get_mut(&root) {
            Some(project) => {
                if project.open_document(document).is_ok() {
                    false
                } else {
                    let analysis = LspDocumentAnalysis::open(document);
                    self.documents.insert(document.uri().to_string(), analysis);
                    false
                }
            }
            None => true,
        }
    } else {
        let analysis = LspDocumentAnalysis::open(document);
        self.documents.insert(document.uri().to_string(), analysis);
    }

    pub(crate) fn update_document(&mut self, document: &DocumentState) -> bool {
        match self.projects.get_mut(&root) {
            Some(project) => {
                if project.update_document(document).is_ok() {
                    false
                } else {
                    let analysis = LspDocumentAnalysis::open(document);
                    self.documents.insert(uri, analysis);
                    false
                }
            }
            None => true,
        }
    } else {
        let analysis = LspDocumentAnalysis::open(document);
        self.documents.insert(uri, analysis);
    }

    pub(crate) fn refresh_projects(&mut self, documents: &DocumentStore) {
        let fallback = LspDocumentAnalysis::open(document);
        self.documents.insert(document.uri().to_string(), fallback);
    }
}
""",
            encoding="utf-8",
        )
        found = violations([seed])
    if not any("standalone analysis from a project-owned path" in item for item in found):
        raise SystemExit("LSP split-brain self-test failed: seeded match project fallback passed")
    if not any("refresh_projects creates standalone project fallback" in item for item in found):
        raise SystemExit("LSP split-brain self-test failed: seeded refresh fallback passed")
    with tempfile.TemporaryDirectory(dir=REPO_ROOT / "target") as tmp:
        seed = Path(tmp) / "analysis_workspace.rs"
        seed.write_text(
            """
impl LspAnalysisWorkspace {
    pub(crate) fn open_document(&mut self, document: &DocumentState) -> bool {
        if let Some(project) = self.projects.get_mut(&root) {
            let analysis = LspDocumentAnalysis::open(document);
            self.documents.insert(document.uri().to_string(), analysis);
            return false;
        } else {
            let analysis = LspDocumentAnalysis::open(document);
            self.documents.insert(document.uri().to_string(), analysis);
        }
    }

    pub(crate) fn update_document(&mut self, document: &DocumentState) -> bool {
        if let Some(project) = self.projects.get_mut(&root) {
            let analysis = LspDocumentAnalysis::open(document);
            self.documents.insert(uri, analysis);
            return false;
        } else {
            let analysis = LspDocumentAnalysis::open(document);
            self.documents.insert(uri, analysis);
        }
    }

    pub(crate) fn refresh_projects(&mut self, documents: &DocumentStore) {
        let analysis = LspProjectAnalysis::open(root.clone(), &documents);
        self.projects.insert(root, analysis);
    }
}
""",
            encoding="utf-8",
        )
        found = violations([seed])
    if not any("standalone analysis from a project-owned path" in item for item in found):
        raise SystemExit("LSP split-brain self-test failed: seeded if-let project fallback passed")
    with tempfile.TemporaryDirectory(dir=REPO_ROOT / "target") as tmp:
        seed = Path(tmp) / "analysis_workspace.rs"
        seed.write_text(
            """
impl LspAnalysisWorkspace {
    pub(crate) fn open_document(&mut self, document: &DocumentState) -> bool {
        if let Some(project) = self.projects.get_mut(&root) {
            return false;
        } else {
            let analysis = LspDocumentAnalysis::open(document);
            self.documents.insert(document.uri().to_string(), analysis);
        }
    }

    pub(crate) fn update_document(&mut self, document: &DocumentState) -> bool {
        if let Some(project) = self.projects.get_mut(&root) {
            return false;
        } else {
            let analysis = LspDocumentAnalysis::open(document);
            self.documents.insert(uri, analysis);
        }
    }

    pub(crate) fn refresh_projects(&mut self, documents: &DocumentStore) {
        let analysis = LspProjectAnalysis::open(root.clone(), &documents);
        self.projects.insert(root, analysis);
    }
}
""",
            encoding="utf-8",
        )
        found = violations([seed])
    if not any("refresh_projects must drop standalone project entries" in item for item in found):
        raise SystemExit("LSP split-brain self-test failed: seeded missing refresh remove passed")
    current = (LSP_ROOT / "src" / "analysis_workspace.rs").read_text(encoding="utf-8")
    current_path = LSP_ROOT / "src" / "analysis_workspace.rs"
    if project_fallback_violations(current_path, current):
        raise SystemExit("LSP split-brain self-test failed: context-injected routing rejected")
    seeded = current.replace(
        "if let Some(project) = self.projects.get_mut(&root) {",
        "if let Some(project) = self.projects.get_mut(&root) {"
        "\nlet fallback = LspDocumentAnalysis::open(&self.compiler, document);",
        1,
    )
    found = project_fallback_violations(current_path, seeded)
    if not any("standalone analysis from a project-owned path" in item for item in found):
        raise SystemExit("LSP split-brain self-test failed: context-injected project fallback passed")
    for label, seeded, required in [
        ("helper removal", current.replace("self.documents.remove(document.uri());", ""),
         "must drop standalone project entries"),
        ("missing helper", current.replace("fn synchronize_projects(", "fn renamed_helper("),
         "missing synchronize_projects ownership helper"),
        ("unknown delegation", current.replace("self.synchronize_projects(documents, true);",
                                               "self.other_helper(documents);"),
         "must drop standalone project entries"),
        ("helper fallback", current.replace("        let mut grouped:",
             "        let fallback = LspDocumentAnalysis::open(document);\n        let mut grouped:"),
         "refresh_projects creates standalone project fallback"),
    ]:
        if not any(required in item for item in project_fallback_violations(current_path, seeded)):
            raise SystemExit(f"LSP split-brain self-test failed: seeded {label} passed")
    print("LSP split-brain self-test: PASS")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        run_self_test()
        return 0

    found = violations(rust_files())
    if found:
        print("LSP split-brain: FAIL", file=sys.stderr)
        for failure in found:
            print(f"  - {failure}", file=sys.stderr)
        return 1
    print("LSP split-brain: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
