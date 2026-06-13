#!/usr/bin/env python3
"""Generate concurrency/runtime CPython evidence and Sifr inventory artifacts."""

from __future__ import annotations

import ast
import json
import re
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[1]
CPYTHON_ROOT = Path("/Users/yaseralnajjar/work/sifr/cpython")

INVENTORY_JSON = REPO_ROOT / "verification/stdlib/concurrency_runtime_substrate_inventory.json"
INVENTORY_MD = REPO_ROOT / "verification/stdlib/concurrency_runtime_substrate_inventory.md"
EVIDENCE_MD = REPO_ROOT / "verification/stdlib/concurrency_runtime_cpython_evidence_matrix.md"
WORKLOAD_MD = REPO_ROOT / "verification/stdlib/concurrency_runtime_workload_database.md"
M0_TRACEABILITY_MD = REPO_ROOT / "verification/stdlib/concurrency_runtime_m0_traceability.md"


SOURCE_GROUPS: dict[str, list[str]] = {
    "subprocess/process": [
        "Lib/subprocess.py",
        "Lib/asyncio/subprocess.py",
        "Doc/library/subprocess.rst",
        "Doc/library/asyncio-subprocess.rst",
        "Lib/test/test_subprocess.py",
        "Lib/test/test_asyncio/test_subprocess.py",
        "Modules/_posixsubprocess.c",
        "Modules/clinic/_posixsubprocess.c.h",
    ],
    "queue/concurrency": [
        "Lib/queue.py",
        "Lib/asyncio/*.py",
        "Lib/concurrent/futures/*.py",
        "Lib/multiprocessing/*.py",
        "Doc/library/queue.rst",
        "Doc/library/asyncio.rst",
        "Doc/library/concurrent.futures.rst",
        "Doc/library/multiprocessing*.rst",
        "Lib/test/test_queue.py",
        "Lib/test/test_asyncio/test_queues.py",
        "Lib/test/test_asyncio/test_tasks.py",
        "Lib/test/test_asyncio/test_taskgroups.py",
        "Lib/test/test_asyncio/test_waitfor.py",
        "Lib/test/test_asyncio/test_timeouts.py",
        "Lib/test/test_asyncio/test_locks.py",
        "Lib/test/test_asyncio/test_runners.py",
        "Lib/test/test_concurrent_futures/",
        "Lib/test/_test_multiprocessing.py",
        "Lib/test/test_multiprocessing_main_handling.py",
        "Lib/test/test_multiprocessing_spawn/",
        "Lib/test/test_multiprocessing_fork/",
        "Lib/test/test_multiprocessing_forkserver/",
        "Modules/_queuemodule.c",
        "Modules/_multiprocessing/*",
        "Modules/clinic/_queuemodule.c.h",
    ],
    "context/warnings/signal": [
        "Lib/contextlib.py",
        "Lib/warnings.py",
        "Doc/library/contextlib.rst",
        "Doc/library/warnings.rst",
        "Doc/library/signal.rst",
        "Lib/test/test_contextlib.py",
        "Lib/test/test_contextlib_async.py",
        "Lib/test/test_warnings/",
        "Lib/test/test_signal.py",
        "Lib/test/test_io/test_signals.py",
        "Modules/signalmodule.c",
        "Python/_warnings.c",
        "Lib/_py_warnings.py",
    ],
}

PRODUCTION_SURFACES: list[dict[str, str]] = [
    {
        "surface": "sifr.task.TaskHandle[T, E]",
        "owner_milestone": "milestone_concurrency_runtime_1",
        "support_tier": "production-public",
        "terminal_state": "production-public",
        "stability": "stable-public-api",
        "notes": "Public affine observation handle; generated/internal Task names remain internal-only.",
    },
    {
        "surface": "sifr.task.TaskGroup[E]",
        "owner_milestone": "milestone_concurrency_runtime_1",
        "support_tier": "production-public",
        "terminal_state": "production-public",
        "stability": "stable-public-api",
        "notes": "Canonical mixed structured-runtime owner; no distinct Scope is introduced in M0.",
    },
    {
        "surface": "sifr.task.spawn_scoped/sleep/timeout/deadline/cancel_scope/join_all/race/select",
        "owner_milestone": "milestone_concurrency_runtime_1",
        "support_tier": "production-public",
        "terminal_state": "production-public",
        "stability": "stable-public-api",
        "notes": "Stable task helpers with typed timeout/cancellation evidence and no public Tokio types.",
    },
    {
        "surface": "sifr.sync channels, locks, semaphores, events",
        "owner_milestone": "milestone_concurrency_runtime_2",
        "support_tier": "production-public",
        "terminal_state": "production-public",
        "stability": "stable-public-api",
        "notes": "Native bounded backpressure and synchronization replace queue parity.",
    },
    {
        "surface": "sifr.runtime spawn_blocking/spawn_cpu/JoinSet",
        "owner_milestone": "milestone_concurrency_runtime_3",
        "support_tier": "production-public",
        "terminal_state": "production-public",
        "stability": "stable-public-api",
        "notes": "Structured blocking and CPU offload; JoinSet is homogeneous and must be consumed.",
    },
    {
        "surface": "sifr.parallel map/try_map/Pool/PoolConfig",
        "owner_milestone": "milestone_concurrency_runtime_3",
        "support_tier": "production-public",
        "terminal_state": "production-public",
        "stability": "stable-public-api",
        "notes": "Private Rayon pools, fixed default sizing from available_parallelism(), no global pool mutation.",
    },
    {
        "surface": "sifr.process Command/Child/ProcessHandle/owned pipes",
        "owner_milestone": "milestone_concurrency_runtime_4",
        "support_tier": "production-public",
        "terminal_state": "production-public",
        "stability": "stable-public-api",
        "notes": "M0 chooses distinct ProcessHandle for scoped process supervision while preserving pipe access.",
    },
    {
        "surface": "sifr.signal structured shutdown streams",
        "owner_milestone": "milestone_concurrency_runtime_5",
        "support_tier": "production-public",
        "terminal_state": "production-public",
        "stability": "stable-public-api",
        "notes": "Portable `Signal`, `SIGINT`, `SIGTERM`, and `strsignal` value-model evidence is importable; structured streams remain M5 work and arbitrary signal.signal handlers are unsupported.",
    },
    {
        "surface": "sifr.resource ExitStack/AsyncExitStack/closing/aclosing/nullcontext",
        "owner_milestone": "milestone_concurrency_runtime_5",
        "support_tier": "production-public",
        "terminal_state": "production-public",
        "stability": "stable-public-api",
        "notes": "`nullcontext(...)` covers no-value and value-carrying generic helper evidence; ExitStack/AsyncExitStack/closing/aclosing are closed as unsupported diagnostics until cleanup-error and owned-close protocols are implemented.",
    },
    {
        "surface": "sifr.task.Context/ContextKey[T]",
        "owner_milestone": "milestone_concurrency_runtime_5",
        "support_tier": "production-public",
        "terminal_state": "production-public",
        "stability": "stable-public-api",
        "notes": "Value-model foundation is importable; explicit propagation remains M5 work with no contextvars parity or implicit dynamic mutation.",
    },
    {
        "surface": "sifr.ipc typed frame protocol",
        "owner_milestone": "milestone_concurrency_runtime_6",
        "support_tier": "production-substrate",
        "terminal_state": "production-substrate",
        "stability": "stable-production-substrate",
        "notes": "Typed, versioned IPC over accepted process pipes; no pickle-style arbitrary object transport.",
    },
]

LEGACY_SURFACES: list[dict[str, str]] = [
    {
        "surface": "sifr.asyncio",
        "terminal_state": "unsupported-with-diagnostic",
        "stability": "compiler-known-intrinsic",
        "replacement": "sifr.task and sifr.sync",
        "owner_milestone": "milestone_concurrency_runtime_0a",
        "revisit_rule": "Only a new Sifr-native API design can revisit this; migration compatibility is insufficient.",
    },
    {
        "surface": "sifr.subprocess",
        "terminal_state": "unsupported-with-diagnostic",
        "stability": "compiler-known-intrinsic",
        "replacement": "sifr.process",
        "owner_milestone": "milestone_concurrency_runtime_0a",
        "revisit_rule": "No CPython-shaped process adapter survives M0a.",
    },
    {
        "surface": "sifr.queue",
        "terminal_state": "unsupported-with-diagnostic",
        "stability": "compiler-known-intrinsic",
        "replacement": "sifr.sync",
        "owner_milestone": "milestone_concurrency_runtime_0a",
        "revisit_rule": "Future queue API must prove Sifr-native value over channels.",
    },
    {
        "surface": "sifr.concurrent.futures",
        "terminal_state": "unsupported-with-diagnostic",
        "stability": "compiler-known-intrinsic",
        "replacement": "sifr.runtime and sifr.parallel",
        "owner_milestone": "milestone_concurrency_runtime_0a",
        "revisit_rule": "Future adapter requires a separate design gate and cannot be the offload spine.",
    },
    {
        "surface": "sifr.multiprocessing",
        "terminal_state": "rejected",
        "stability": "compiler-known-intrinsic",
        "replacement": "sifr.process plus sifr.ipc",
        "owner_milestone": "milestone_concurrency_runtime_0a",
        "revisit_rule": "Process workers require the M6 typed IPC design and a future process-worker phase.",
    },
    {
        "surface": "sifr.threading",
        "terminal_state": "unsupported-with-diagnostic",
        "stability": "compiler-known-intrinsic",
        "replacement": "sifr.runtime, sifr.parallel, and sifr.sync",
        "owner_milestone": "milestone_concurrency_runtime_0a",
        "revisit_rule": "Raw thread objects are not the public execution model.",
    },
    {
        "surface": "Python warnings global filter model",
        "terminal_state": "rejected",
        "stability": "compiler-known-intrinsic",
        "replacement": "structured diagnostics/tracing events",
        "owner_milestone": "milestone_concurrency_runtime_5",
        "revisit_rule": "No process-global warning/filter mutation from concurrent contexts.",
    },
    {
        "surface": "sifr.contextlib",
        "terminal_state": "unsupported-with-diagnostic",
        "stability": "compiler-known-intrinsic",
        "replacement": "sifr.resource",
        "owner_milestone": "milestone_concurrency_runtime_5",
        "revisit_rule": "Generator decorator helpers require a future generator semantics phase; cleanup scopes remain Sifr-native.",
    },
    {
        "surface": "sifr.warnings",
        "terminal_state": "rejected",
        "stability": "compiler-known-intrinsic",
        "replacement": "structured diagnostics/tracing events",
        "owner_milestone": "milestone_concurrency_runtime_5",
        "revisit_rule": "No Python global warning filter adapter ships in this phase.",
    },
]

M0_DECISIONS: list[dict[str, str]] = [
    {
        "decision": "mixed runtime owner",
        "outcome": "Use TaskGroup[E] as the canonical mixed owner; no distinct task.Scope/runtime.Scope is introduced in M0.",
        "evidence": "Server shutdown, process pump tasks, blocking offload, and CPU offload can be represented as typed child handles under TaskGroup or JoinSet without non-fail-fast scope semantics.",
    },
    {
        "decision": "public task handle name",
        "outcome": "TaskHandle[T, E] is public; Task and BlockingTask remain internal/generated names only.",
        "evidence": "Existing generated lowering already treats task and blocking handles as affine observation handles.",
    },
    {
        "decision": "observed TaskGroup failures",
        "outcome": "An explicitly awaited and statically handled child failure is observed and is not re-reported at group exit; unhandled child failure triggers fail-fast sibling cancellation and TaskGroupError[E].",
        "evidence": "Matches the phase's static handled-failure proof while preserving deterministic fail-fast group exit.",
    },
    {
        "decision": "race/select containers",
        "outcome": "race returns RaceResult { winner_index, outcome, loser_cancellations }; select returns SelectResult { branch_tag, outcome, loser_cancellations }.",
        "evidence": "Both use homogeneous result/error types unless the user supplies an explicit sum/enum type.",
    },
    {
        "decision": "select syntax",
        "outcome": "select uses a compiler-known named-branch form task.select(name=awaitable, ...); keyword names are static branch tags.",
        "evidence": "Static branch identity is enforced without exposing runtime callback transports or event-loop objects.",
    },
    {
        "decision": "scoped process handle shape",
        "outcome": "TaskGroup.spawn_process returns a distinct ProcessHandle that owns Child status observation plus stdin/stdout/stderr pipe access.",
        "evidence": "TaskHandle[Status, SubprocessError] would lose first-class pipe ownership; returning Child alone would blur supervision and observation.",
    },
    {
        "decision": "offload error binding",
        "outcome": "TaskGroup.spawn_blocking/spawn_cpu require TaskGroup[WorkerError[E]] and align with JoinSet.join_all().await -> list[Result[T, WorkerError[E]]].",
        "evidence": "Runtime worker failures and user E remain typed under one homogeneous group error type.",
    },
    {
        "decision": "lock/permit await policy",
        "outcome": "Sync lock guards cannot cross await. Async lock guards are await-forbidden in M2 unless a specific guard is marked await-safe. Semaphore permits are guard-like: they cannot cross await and cannot escape through returns.",
        "evidence": "Prevents hidden shared mutable state and unbounded permit retention across suspension points.",
    },
    {
        "decision": "Barrier and Once",
        "outcome": "Barrier is deferred-to-phase-X unless M2 finds a near-term production need; Once remains internal-only over std::sync::OnceLock.",
        "evidence": "Channels, locks, semaphores, and events cover near-term web/worker/data needs without toy public primitives.",
    },
]

MILESTONE_BACKLOG: list[dict[str, str]] = [
    {
        "milestone": "milestone_concurrency_runtime_0a",
        "artifact": "concurrency_runtime_m0a_legacy_surface_traceability.md",
        "acceptance": "legacy public sifr.asyncio/sifr.subprocess/sifr.concurrent/sifr.threading surfaces removed, hidden, or diagnosed; native imports remain canonical",
        "fixtures": "bare_cpython_asyncio/queue/subprocess/concurrent_futures/multiprocessing/signal/contextlib/warnings/threading import fixtures; legacy sifr.* unsupported-diagnostic fixtures",
    },
    {
        "milestone": "milestone_concurrency_runtime_1",
        "artifact": "concurrency_runtime_m1_task_traceability.md",
        "acceptance": "TaskHandle/TaskGroup/cancel_scope/race/select/timeout behavior typed and no public Tokio leak",
        "fixtures": "task sendability, observed failure, unobserved drop, race/select loser evidence fixtures",
    },
    {
        "milestone": "milestone_concurrency_runtime_2",
        "artifact": "concurrency_runtime_m2_sync_traceability.md",
        "acceptance": "bounded backpressure, close, cancellation, sendability, and lock/permit await diagnostics",
        "fixtures": "producer/consumer pipeline, channel non-send, sync lock across await, semaphore permit policy fixtures",
    },
    {
        "milestone": "milestone_concurrency_runtime_3",
        "artifact": "concurrency_runtime_m3_offload_traceability.md",
        "acceptance": "spawn_blocking/spawn_cpu/JoinSet/parallel map use typed WorkerError and private Rayon pools",
        "fixtures": "JoinSet drop/order/cancel, non-send CPU capture, async direct parallel.map diagnostic fixtures",
    },
    {
        "milestone": "milestone_concurrency_runtime_4",
        "artifact": "concurrency_runtime_m4_process_traceability.md",
        "acceptance": "sifr.process sync/async child supervision, owned pipes, text mode, shell_exec effect, timeout/cancel",
        "fixtures": "sync/async subprocess loopback, pipe ownership, shell effect, process cancellation fixtures",
    },
    {
        "milestone": "milestone_concurrency_runtime_5",
        "artifact": "concurrency_runtime_m5_shutdown_traceability.md",
        "acceptance": "structured signals, cleanup stacks, explicit task Context, diagnostics/tracing policy",
        "fixtures": "shutdown stream, cleanup under cancellation, context propagation, signal.signal/warnings rejection fixtures",
    },
    {
        "milestone": "milestone_concurrency_runtime_6",
        "artifact": "concurrency_runtime_m6_ipc_traceability.md",
        "acceptance": "typed IPC frame protocol, schema identity, version negotiation, malformed-frame errors, payload diagnostics",
        "fixtures": "schema accept/reject, unsupported payload, malformed frame, cancellation frame fixtures",
    },
    {
        "milestone": "milestone_concurrency_runtime_7",
        "artifact": "concurrency_runtime_m7_closeout_traceability.md",
        "acceptance": "docs, demos, validation lanes, panic scans, final inventory closure, final external PASS review",
        "fixtures": "structured task demo, channel pipeline demo, offload demo, process demo, shutdown/cleanup demo",
    },
]

WORKLOAD_ROWS: list[dict[str, str]] = [
    {"api": "sifr.task.sleep", "owner": "M1", "classification": "async-suspension", "validation": "task sleep fixture"},
    {"api": "sifr.task.timeout/deadline", "owner": "M1", "classification": "async-suspension cancellation", "validation": "timeout/deadline evidence fixture"},
    {"api": "sifr.task.cancel_scope", "owner": "M1", "classification": "async-suspension cancellation", "validation": "cancel-scope fixture"},
    {"api": "sifr.sync.Channel.send/receive async forms", "owner": "M2", "classification": "async-suspension backpressure", "validation": "channel backpressure and cancellation fixtures"},
    {"api": "sifr.sync.Channel send/receive sync forms", "owner": "M2", "classification": "@blocking_io-equivalent sync wait", "validation": "blocking-in-async diagnostic fixture"},
    {"api": "sifr.sync.Mutex/RwLock sync lock", "owner": "M2", "classification": "@blocking_io-equivalent sync wait", "validation": "lock direct async diagnostic fixture"},
    {"api": "sifr.sync.AsyncMutex/AsyncRwLock/Semaphore/Event", "owner": "M2", "classification": "async-suspension", "validation": "async sync primitive fixtures"},
    {"api": "sifr.runtime.spawn_blocking", "owner": "M3", "classification": "@blocking_io offload boundary", "validation": "spawn_blocking typed WorkerError fixture"},
    {"api": "sifr.task.spawn_cpu", "owner": "M3", "classification": "@cpu_heavy offload boundary with typed runtime/worker evidence", "validation": "`spawn_cpu_basic`, `spawn_cpu_user_error_typed`, `spawn_cpu_worker_panic_typed`, `spawn_cpu_unannotated_rejected`, `spawn_cpu_blocking_io_rejected`, `spawn_cpu_non_send_rejected`"},
    {"api": "sifr.task.TaskScope/TaskGroup scoped offload", "owner": "M3", "classification": "@blocking_io/@cpu_heavy scoped owner offload with typed task evidence", "validation": "`task_scope_spawn_blocking`, `task_group_spawn_cpu`, `task_group_spawn_cpu_user_error`, `task_scope_spawn_cpu_unannotated_rejected`, `task_group_spawn_blocking_error_mismatch_rejected`"},
    {"api": "sifr.task.JoinSet", "owner": "M3", "classification": "homogeneous task/offload collection with explicit observation/cancellation", "validation": "`join_set_add_task_join_all`, `join_set_spawn_cpu_join_all_ordered`, `join_set_cancel_all_evidence`, `join_set_cancel_all_task_cancelled`, `join_set_spawn_blocking`, `join_set_bound_terminal_await`, `join_set_reassign_live_rejected`, `join_set_unconsumed_rejected`, `join_set_terminal_must_be_awaited_rejected`"},
    {"api": "sifr.parallel.map/try_map", "owner": "M3", "classification": "@cpu_heavy synchronous, typed worker-runtime boundary", "validation": "`parallel_map_basic`, `parallel_try_map_basic`, `parallel_map_worker_panic_typed`, `parallel_try_map_user_error_typed`, async direct-call diagnostic fixture"},
    {"api": "sifr.process.run/output/wait sync", "owner": "M4", "classification": "@blocking_io plus optional @shell_exec", "validation": "process blocking-in-async and shell-effect fixtures"},
    {"api": "sifr.process async spawn/wait/communicate", "owner": "M4", "classification": "async-suspension plus optional @shell_exec", "validation": "async process loopback fixture"},
    {"api": "sifr.signal.shutdown_stream/ctrl_c/terminate", "owner": "M5", "classification": "async-suspension host-limited", "validation": "signal host matrix fixture"},
    {"api": "sifr.resource.AsyncExitStack", "owner": "M5", "classification": "async cleanup under cancellation", "validation": "async cleanup cancellation fixture"},
    {"api": "sifr.ipc.Connection send/receive", "owner": "M6", "classification": "async-suspension backpressure serialization", "validation": "IPC frame/malformed/cancel fixtures"},
]


@dataclass(frozen=True)
class ExpandedPath:
    domain: str
    pattern: str
    relative_path: str
    path: Path


def cpython_commit() -> str:
    return subprocess.check_output(
        ["git", "-C", str(CPYTHON_ROOT), "rev-parse", "HEAD"],
        text=True,
    ).strip()


def expand_paths() -> list[ExpandedPath]:
    expanded: list[ExpandedPath] = []
    for domain, patterns in SOURCE_GROUPS.items():
        for pattern in patterns:
            root = CPYTHON_ROOT / pattern
            paths: list[Path]
            if pattern.endswith("/"):
                paths = sorted(path for path in root.rglob("*") if path.is_file())
            elif any(ch in pattern for ch in "*?["):
                paths = sorted(CPYTHON_ROOT.glob(pattern))
                paths = [path for path in paths if path.is_file()]
            else:
                paths = [root] if root.exists() and root.is_file() else []
            for path in paths:
                expanded.append(
                    ExpandedPath(
                        domain=domain,
                        pattern=pattern,
                        relative_path=path.relative_to(CPYTHON_ROOT).as_posix(),
                        path=path,
                    )
                )
    return sorted({item.relative_path: item for item in expanded}.values(), key=lambda item: item.relative_path)


def public_name(name: str) -> bool:
    return bool(name) and not name.startswith("_")


def parse_python(path: Path) -> dict[str, Any]:
    text = path.read_text(encoding="utf-8", errors="replace")
    result: dict[str, Any] = {
        "public_functions": [],
        "public_classes": [],
        "public_methods": [],
        "public_constants": [],
        "keyword_forms": [],
        "test_classes": [],
        "test_methods": [],
        "deprecation_markers": deprecation_markers(text),
    }
    try:
        tree = ast.parse(text, filename=str(path))
    except SyntaxError:
        return result
    for node in tree.body:
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) and public_name(node.name):
            result["public_functions"].append(node.name)
            result["keyword_forms"].extend(keyword_forms(node.name, node.args))
        elif isinstance(node, ast.ClassDef) and public_name(node.name):
            result["public_classes"].append(node.name)
            if node.name.startswith("Test") or node.name.endswith("Test") or "Test" in node.name:
                result["test_classes"].append(node.name)
            for item in node.body:
                if isinstance(item, (ast.FunctionDef, ast.AsyncFunctionDef)):
                    if public_name(item.name):
                        result["public_methods"].append(f"{node.name}.{item.name}")
                        result["keyword_forms"].extend(keyword_forms(f"{node.name}.{item.name}", item.args))
                    if item.name.startswith("test"):
                        result["test_methods"].append(f"{node.name}.{item.name}")
        elif isinstance(node, (ast.Assign, ast.AnnAssign)):
            names = assignment_names(node)
            result["public_constants"].extend(name for name in names if public_name(name) and name.upper() == name)
    return {key: sorted(set(value)) if isinstance(value, list) else value for key, value in result.items()}


def assignment_names(node: ast.Assign | ast.AnnAssign) -> list[str]:
    targets = node.targets if isinstance(node, ast.Assign) else [node.target]
    names: list[str] = []
    for target in targets:
        if isinstance(target, ast.Name):
            names.append(target.id)
        elif isinstance(target, (ast.Tuple, ast.List)):
            names.extend(item.id for item in target.elts if isinstance(item, ast.Name))
    return names


def keyword_forms(prefix: str, args: ast.arguments) -> list[str]:
    names = [arg.arg for arg in args.kwonlyargs]
    positional = args.args[-len(args.defaults) :] if args.defaults else []
    names.extend(arg.arg for arg in positional)
    return [f"{prefix}({name}=...)" for name in names if public_name(name)]


def parse_c_like(path: Path) -> dict[str, Any]:
    text = path.read_text(encoding="utf-8", errors="replace")
    method_names = sorted(set(re.findall(r'\{\s*"([A-Za-z][A-Za-z0-9_]*)"\s*,', text)))
    constants = sorted(set(re.findall(r'\b(SIG[A-Z0-9_]+|F_[A-Z0-9_]+|HAVE_[A-Z0-9_]+)\b', text)))
    return {
        "public_functions": [name for name in method_names if public_name(name)],
        "public_classes": [],
        "public_methods": [],
        "public_constants": constants,
        "keyword_forms": sorted(set(re.findall(r"\b([a-zA-Z_][a-zA-Z0-9_]+)\s*=", text)))[:80],
        "test_classes": [],
        "test_methods": [],
        "deprecation_markers": deprecation_markers(text),
    }


def parse_rst(path: Path) -> dict[str, Any]:
    text = path.read_text(encoding="utf-8", errors="replace")
    directive_names = re.findall(r"^\.\.\s+(?:function|class|method|data|attribute)::\s+([A-Za-z_][\w.]*)", text, re.MULTILINE)
    inline_calls = re.findall(r":(?:func|class|meth|data):`~?([A-Za-z_][\w.]*)`", text)
    return {
        "public_functions": sorted(set(name for name in directive_names + inline_calls if public_name(name.split(".")[-1]))),
        "public_classes": [],
        "public_methods": [],
        "public_constants": [],
        "keyword_forms": sorted(set(re.findall(r"\b([a-zA-Z_][a-zA-Z0-9_]*)=", text)))[:80],
        "test_classes": [],
        "test_methods": [],
        "deprecation_markers": deprecation_markers(text),
    }


def deprecation_markers(text: str) -> list[str]:
    markers: list[str] = []
    for line in text.splitlines():
        if re.search(r"\b(deprecat|legacy|obsolete|removed|pending removal)\w*", line, re.IGNORECASE):
            cleaned = " ".join(line.strip().split())
            if cleaned:
                markers.append(cleaned[:220])
    return sorted(set(markers))[:40]


def scan_file(item: ExpandedPath) -> dict[str, Any]:
    suffix = item.path.suffix
    if suffix == ".py":
        extracted = parse_python(item.path)
    elif suffix == ".rst":
        extracted = parse_rst(item.path)
    else:
        extracted = parse_c_like(item.path)
    counts = {
        key: len(value)
        for key, value in extracted.items()
        if isinstance(value, list)
    }
    return {
        "path": item.relative_path,
        "domain": item.domain,
        "source_pattern": item.pattern,
        "classification": classify_file(item.relative_path),
        "counts": counts,
        "extracted": extracted,
    }


def classify_file(relative_path: str) -> dict[str, str]:
    if "/test" in relative_path or relative_path.startswith("Lib/test/"):
        return {
            "support_tier": "test-only-harness",
            "terminal_state": "test-only-harness",
            "stability": "test-only-harness",
            "evidence_state": evidence_state(relative_path),
            "native_mapping": native_mapping(relative_path),
        }
    return {
        "support_tier": "internal-only",
        "terminal_state": "unsupported-with-diagnostic",
        "stability": "compiler-known-intrinsic",
        "evidence_state": evidence_state(relative_path),
        "native_mapping": native_mapping(relative_path),
    }


def evidence_state(relative_path: str) -> str:
    if "multiprocessing" in relative_path:
        return "rejected" if relative_path.endswith((".rst", ".py")) and "test" not in relative_path else "mined-as-substrate-fixture"
    if "concurrent/futures" in relative_path:
        return "adapted-for-sifr-api"
    if "asyncio" in relative_path and any(part in relative_path for part in ("events", "protocols", "transports", "selector", "proactor", "unix_events", "windows")):
        return "rejected"
    if any(part in relative_path for part in ("subprocess", "queue", "asyncio", "contextlib", "signal", "warnings")):
        return "adapted-for-sifr-api"
    return "mined-as-substrate-fixture"


def native_mapping(relative_path: str) -> str:
    if "subprocess" in relative_path or "_posixsubprocess" in relative_path:
        return "sifr.process"
    if "queue" in relative_path or "queues" in relative_path or "locks" in relative_path:
        return "sifr.sync"
    if "concurrent/futures" in relative_path:
        return "sifr.runtime / sifr.parallel"
    if "multiprocessing" in relative_path:
        return "sifr.ipc deferred worker substrate"
    if "contextlib" in relative_path:
        return "sifr.resource"
    if "warnings" in relative_path or "_warnings" in relative_path:
        return "structured diagnostics"
    if "signal" in relative_path:
        return "sifr.signal"
    if "asyncio" in relative_path:
        return "sifr.task"
    return "phase evidence"


def aggregate_domains(files: list[dict[str, Any]]) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for domain in sorted({file["domain"] for file in files}):
        subset = [file for file in files if file["domain"] == domain]
        counts: dict[str, int] = {}
        for file in subset:
            for key, value in file["counts"].items():
                counts[key] = counts.get(key, 0) + value
        rows.append({"domain": domain, "files": len(subset), "counts": counts})
    return rows


def write_inventory_json(files: list[dict[str, Any]], commit: str) -> None:
    data = {
        "schema_version": 1,
        "status": "milestone_concurrency_runtime_7-inventory-audited",
        "platform_contract": "verification/areas/runtime_platform/platform_contract.json",
        "cpython_checkout": {"path": str(CPYTHON_ROOT), "commit": commit},
        "source_patterns": SOURCE_GROUPS,
        "domain_summary": aggregate_domains(files),
        "production_surfaces": PRODUCTION_SURFACES,
        "legacy_python_shaped_surfaces": LEGACY_SURFACES,
        "m0_resolved_decisions": M0_DECISIONS,
        "milestone_backlog": MILESTONE_BACKLOG,
        "workload_database": WORKLOAD_ROWS,
        "scanned_files": files,
    }
    INVENTORY_JSON.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def md_table(headers: list[str], rows: list[list[str]]) -> str:
    out = ["| " + " | ".join(headers) + " |", "| " + " | ".join("---" for _ in headers) + " |"]
    out.extend("| " + " | ".join(cell.replace("\n", " ") for cell in row) + " |" for row in rows)
    return "\n".join(out)


def write_inventory_md(files: list[dict[str, Any]], commit: str) -> None:
    domain_rows = [
        [
            row["domain"],
            str(row["files"]),
            str(row["counts"].get("public_functions", 0)),
            str(row["counts"].get("public_classes", 0)),
            str(row["counts"].get("public_methods", 0)),
            str(row["counts"].get("public_constants", 0)),
            str(row["counts"].get("test_methods", 0)),
        ]
        for row in aggregate_domains(files)
    ]
    surface_rows = [
        [item["surface"], item["owner_milestone"], item["terminal_state"], item["stability"], item["notes"]]
        for item in PRODUCTION_SURFACES
    ]
    legacy_rows = [
        [item["surface"], item["owner_milestone"], item["terminal_state"], item["replacement"], item["revisit_rule"]]
        for item in LEGACY_SURFACES
    ]
    decision_rows = [[item["decision"], item["outcome"], item["evidence"]] for item in M0_DECISIONS]
    backlog_rows = [[item["milestone"], item["artifact"], item["acceptance"], item["fixtures"]] for item in MILESTONE_BACKLOG]
    INVENTORY_MD.write_text(
        "\n\n".join(
            [
                "# Concurrency Runtime Substrate Inventory",
                "Status: M7 inventory audited; generated by `scripts/generate_concurrency_runtime_inventory.py`.",
                f"CPython checkout: `{CPYTHON_ROOT}` at `{commit}`.",
                "Platform contract: [platform_contract.md](../platform/platform_contract.md).",
                "## Scan Summary\n\n"
                + md_table(
                    ["Domain", "Files", "Functions", "Classes", "Methods", "Constants", "Test methods"],
                    domain_rows,
                ),
                "## Production Native Surface Boundary\n\n"
                + md_table(["Surface", "Milestone", "Terminal state", "Stability", "Notes"], surface_rows),
                "## Legacy Python-Shaped Surface Disposition\n\n"
                + md_table(["Surface", "Milestone", "Terminal state", "Replacement", "Revisit rule"], legacy_rows),
                "## M0 Resolved Decisions\n\n" + md_table(["Decision", "Outcome", "Evidence"], decision_rows),
                "## Implementation Backlog\n\n"
                + md_table(["Milestone", "Traceability artifact", "Acceptance", "Representative fixtures"], backlog_rows),
                "## Regeneration\n\nRun `python3 scripts/generate_concurrency_runtime_inventory.py` after changing the phase source-of-truth list, CPython checkout, or M0 decisions.",
            ]
        )
        + "\n",
        encoding="utf-8",
    )


def write_evidence_md(files: list[dict[str, Any]], commit: str) -> None:
    rows = []
    for file in files:
        counts = file["counts"]
        rows.append(
            [
                f"`{file['path']}`",
                file["domain"],
                file["classification"]["native_mapping"],
                file["classification"]["evidence_state"],
                ", ".join(f"{key}={value}" for key, value in counts.items() if value),
            ]
        )
    EVIDENCE_MD.write_text(
        "\n\n".join(
            [
                "# Concurrency Runtime CPython Evidence Matrix",
                "Status: M7 inventory audited; generated from the phase source-of-truth list.",
                f"CPython checkout: `{CPYTHON_ROOT}` at `{commit}`.",
                md_table(["Reference", "Domain", "Native mapping", "Evidence state", "Extracted signal"], rows),
                "## Notes\n\nCPython module shapes are evidence only. Production Sifr APIs are native `sifr.*` surfaces, and CPython-shaped imports are rejected or diagnosed according to the inventory.",
            ]
        )
        + "\n",
        encoding="utf-8",
    )


def write_workload_md() -> None:
    rows = [[row["api"], row["owner"], row["classification"], row["validation"]] for row in WORKLOAD_ROWS]
    WORKLOAD_MD.write_text(
        "\n\n".join(
            [
                "# Concurrency Runtime Workload Database",
                "Status: M7 inventory audited; implementation milestones have recorded validation evidence for accepted concurrency/runtime surfaces.",
                md_table(["API", "Owner", "Workload/effect classification", "Validation"], rows),
                "## Rules\n\nSync APIs that can wait on channels, locks, processes, pipes, or external runtime state are classified as blocking and remain invalid in `async def` unless explicitly offloaded. CPU-heavy APIs use `@cpu_heavy` and must route through `spawn_cpu` in async contexts. Shell subprocess APIs carry `@shell_exec` in addition to blocking or async suspension classification.",
            ]
        )
        + "\n",
        encoding="utf-8",
    )


def write_m0_traceability_md() -> None:
    rows = [
        ["CPython source scan", "verification/stdlib/concurrency_runtime_substrate_inventory.json"],
        ["Human inventory", "verification/stdlib/concurrency_runtime_substrate_inventory.md"],
        ["Evidence matrix", "verification/stdlib/concurrency_runtime_cpython_evidence_matrix.md"],
        ["Workload database", "verification/stdlib/concurrency_runtime_workload_database.md"],
        ["Shared platform contract", "verification/areas/runtime_platform/platform_contract.md and .json"],
        ["Supported host matrix", "verification/areas/runtime_platform/supported_host_matrix.md"],
        ["Golden manifest entries", "verification/areas/runtime_platform/golden/manifest.json"],
        ["Bare CPython import fixtures", "crates/sifr/tests/e2e/fail/bare_cpython_asyncio/queue/subprocess/concurrent_futures/multiprocessing/signal/contextlib/warnings/threading import fixture family"],
    ]
    M0_TRACEABILITY_MD.write_text(
        "\n\n".join(
            [
                "# Concurrency Runtime M0 Traceability",
                "Milestone: `milestone_concurrency_runtime_0`",
                md_table(["Requirement", "Evidence"], rows),
                "## M0 Closure Gate\n\nM0 is complete only after a post-M0 external review returns `PASS` and the result is recorded in the execution ledger. M1 remains blocked until M0a removes, hides, or diagnoses legacy CPython-shaped public surfaces.",
            ]
        )
        + "\n",
        encoding="utf-8",
    )


def main() -> None:
    if not CPYTHON_ROOT.exists():
        raise SystemExit(f"CPython checkout not found: {CPYTHON_ROOT}")
    files = [scan_file(item) for item in expand_paths()]
    commit = cpython_commit()
    write_inventory_json(files, commit)
    write_inventory_md(files, commit)
    write_evidence_md(files, commit)
    write_workload_md()
    write_m0_traceability_md()
    print(f"generated {len(files)} CPython evidence entries")


if __name__ == "__main__":
    main()
