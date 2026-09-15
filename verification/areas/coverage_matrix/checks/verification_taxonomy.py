#!/usr/bin/env python3
"""Reject delivery-plan taxonomy in compiler, verification, and codebase surfaces."""

from __future__ import annotations

import argparse
import os
import re
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[4]
RULES_ALIAS = "con" + "tract"
DELIVERY_STAGE = "pha" + "se"
DELIVERY_STEP = "mile" + "stone"
DELIVERY_BATCH = "wa" + "ve"
DELIVERY_MAP = "road" + "map"
DELIVERY_DONE = "close" + "out"
DELIVERY_QUEUE = "back" + "log"
DELIVERY_TICKET = "tick" + "et"
DELIVERY_EPIC = "ep" + "ic"
DELIVERY_WORK_ITEM = "work-" + "item"
DELIVERY_TERMS = (DELIVERY_STAGE, DELIVERY_STEP, DELIVERY_BATCH)

ACTIVE_ROOTS = (
    REPO_ROOT / "verification",
    REPO_ROOT / ".github" / "workflows",
    REPO_ROOT / "crates",
    REPO_ROOT / "demos",
    REPO_ROOT / "docs",
    REPO_ROOT / "editor_integrations",
    REPO_ROOT / "internal_docs",
    REPO_ROOT / "stdlib",
    REPO_ROOT / "README.md",
    REPO_ROOT / "Cargo.toml",
    REPO_ROOT / "sifr.toml",
    REPO_ROOT / "scripts",
)

TEXT_EXTENSIONS = {
    ".json",
    ".md",
    ".mdx",
    ".sh",
    ".py",
    ".rs",
    ".sifr",
    ".toml",
    ".yml",
    ".yaml",
}

SKIP_DIR_NAMES = {".git", ".venv", "__pycache__", "node_modules", "target", "third_party", "vendor", "skills"}

NUMBERED_DELIVERY_NAME = re.compile(
    r"(?:^|[/_.-])(?:item|part|phase|milestone|wave|sprint|workstream)[_-]?\d+[a-z]?(?=[/_.-]|$)",
    re.IGNORECASE,
)
# Percentiles are measurement names, not abbreviated delivery labels.
ABBREVIATED_DELIVERY_LABEL = (
    r"(?:m|ms|p(?!(?:50|90|95|99)(?:[_\W]|$))|ph|pt|w)\d+[a-z]?(?:[._-]\d+)*"
)
ABBREVIATED_DELIVERY_NAME = re.compile(
    r"(?:^|[/_.-])" + ABBREVIATED_DELIVERY_LABEL + r"(?=[/_.-]|$)", re.IGNORECASE
)
AMBIGUOUS_DEMO_VARIABLE = re.compile(
    r"^\s*(?:let\s+(?:mut\s+)?)?(?P<name>_?" + ABBREVIATED_DELIVERY_LABEL
    + r")\s*(?::|=(?!=))", re.IGNORECASE
)
ABBREVIATED_DELIVERY_TEXT = re.compile(
    r"(?<![A-Za-z0-9])" + ABBREVIATED_DELIVERY_LABEL + r"(?=[/_-][A-Za-z_])"
    r"|[A-Za-z][A-Za-z0-9_./-]*[/_.-]" + ABBREVIATED_DELIVERY_LABEL + r"(?=[_\W]|$)"
    r"|['\"]/?" + ABBREVIATED_DELIVERY_LABEL + r"(?:\s+[A-Za-z][^'\"]*)?['\"]"
    r"|\b(?:TODO\(|Reference:\s*)" + ABBREVIATED_DELIVERY_LABEL + r"\b"
    r"|(?:^\s*#+|//)\s*" + ABBREVIATED_DELIVERY_LABEL + r"\b",
    re.IGNORECASE,
)
NUMBERED_DELIVERY_TEXT = re.compile(
    r"(?<![A-Za-z0-9_])(?:item|phase|milestone|wave|sprint)[_-]?\d+[a-z]?(?=[_\W]|$)"
    r"|\b(?:items?|parts?|phases?|milestones?|waves?|sprints?)\s+\d+\b"
    r"|(?:^|[/_.-])(?:part|item)[_-]?\d+[a-z]?(?=/|\.(?:rs|sifr|mdx?|json)\b|$)",
    re.IGNORECASE,
)
DELIVERY_METADATA = re.compile(
    r"\b(?:implementation_items|owner_items?|closure_item)\b"
    r'|["\'](?:phase|milestone|wave)["\']\s*:\s*["\']?\d'
    r"|^\s*# Reference: (?!https?://|(?:\.\.?/)?(?:docs|internal_docs)/)",
    re.IGNORECASE,
)

FILENAME_PATTERNS = (
    NUMBERED_DELIVERY_NAME,
    ABBREVIATED_DELIVERY_NAME,
    re.compile(r"(^|[-_])(?:" + "|".join(DELIVERY_TERMS) + r")([-_]|$)", re.IGNORECASE),
    re.compile(r"(^|[-_])work[-_]item([-_]|$)", re.IGNORECASE),
)

ALLOW_TEXT_PATTERNS = (
    re.compile(r"\b(?:WorkspaceTracePhase|SingleOwnerCompilerPhase|LintPhase|PhaseExecution|ProgressPhase)\b"),
    re.compile(r"\b(?:phase_plan|empty_phase_plan|phase_has_enabled_rules|mark_phase_readonly)\b"),
    re.compile(r"\b(?:record_compiler_phase_trace|build phase|compiler phase|trace phases|phase-aware|phase=)\b", re.IGNORECASE),
    re.compile(r"\b" + "exp" + r"_m1\b"),
    re.compile(r"\b(?:contract_config|contract_field|contract_types|contract_for_identity|contract_error)\b"),
    re.compile(r'\bid\("m\d+"\)'),
    re.compile(r"\bsifr\.sql\.migration\.state\.m\d+\b"),
    re.compile(
        r"\b(?:"
        + RULES_ALIAS
        + r"_suites?|"
        + RULES_ALIAS
        + r"_errors|format_expectation_"
        + RULES_ALIAS
        + r"_errors|capability_queue|"
        + RULES_ALIAS
        + r"_id|"
        + RULES_ALIAS
        + r"_version|"
        + RULES_ALIAS
        + r"_check|"
        + RULES_ALIAS
        + r"_path|bad_alias|bad_prefixed_alias_label)\b",
        re.IGNORECASE,
    ),
)

TEXT_TRIGGER_TERMS = (
    DELIVERY_STAGE,
    DELIVERY_STEP,
    DELIVERY_BATCH,
    DELIVERY_MAP,
    DELIVERY_DONE,
    DELIVERY_QUEUE,
    "ad hoc",
    "ad-hoc",
    "backlog",
    "capabilities",
    "capability",
    "closed from a local validation standpoint",
    "closure",
    RULES_ALIAS,
    "conversion set",
    "follow-up",
    "future phases",
    "gate-closure",
    "implementation slice",
    "later phases",
    "reference:",
    "slice",
    "source issue:",
    "surface",
    "successor-" + DELIVERY_STAGE,
    "task ownership",
    "todo(",
    "work item",
    "work-item",
    "work-on-item",
)
M_TOKEN_TRIGGER = re.compile(r"(?:^|[^A-Za-z0-9])m\d|[/_-]m\d|\bm_[a-z]", re.IGNORECASE)

LEGACY_FIELD_PATTERNS = (
    "implementation_" + DELIVERY_STEP,
    "updated_by_" + DELIVERY_STEP,
    "future-" + DELIVERY_STAGE,
    "closes_in_" + DELIVERY_BATCH,
    "closes_in_sub" + DELIVERY_BATCH,
    "expires_in_" + DELIVERY_BATCH,
)

TEXT_PATTERNS = (
    NUMBERED_DELIVERY_TEXT,
    ABBREVIATED_DELIVERY_TEXT,
    DELIVERY_METADATA,
    re.compile(
        r"(?:^|[^A-Za-z0-9])(?:"
        + DELIVERY_BATCH
        + r"s?|"
        + DELIVERY_STEP
        + r"s?)(?:[^A-Za-z0-9]|$)",
        re.IGNORECASE,
    ),
    re.compile(r"(?:^|[^A-Za-z0-9])(?:" + DELIVERY_MAP + r"s?)(?:[^A-Za-z0-9]|$)", re.IGNORECASE),
    re.compile(r"\b" + DELIVERY_STAGE + r"\s+\d+\b", re.IGNORECASE),
    re.compile(r"\b" + DELIVERY_STEP + r"\s+\d+\b", re.IGNORECASE),
    re.compile(r"\b" + DELIVERY_BATCH + r"\s+\d+\b", re.IGNORECASE),
    re.compile(
        r"(?:^|[^A-Za-z0-9])(?:" + "|".join(DELIVERY_TERMS) + r")\d+(?:[^A-Za-z0-9]|$)",
        re.IGNORECASE,
    ),
    re.compile(r"\b(?:" + "|".join(DELIVERY_TERMS) + r")[_-][a-z0-9][a-z0-9_-]*\b", re.IGNORECASE),
    re.compile(r"\b" + RULES_ALIAS + r"\s+slice\s+\d+\b", re.IGNORECASE),
    re.compile(r"\b" + RULES_ALIAS + r"\d+(?:[._-]\d+|[_-][a-z0-9][a-z0-9_-]*|\b)", re.IGNORECASE),
    re.compile(r"\b" + RULES_ALIAS.capitalize() + r"\s+\d+\b"),
    re.compile(r"\b" + RULES_ALIAS + r"_[a-z][a-z0-9]*_\d+[a-z0-9_]*\b", re.IGNORECASE),
    re.compile(r"\bcapability\s+pass\s+\d+(?:\.\d+)?\b", re.IGNORECASE),
    re.compile(r"\bcapability_[a-z][a-z0-9_]*_\d+[a-z0-9_]*\b", re.IGNORECASE),
    re.compile(r"`[a-z_][a-z0-9_/-]* [^`]*\.md`", re.IGNORECASE),
    re.compile(
        r"\b[a-z0-9_]*" + RULES_ALIAS + r"_(?:psp|" + DELIVERY_MAP + r"|ext)[a-z0-9_]*\d+[a-z0-9_]*\b",
        re.IGNORECASE,
    ),
    re.compile(r"\b" + RULES_ALIAS + r"_[a-z][a-z0-9_]*\b", re.IGNORECASE),
    re.compile(r"\barchived\s+[a-z0-9_/-]*\s*" + RULES_ALIAS + r"\s+record\b", re.IGNORECASE),
    re.compile(r"\btask ownership surface\d*\b", re.IGNORECASE),
    re.compile(r"\bm_[a-z][a-z0-9_]*_\d+[a-z0-9_]*\b", re.IGNORECASE),
    re.compile(
        r"\b(?:native task|process/runtime|task-context and shutdown|blocking/offload|synchronization|"
        r"typed\s+IPC|cache-key identity|production runtime audit|flow-graph|trace/status|"
        r"editor corpus and snapshot handle|bucketed index|project residency|foundation|"
        r"LSP (?:scheduler|latency budget|cancellation[,/ ]+progress[,/ ]+and watchdog))\s+surface"
        r"(?:\d+(?:\.\d+)?)?\b",
        re.IGNORECASE,
    ),
    re.compile(r"\bPhase\s+ad-hoc\b", re.IGNORECASE),
    re.compile(
        r"\bSource issue:\s+(?:revised compiler "
        + DELIVERY_MAP
        + r" record|.*-"
        + DELIVERY_EPIC
        + r"\.md|.*-"
        + DELIVERY_TICKET
        + r"\.md)\b",
        re.IGNORECASE,
    ),
    re.compile(r"\bSource issue:\s+ad-hoc-[a-z0-9-]+(?:\.md|-execution\.md)\b", re.IGNORECASE),
    re.compile(r"(?:^|[^A-Za-z0-9])ad-hoc-[a-z0-9-]+(?:[^A-Za-z0-9]|$)", re.IGNORECASE),
    re.compile(r"\b(?:delivery|implementation|PR)?\s*slice\s+\d+\b", re.IGNORECASE),
    re.compile(r"\b(?:implementation|compiler)\s+slice\b", re.IGNORECASE),
    re.compile(r"\bconversion set\s+\d+\s+demo\b", re.IGNORECASE),
    re.compile(r"\bConversion Set\s+\d+\b"),
    re.compile(r"(?:^|[^A-Za-z0-9])" + DELIVERY_DONE + r"(?:[^A-Za-z0-9]|$)", re.IGNORECASE),
    re.compile(r"\bfeatures delivered in this follow-up work\b", re.IGNORECASE),
    re.compile(r"\bfollow-up\b", re.IGNORECASE),
    re.compile(r"\bsuccessor-" + DELIVERY_STAGE + r"\b", re.IGNORECASE),
    re.compile(r"\bcontinuation " + DELIVERY_STAGE + r"\b", re.IGNORECASE),
    re.compile(r"\bthis " + DELIVERY_STAGE + r"\b", re.IGNORECASE),
    re.compile(r"\bPhase:\s*`?[^`\n]+"),
    re.compile(r"\b(?:later|future) phases\b", re.IGNORECASE),
    re.compile(r"\bcapabilities?\s+\d+\s+(?:through|to)\s+\d+\b", re.IGNORECASE),
    re.compile(
        r"\b(?!and\b|or\b|the\b)[a-z][a-z0-9/-]*\s+capability\s+"
        r"(?:evidence|fixtures|implementation|public|subprocess|readiness|supported|owned|text/process|legacy)\b",
        re.IGNORECASE,
    ),
    re.compile(
        r"^#.*\b[a-z][a-z0-9/-]*\s+capability\s+"
        r"(?:Process|Offload|Blocking|Synchronization|Shutdown)\b",
        re.IGNORECASE,
    ),
    re.compile(r"\bgate-closure\b", re.IGNORECASE),
    re.compile(
        r"\b(?:lazy-parity|dependency-metadata|text_surface_governance|downstream_alignment|"
        r"iterable_stdlib)-closure\b",
        re.IGNORECASE,
    ),
    re.compile(r"\b(?:web-framework|server-framework|HTTP-client|HTTP client|baseline)\s+" + RULES_ALIAS + r"\b", re.IGNORECASE),
    re.compile(
        r"\b(?:owner_"
        + RULES_ALIAS
        + r"s?|"
        + RULES_ALIAS
        + r"_backlog|blocked_until_"
        + RULES_ALIAS
        + r"s|closed_"
        + RULES_ALIAS
        + r"s_env|deferred-to-future-"
        + RULES_ALIAS
        + r"|blocked-on-"
        + RULES_ALIAS
        + r"|deferred-to-adapter-"
        + RULES_ALIAS
        + r")\b",
        re.IGNORECASE,
    ),
    re.compile(r"\bworld-" + r"class verification " + DELIVERY_STAGE + r"\b", re.IGNORECASE),
    re.compile(r"\bclosed from a local validation standpoint\b", re.IGNORECASE),
    re.compile(r"\bImplementation " + DELIVERY_QUEUE + r"\b", re.IGNORECASE),
    re.compile(r"\b" + DELIVERY_QUEUE + r" item\b", re.IGNORECASE),
    re.compile(r"\b(?:Capability|Concrete|parity)\s+" + DELIVERY_QUEUE + r"\b", re.IGNORECASE),
    re.compile(
        r"\b" + DELIVERY_QUEUE + r"(?:-generating|-oriented| entries| generation| prioritization)\b",
        re.IGNORECASE,
    ),
    re.compile(r"\bwork items?\b", re.IGNORECASE),
    re.compile(r"`/(?:add-" + DELIVERY_WORK_ITEM + r"|work-on-item)`"),
    re.compile(r"\bTODO\(m\d+\)", re.IGNORECASE),
    re.compile(r"\bReference:\s*m\d+\b", re.IGNORECASE),
    re.compile(r"['\"]m\d+(?:\s+[a-z][^'\"]*)?['\"]", re.IGNORECASE),
    re.compile(r"['\"]/m\d+(?:[/-][a-z0-9][a-z0-9_/-]*)?['\"]", re.IGNORECASE),
    re.compile(r"\b[a-z][a-z0-9_]*_m\d+[a-z0-9_]*\b", re.IGNORECASE),
    re.compile(r"\bm\d+[_-][a-z0-9][a-z0-9_-]*\b", re.IGNORECASE),
    re.compile(r"\b[a-z][a-z0-9_-]*[_-]m\d+\b", re.IGNORECASE),
    re.compile(r"\bm\d+-[a-z0-9][a-z0-9_-]*\b", re.IGNORECASE),
    re.compile(r"\bM\d+(?:\.\d+)?\b"),
    re.compile(r"\b(?:" + "|".join(re.escape(pattern) for pattern in LEGACY_FIELD_PATTERNS) + r")\b"),
)


@dataclass(frozen=True)
class Failure:
    path: Path
    line: int | None
    text: str

    def render(self) -> str:
        location = repo_path(self.path)
        if self.line is not None:
            location = f"{location}:{self.line}"
        return f"{location}: {self.text}"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        return run_self_test()

    self_test_status = run_self_test(quiet=True)
    if self_test_status != 0:
        return self_test_status
    failures = collect_failures(ACTIVE_ROOTS)
    if failures:
        for failure in failures:
            print(f"verification-taxonomy error: {failure.render()}", file=sys.stderr)
        return 1
    print("verification taxonomy ok: active verification and crate surfaces use compiler/codebase terminology")
    return 0


def collect_failures(
    roots: tuple[Path, ...], *, audit_root: Path = REPO_ROOT
) -> list[Failure]:
    """Audit selected paths within one owning tree, excluding caller ancestors."""
    failures: list[Failure] = []
    seen: set[Path] = set()
    for root in roots:
        root.relative_to(audit_root)
        if not root.exists():
            continue
        if root.is_file():
            if root not in seen and not should_skip(root, audit_root=audit_root):
                seen.add(root)
                failures.extend(validate_filename(root, audit_root=audit_root))
                failures.extend(validate_text(root, audit_root=audit_root))
            continue
        for path in walk_text_candidates(root, audit_root=audit_root):
            if path in seen:
                continue
            seen.add(path)
            failures.extend(validate_filename(path, audit_root=audit_root))
            failures.extend(validate_text(path, audit_root=audit_root))
    return failures


def walk_text_candidates(root: Path, *, audit_root: Path = REPO_ROOT) -> list[Path]:
    candidates: list[Path] = []
    if should_skip(root, audit_root=audit_root):
        return candidates
    for current_root, dirnames, filenames in os.walk(root):
        dirnames[:] = [dirname for dirname in dirnames if dirname not in SKIP_DIR_NAMES]
        current = Path(current_root)
        for filename in filenames:
            path = current / filename
            if not should_skip(path, audit_root=audit_root):
                candidates.append(path)
    return candidates


def should_skip(path: Path, *, audit_root: Path = REPO_ROOT) -> bool:
    parts = set(path.relative_to(audit_root).parts)
    if parts & SKIP_DIR_NAMES:
        return True
    return path.is_file() and path.suffix not in TEXT_EXTENSIONS


def validate_filename(path: Path, *, audit_root: Path = REPO_ROOT) -> list[Failure]:
    name = path.relative_to(audit_root).as_posix()
    return [
        Failure(path, None, f"filename contains delivery-plan taxonomy: {name}")
        for pattern in FILENAME_PATTERNS
        if any(pattern.search(component) for component in Path(name).parts)
    ]


def validate_text(path: Path, *, audit_root: Path = REPO_ROOT) -> list[Failure]:
    relative_path = path.relative_to(audit_root)
    if path == Path(__file__).resolve():
        return []
    try:
        text = path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        return []
    failures: list[Failure] = []
    if relative_path.is_relative_to("demos") and path.suffix in {".sifr", ".rs"}:
        failures.extend(validate_demo_variables(path, text))
    for line_number, line in enumerate(text.splitlines(), start=1):
        if not has_taxonomy_trigger(line):
            continue
        checked_line = line
        for pattern in ALLOW_TEXT_PATTERNS:
            checked_line = pattern.sub("", checked_line)
        for pattern in TEXT_PATTERNS:
            if pattern.search(checked_line):
                failures.append(
                    Failure(path, line_number, f"line contains delivery-plan taxonomy: {line.strip()[:160]}")
                )
                break
    return failures


def validate_demo_variables(path: Path, text: str) -> list[Failure]:
    return [
        Failure(path, line_number, f"demo variable needs a descriptive name: {match['name']}")
        for line_number, line in enumerate(text.splitlines(), start=1)
        if (match := AMBIGUOUS_DEMO_VARIABLE.search(line)) is not None
    ]


def has_taxonomy_trigger(line: str) -> bool:
    lower = line.lower()
    return (
        any(term in lower for term in TEXT_TRIGGER_TERMS)
        or M_TOKEN_TRIGGER.search(line) is not None
        or ABBREVIATED_DELIVERY_TEXT.search(line) is not None
        or NUMBERED_DELIVERY_TEXT.search(line) is not None
        or DELIVERY_METADATA.search(line) is not None
    )


def run_self_test(*, quiet: bool = False) -> int:
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        good = root / "compiler_interface.rs"
        good.write_text(
            "enum WorkspaceTracePhase { Parse }\nfn record_compiler_phase_trace() {}\n",
            encoding="utf-8",
        )
        bad_text = root / "active_manifest.json"
        bad_label = DELIVERY_STAGE.capitalize() + " 99 readiness"
        bad_file = DELIVERY_STEP + "_99_tests.rs"
        bad_text.write_text(f'{{"label": "{bad_label}"}}\n', encoding="utf-8")
        bad_name = root / bad_file
        bad_name.write_text("// compiler test\n", encoding="utf-8")
        bad_mixed = root / "mixed_allowed_and_bad.rs"
        bad_mixed_label = DELIVERY_STEP.capitalize() + " 99"
        bad_mixed.write_text(f"// compiler phase trace; {bad_mixed_label}\n", encoding="utf-8")
        bad_lowercase = root / "lowercase_taxonomy.md"
        bad_lowercase_label = DELIVERY_BATCH + " 99"
        bad_lowercase.write_text(f"{bad_lowercase_label} should be rejected.\n", encoding="utf-8")
        bad_plain_milestone = root / "plain_milestone_taxonomy.md"
        bad_plain_milestone_label = "mile" + "stone"
        bad_plain_milestone.write_text(
            f"plain {bad_plain_milestone_label} terminology should be rejected.\n", encoding="utf-8"
        )
        bad_plain_wave = root / "plain_wave_taxonomy.md"
        bad_plain_wave_label = "wa" + "ve"
        bad_plain_wave.write_text(
            f"plain {bad_plain_wave_label} terminology should be rejected.\n", encoding="utf-8"
        )
        bad_plain_roadmap = root / "plain_roadmap_taxonomy.md"
        bad_plain_roadmap_label = "road" + "map"
        bad_plain_roadmap.write_text(
            f"plain {bad_plain_roadmap_label} terminology should be rejected.\n", encoding="utf-8"
        )
        bad_prefix = root / "prefix_m_taxonomy.json"
        bad_prefix_label = "m" + "5_closure_evidence"
        bad_prefix.write_text(f'{{"{bad_prefix_label}": true}}\n', encoding="utf-8")
        bad_suffix = root / "suffix_m_taxonomy.json"
        bad_suffix_label = "blocked-on-runtime-" + "m1"
        bad_suffix.write_text(f'{{"state": "{bad_suffix_label}"}}\n', encoding="utf-8")
        bad_bare = root / "bare_m_taxonomy.md"
        bad_bare_label = "M" + "1"
        bad_bare.write_text(f"{bad_bare_label} owns a delivery slice.\n", encoding="utf-8")
        bad_alias = root / (RULES_ALIAS + "_taxonomy.md")
        bad_alias_label = RULES_ALIAS + "42_1"
        bad_alias.write_text(f"{bad_alias_label} should not rename delivery work.\n", encoding="utf-8")
        bad_capability_pass = root / "capability_pass_taxonomy.md"
        bad_capability_pass_label = "capability " + "pass 3"
        bad_capability_pass.write_text(f"{bad_capability_pass_label} should not rename delivery work.\n", encoding="utf-8")
        bad_capability_id = root / "capability_id_taxonomy.md"
        bad_capability_id_label = "capability_" + "runtime_3"
        bad_capability_id.write_text(f"{bad_capability_id_label} should not rename delivery work.\n", encoding="utf-8")
        bad_spaced_filename = root / "spaced_filename_taxonomy.md"
        bad_spaced_filename_label = "`" + "concurrency_runtime_structured-task " + "capability_traceability.md`"
        bad_spaced_filename.write_text(f"{bad_spaced_filename_label}\n", encoding="utf-8")
        bad_digitless_alias = root / ("digitless_" + RULES_ALIAS + "_taxonomy.md")
        bad_digitless_alias_label = RULES_ALIAS + "_" + "safe_indexing"
        bad_digitless_alias.write_text(
            f"{bad_digitless_alias_label} should not rename delivery work.\n", encoding="utf-8"
        )
        bad_prefixed_alias = root / ("prefixed_" + RULES_ALIAS + "_taxonomy.md")
        bad_prefixed_alias_label = "sifr_" + RULES_ALIAS + "_" + "ext_" + "3_demo"
        bad_prefixed_alias.write_text(
            f"{bad_prefixed_alias_label} should not rename delivery work.\n", encoding="utf-8"
        )
        bad_task_surface = root / "task_surface_taxonomy.md"
        bad_task_surface_label = "task ownership " + "surface2"
        bad_task_surface.write_text(f"{bad_task_surface_label} should not rename delivery work.\n", encoding="utf-8")
        bad_m_word = root / "m_word_taxonomy.md"
        bad_m_word_label = "m_" + "driver_4"
        bad_m_word.write_text(f"{bad_m_word_label} should not rename delivery work.\n", encoding="utf-8")
        bad_surface_alias = root / "surface_alias_taxonomy.md"
        bad_surface_alias_label = "native task " + "surface"
        bad_surface_alias.write_text(
            f"{bad_surface_alias_label} should not rename delivery work.\n", encoding="utf-8"
        )
        bad_alias_record = root / (RULES_ALIAS + "_record_taxonomy.md")
        bad_alias_record_label = "archived compiler/codebase " + RULES_ALIAS + " record"
        bad_alias_record.write_text(
            f"{bad_alias_record_label} should not rename delivery work.\n", encoding="utf-8"
        )
        bad_ad_hoc_phase = root / "phase_ad_hoc_taxonomy.md"
        bad_ad_hoc_phase_label = DELIVERY_STAGE.capitalize() + " " + "ad-hoc"
        bad_ad_hoc_phase.write_text(
            f"{bad_ad_hoc_phase_label} should not rename delivery work.\n", encoding="utf-8"
        )
        bad_source_issue = root / "source_issue_taxonomy.sifr"
        bad_source_issue_label = "Source issue: revised compiler " + DELIVERY_MAP + " record"
        bad_source_issue.write_text(f"# {bad_source_issue_label}\n", encoding="utf-8")
        bad_ad_hoc_issue = root / "ad_hoc_issue_taxonomy.md"
        bad_ad_hoc_issue_label = "ad-" + "hoc-compiler-cleanup-execution.md"
        bad_ad_hoc_issue.write_text(f"{bad_ad_hoc_issue_label}\n", encoding="utf-8")
        bad_numbered_slice = root / "numbered_slice_taxonomy.md"
        bad_numbered_slice_label = "slice " + "7"
        bad_numbered_slice.write_text(f"implementation {bad_numbered_slice_label} demo\n", encoding="utf-8")
        bad_named_slice = root / "named_slice_taxonomy.md"
        bad_named_slice_label = "implementation " + "slice"
        bad_named_slice.write_text(f"{bad_named_slice_label} should be rejected.\n", encoding="utf-8")
        bad_followup = root / "followup_taxonomy.md"
        bad_followup_label = "follow-up " + "work"
        bad_followup.write_text(f"future behavior is {bad_followup_label}.\n", encoding="utf-8")
        bad_successor_phase = root / "successor_phase_taxonomy.md"
        bad_successor_phase_label = "successor-" + DELIVERY_STAGE
        bad_successor_phase.write_text(f"{bad_successor_phase_label} should be rejected.\n", encoding="utf-8")
        bad_continuation_phase = root / "continuation_phase_taxonomy.md"
        bad_continuation_phase_label = "Continuation " + DELIVERY_STAGE
        bad_continuation_phase.write_text(f"{bad_continuation_phase_label} should be rejected.\n", encoding="utf-8")
        bad_future_phases = root / "future_phases_taxonomy.md"
        bad_future_phases_label = "future " + "phases"
        bad_future_phases.write_text(f"{bad_future_phases_label} should be rejected.\n", encoding="utf-8")
        bad_numbered_capabilities = root / "numbered_capabilities_taxonomy.md"
        bad_numbered_capabilities_label = "capabilities " + "0 through 6"
        bad_numbered_capabilities.write_text(f"## {bad_numbered_capabilities_label}\n", encoding="utf-8")
        bad_welded_capability = root / "welded_capability_taxonomy.md"
        bad_welded_capability_label = "process-supervision " + "capability evidence"
        bad_welded_capability.write_text(f"| Surface | {bad_welded_capability_label} | Notes |\n", encoding="utf-8")
        bad_welded_heading = root / "welded_heading_taxonomy.md"
        bad_welded_heading_label = "process-supervision " + "capability Process"
        bad_welded_heading.write_text(f"# {bad_welded_heading_label} Traceability\n", encoding="utf-8")
        bad_this_phase = root / "this_phase_taxonomy.md"
        bad_this_phase_label = "this " + DELIVERY_STAGE
        bad_this_phase.write_text(f"do not scope codebase behavior to {bad_this_phase_label}.\n", encoding="utf-8")
        bad_phase_field = root / "phase_field_taxonomy.md"
        bad_phase_field_label = DELIVERY_STAGE.capitalize() + ": `delivery record`"
        bad_phase_field.write_text(f"{bad_phase_field_label}\n", encoding="utf-8")
        bad_gate_closure = root / "gate_closure_taxonomy.md"
        bad_gate_closure_label = "gate-" + "closure"
        bad_gate_closure.write_text(f"{bad_gate_closure_label} should be rejected.\n", encoding="utf-8")
        bad_closure_slug = root / "closure_slug_taxonomy.md"
        bad_closure_slug_label = "lazy-parity-" + "closure"
        bad_closure_slug.write_text(f"{bad_closure_slug_label} should be rejected.\n", encoding="utf-8")
        bad_rules_alias = root / (RULES_ALIAS + "_alias_taxonomy.md")
        bad_rules_alias_label = "baseline " + RULES_ALIAS
        bad_rules_alias.write_text(f"{bad_rules_alias_label} should be rejected.\n", encoding="utf-8")
        bad_alias_schema = root / (RULES_ALIAS + "_schema_taxonomy.json")
        bad_alias_schema_label = "owner_" + RULES_ALIAS
        bad_alias_schema.write_text(f'{{"{bad_alias_schema_label}": "x"}}\n', encoding="utf-8")
        bad_world_class_phase = root / "world_class_phase_taxonomy.md"
        bad_world_class_phase_label = "world-" + "class verification " + DELIVERY_STAGE
        bad_world_class_phase.write_text(f"{bad_world_class_phase_label} should be rejected.\n", encoding="utf-8")
        bad_local_validation_close = root / "local_validation_close_taxonomy.md"
        bad_local_validation_close_label = "closed from a local validation " + "standpoint"
        bad_local_validation_close.write_text(f"{bad_local_validation_close_label} should be rejected.\n", encoding="utf-8")
        bad_backlog = root / "backlog_taxonomy.md"
        bad_backlog_label = "back" + "log generation"
        bad_backlog.write_text(f"signal-only {bad_backlog_label}.\n", encoding="utf-8")
        bad_implementation_backlog = root / "implementation_backlog_taxonomy.md"
        bad_implementation_backlog_label = "Implementation " + DELIVERY_QUEUE.capitalize()
        bad_implementation_backlog.write_text(f"{bad_implementation_backlog_label}\n", encoding="utf-8")
        bad_backlog_item = root / "backlog_item_taxonomy.md"
        bad_backlog_item_label = DELIVERY_QUEUE.capitalize() + " item"
        bad_backlog_item.write_text(f"| {bad_backlog_item_label} | Evidence |\n", encoding="utf-8")
        bad_capability_backlog = root / "capability_backlog_taxonomy.md"
        bad_capability_backlog_label = "Capability " + DELIVERY_QUEUE.capitalize()
        bad_capability_backlog.write_text(f"## {bad_capability_backlog_label}\n", encoding="utf-8")
        bad_conversion_set = root / "conversion_set_taxonomy.md"
        bad_conversion_set_label = "Conversion " + "Set 3"
        bad_conversion_set.write_text(f"{bad_conversion_set_label} should be rejected.\n", encoding="utf-8")
        bad_closeout = root / "closeout_taxonomy.md"
        bad_closeout_label = "close" + "out"
        bad_closeout.write_text(f"{bad_closeout_label} wording should be rejected.\n", encoding="utf-8")
        bad_work_item = root / "work_item_taxonomy.md"
        bad_work_item_label = "work " + "item"
        bad_work_item.write_text(f"old workflow {bad_work_item_label} wording should be rejected.\n", encoding="utf-8")
        bad_work_item_file = root / ("work-" + "item-helper.md")
        bad_work_item_file.write_text("old workflow helper filename should be rejected.\n", encoding="utf-8")
        bad_work_item_command = root / "work_item_command_taxonomy.md"
        bad_work_item_command_label = "`/" + "work-on-item`"
        bad_work_item_command.write_text(f"{bad_work_item_command_label} should be rejected.\n", encoding="utf-8")
        bad_todo_marker = root / "todo_marker_taxonomy.rs"
        bad_todo_marker_label = "TODO(" + "m6" + ")"
        bad_todo_marker.write_text(f"// {bad_todo_marker_label}: replace delivery marker.\n", encoding="utf-8")
        bad_reference_marker = root / "reference_marker_taxonomy.sifr"
        bad_reference_marker_label = "Reference: " + "m0"
        bad_reference_marker.write_text(f"# {bad_reference_marker_label}\n", encoding="utf-8")
        bad_quoted_marker = root / "quoted_marker_taxonomy.sifr"
        bad_quoted_marker_label = "m" + "14 preamble"
        bad_quoted_marker.write_text(f'write_text(path, "{bad_quoted_marker_label}")\n', encoding="utf-8")
        bad_path_marker = root / "path_marker_taxonomy.sifr"
        bad_path_marker_label = "/m" + "4/http1"
        bad_path_marker.write_text(f'assert request[1] == "{bad_path_marker_label}"\n', encoding="utf-8")
        check_numbered_labels(root)
        failures = collect_failures((root,), audit_root=root)
    rendered = "\n".join(failure.render() for failure in failures)
    if (
        bad_label not in rendered
        or bad_file not in rendered
        or "mixed_allowed_and_bad.rs" not in rendered
        or bad_lowercase_label not in rendered
        or bad_plain_milestone_label not in rendered
        or bad_plain_wave_label not in rendered
        or bad_plain_roadmap_label not in rendered
        or bad_prefix_label not in rendered
        or bad_suffix_label not in rendered
        or bad_bare_label not in rendered
        or bad_alias_label not in rendered
        or bad_capability_pass_label not in rendered
        or bad_capability_id_label not in rendered
        or bad_spaced_filename_label not in rendered
        or bad_digitless_alias_label not in rendered
        or bad_prefixed_alias_label not in rendered
        or bad_task_surface_label not in rendered
        or bad_m_word_label not in rendered
        or bad_surface_alias_label not in rendered
        or bad_alias_record_label not in rendered
        or bad_ad_hoc_phase_label not in rendered
        or bad_source_issue_label not in rendered
        or bad_ad_hoc_issue_label not in rendered
        or bad_numbered_slice_label not in rendered
        or bad_named_slice_label not in rendered
        or bad_followup_label not in rendered
        or bad_successor_phase_label not in rendered
        or bad_continuation_phase_label not in rendered
        or bad_future_phases_label not in rendered
        or bad_numbered_capabilities_label not in rendered
        or bad_welded_capability_label not in rendered
        or bad_welded_heading_label not in rendered
        or bad_this_phase_label not in rendered
        or bad_phase_field_label not in rendered
        or bad_gate_closure_label not in rendered
        or bad_closure_slug_label not in rendered
        or bad_rules_alias_label not in rendered
        or bad_alias_schema_label not in rendered
        or bad_world_class_phase_label not in rendered
        or bad_local_validation_close_label not in rendered
        or bad_backlog_label not in rendered
        or bad_implementation_backlog_label not in rendered
        or bad_backlog_item_label not in rendered
        or bad_capability_backlog_label not in rendered
        or bad_conversion_set_label not in rendered
        or bad_closeout_label not in rendered
        or bad_work_item_label not in rendered
        or DELIVERY_WORK_ITEM + "-helper.md" not in rendered
        or bad_work_item_command_label not in rendered
        or bad_todo_marker_label not in rendered
        or bad_reference_marker_label not in rendered
        or bad_quoted_marker_label not in rendered
        or bad_path_marker_label not in rendered
        or "compiler_interface.rs" in rendered
    ):
        print(f"verification taxonomy self-test failed: {rendered}", file=sys.stderr)
        return 1
    if not quiet:
        print("verification taxonomy self-test ok")
    return 0


def check_numbered_labels(root: Path) -> None:
    demo = root / "example.sifr"
    for source in (
        "    m12: bool = False", "p2 = origin", "let mut m12 = false;",
        "_w2: None = write_text(path, text)",
    ):
        if not validate_demo_variables(demo, source):
            raise AssertionError(f"ambiguous demo variable was accepted: {source}")
    for source in (
        "digit_match: bool = False", "left_root = find_root(node, parents)",
        "let monotonic_start = monotonic();", "p95 = latency", 'print("m12")',
        "# m12 is a sample label", 'grep -F -m1 "matched"',
    ):
        if validate_demo_variables(demo, source):
            raise AssertionError(f"descriptive name or non-variable text was rejected: {source}")
    good = root / "semantic_terms.rs"
    good.write_text(
        'fn contract_error() {}\nlet migration = id("m1");\n'
        'let part2_value = parts.next();\n'
        'let __sifr_bridge_item_1 = value;\n'
        'let p1 = Point::new(0, 0); let m1 = pattern_matches("a", "a");\n'
        'assert_eq!(p1.x, p2.x); let parent = p1.clone();\n'
        'let mean = m1; let growth = value.exp_m1();\n'
        'let state = "sifr.sql.migration.state.m1.baseline.0.aaaa";\n'
        'let p95_ms = 5; let p99_ms = 10; let P50_MAX = 20;\n'
        'grep -F -m1 "matched"\n'
        '// compiler phase; collection item; slice step; ad hoc expressions\n'
        '// PostgreSQL DDL and capability evidence\n',
        encoding="utf-8",
    )
    if validate_text(good, audit_root=root) or validate_filename(good, audit_root=root):
        raise AssertionError("technical terminology was rejected")
    for label in (
        "item8", "item_10h", "part6", "phase40", "wave_2", "sprint3",
        "m1", "p1", "ms2", "pt3", "ph4", "w5", "M8", "P2", "m1_2",
    ):
        directory = root / ("example_" + label)
        directory.mkdir()
        source = directory / "main.sifr"
        source.write_text("def main(): pass\n", encoding="utf-8")
        if not validate_filename(source, audit_root=root):
            raise AssertionError(f"numbered directory label was accepted: {label}")
    for suffix in (".rs", ".sifr", ".json", ".mdx", ".sh"):
        source = root / ("numbered_label" + suffix)
        for line in (
            "fn item8_cleanup() {}", '# Part 3: captures',
            '{"owner_item": 3}', '{"phase": "39_rust_interop"}',
            '# Reference: compiler-feature-history',
            'let migration = id("m1"); // Item 8',
            'fn contract_error() {} // Phase 4',
            '@rust(sifr_stdlib.m8.noop)',
            '"/opt/sifr/stdlib/_sifr/p1.sifr"',
            '"stable-token-m6"', '"SIFR_ENV_P30_MISSING"',
            'fn lower_p1_captures() {}', 'fn m2_lowering() {}',
            '# p1: captures', '# Reference: pt2', '// TODO(ph3)',
            '{"owner": "w2"}', '{"stage": "p1 lowering"}',
            'let migration = id("m1"); // p1: captures',
            'let p95_ms = 5; // p1: captures',
        ):
            source.write_text(line + "\n", encoding="utf-8")
            if should_skip(source, audit_root=root) or not validate_text(source, audit_root=root):
                raise AssertionError(f"numbered delivery label was accepted: {line}")
    for name in ("budget_p95_regression_result.json", "p50_latency.rs", "p99_latency.rs"):
        if validate_filename(root / name, audit_root=root):
            raise AssertionError(f"percentile filename was rejected: {name}")
    skipped = root / "skills" / "example"
    skipped.mkdir(parents=True)
    (skipped / "SKILL.md").write_text("Phase 99\n", encoding="utf-8")
    if collect_failures((root / "skills",), audit_root=root):
        raise AssertionError("skill files entered the codebase scan")


def repo_path(path: Path) -> str:
    try:
        return path.relative_to(REPO_ROOT).as_posix()
    except ValueError:
        return path.as_posix()


if __name__ == "__main__":
    raise SystemExit(main())
