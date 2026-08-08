"""Summarize validation profile runtime, cache, and resource metrics."""

from __future__ import annotations

import argparse
import json
import re
import tempfile
from pathlib import Path
from typing import Any

from .paths import REPO_ROOT
from .profiles import legacy_facade, load_profile

ARTIFACT_CACHE_ROOT = Path(tempfile.gettempdir()) / "sifr_generated_artifact_cache"
BSD_TIME_COMBINED_RE = re.compile(r"^\s*([0-9.]+)\s+real\s+([0-9.]+)\s+user\s+([0-9.]+)\s+sys$")
TIME_REAL_RE = re.compile(r"^\s*([0-9.]+)\s+real$")
TIME_USER_RE = re.compile(r"^\s*([0-9.]+)\s+user$")
TIME_SYS_RE = re.compile(r"^\s*([0-9.]+)\s+sys$")
TIME_MAX_RSS_RE = re.compile(r"^\s*(\d+)\s+maximum resident set size$")
TIME_SWAPS_RE = re.compile(r"^\s*(\d+)\s+swaps$")
VALIDATION_SUITE_RE = re.compile(r"^\s*-\s+([a-z0-9_-]+):\s+(\d+)\s+rows,\s+(\d+)ms$")
VALIDATION_TOTAL_RE = re.compile(r"^\[validation-suite\]\s+total_rows=(\d+)\s+total_ms=(\d+)$")
E2E_TIMING_RE = re.compile(
    r"^\[sifr-e2e\]\s+timing:\s+compile=(\d+)ms\s+plan=(\d+)ms\s+build=(\d+)ms\s+build-sum=(\d+)ms\s+run=(\d+)ms\s+cache_hits=(\d+)/(\d+)$"
)
E2E_GROUP_RE = re.compile(
    r"^\[sifr-e2e\]\s+group_stats:\s+groups=(\d+)\s+largest_group_fixtures=(\d+)\s+median_group_fixtures=(\d+)$"
)
ARTIFACT_CACHE_RE = re.compile(
    r"^\[sifr-artifact-cache\]\s+namespace=([a-z0-9_-]+)\s+key=([0-9a-f]+)\s+cache_hit=(true|false)\s+workspace=([^\s]+)(?:\s+miss_reason=([a-z0-9_-]+))?$"
)
HARDENING_OK_RE = re.compile(
    r"^verification ok:\s+variants=(\d+),\s+failures=(\d+),\s+blocking_failures=(\d+),"
    r"\s+non_blocking_failures=(\d+)(?:,\s+skipped=(\d+))?$"
)
CACHE_DIR_RE = re.compile(r"^\s*cache_dir=(.+)$")
LANE_STEP_RE = re.compile(r"^\[sifr-lane-step\]\s+name=([A-Za-z0-9_.-]+)\s+elapsed_ms=(\d+)\s+status=(pass|fail)$")
LANE_STEP_BUDGET_RE = re.compile(
    r"^\[sifr-lane-step-budget\]\s+name=([A-Za-z0-9_.-]+)\s+elapsed_ms=(\d+)\s+"
    r"budget_ms=(\d+)\s+enforcement=(advisory|blocking)\s+status=(pass|fail)$"
)
LANE_STEP_CACHE_RE = re.compile(
    r"^\[sifr-lane-step-cache\]\s+name=([A-Za-z0-9_.-]+)\s+state=(warm|cold)\s+"
    r"reason=([a-z0-9-]+)\s+fingerprint=([0-9a-f]+)$"
)
CASE_TIMING_RE = re.compile(
    r"^\[sifr-case-timing\]\s+bucket=([A-Za-z0-9_-]+)\s+case=([A-Za-z0-9_.:/+-]+)"
    r"\s+elapsed_ms=(\d+)\s+status=(pass|fail|skip)$"
)
WARM_CACHE_HIT_TARGET = 0.90
GROUP_SKEW_ADVISORY_RATIO = 4.0
GROUP_SKEW_ABSOLUTE_DELTA = 8
DEFAULT_LANE_RSS_ADVISORY_BYTES = 6 * 1024 * 1024 * 1024


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)

    summarize = subparsers.add_parser("summarize")
    summarize.add_argument("--profile", required=True)
    summarize.add_argument("--log", required=True)
    summarize.add_argument("--time-file", required=True)
    summarize.add_argument(
        "--json-out",
        default="",
        help="Optional output path for machine-readable JSON summary.",
    )
    return parser.parse_args(argv)


def load_json(path: Path) -> dict[str, Any]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise SystemExit(f"invalid JSON object: {path}")
    return payload


def resolve_profile(profile: str) -> tuple[str, dict[str, Any]]:
    payload = load_profile(profile)
    legacy = legacy_facade(payload)
    budgets = payload["budgets"]
    lane = {
        "name": payload["name"],
        "description": payload.get("description", ""),
        "warm_wall_time_target_minutes": budgets["warm_wall_time_minutes"],
        "cold_wall_time_target_minutes": budgets["cold_wall_time_minutes"],
        **legacy,
        "step_budgets": payload.get("step_budgets", {}),
    }
    return str(payload["name"]), lane


def parse_time_file(path: Path) -> dict[str, int | float]:
    metrics: dict[str, int | float] = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        if match := BSD_TIME_COMBINED_RE.match(line):
            metrics["real_seconds"] = float(match.group(1))
            metrics["user_seconds"] = float(match.group(2))
            metrics["sys_seconds"] = float(match.group(3))
        elif match := TIME_REAL_RE.match(line):
            metrics["real_seconds"] = float(match.group(1))
        elif match := TIME_USER_RE.match(line):
            metrics["user_seconds"] = float(match.group(1))
        elif match := TIME_SYS_RE.match(line):
            metrics["sys_seconds"] = float(match.group(1))
        elif match := TIME_MAX_RSS_RE.match(line):
            metrics["max_rss_bytes"] = int(match.group(1))
        elif match := TIME_SWAPS_RE.match(line):
            metrics["swaps"] = int(match.group(1))
    return metrics


def format_bytes(value: int) -> str:
    if value <= 0:
        return "0B"
    units = ["B", "KiB", "MiB", "GiB", "TiB"]
    scaled = float(value)
    for unit in units:
        if scaled < 1024.0 or unit == units[-1]:
            return f"{scaled:.1f}{unit}" if unit != "B" else f"{int(scaled)}B"
        scaled /= 1024.0
    raise AssertionError("unreachable")


def lane_workers(lane: dict[str, Any]) -> dict[str, int]:
    e2e = lane.get("e2e")
    if not isinstance(e2e, dict):
        return {"sifr_jobs": 0, "rust_jobs": 0, "run_jobs": 0, "cargo_build_jobs": 0}
    return {
        "sifr_jobs": int(e2e.get("sifr_jobs", 0) or 0),
        "rust_jobs": int(e2e.get("rust_jobs", 0) or 0),
        "run_jobs": int(e2e.get("run_jobs", 0) or 0),
        "cargo_build_jobs": int(e2e.get("cargo_build_jobs", 0) or 0),
    }


def build_advisories(
    profile: str,
    warm_target_minutes: int,
    real_seconds: float,
    time_metrics: dict[str, int | float],
    e2e_metrics: dict[str, int] | None,
) -> list[str]:
    advisories: list[str] = []

    if warm_target_minutes > 0 and real_seconds > warm_target_minutes * 60:
        advisories.append("warm wall-time budget exceeded")

    swaps = int(time_metrics.get("swaps", 0))
    if swaps > 0:
        advisories.append("swap activity observed; lower worker counts or rebalance groups")

    max_rss_bytes = int(time_metrics.get("max_rss_bytes", 0))
    if profile in {"create-pr", "merge"} and max_rss_bytes > DEFAULT_LANE_RSS_ADVISORY_BYTES:
        advisories.append("peak RSS exceeded low-single-digit GiB guidance for the default lane")

    if isinstance(e2e_metrics, dict):
        group_count = int(e2e_metrics.get("group_count", 0))
        cache_hits = int(e2e_metrics.get("cache_hits", 0))
        if group_count > 0:
            cache_hit_rate = cache_hits / group_count
            if cache_hits > 0 and cache_hit_rate < WARM_CACHE_HIT_TARGET:
                advisories.append(
                    "warm-cache hit rate below advisory target; unchanged reruns should trend toward >=90%"
                )
        largest_group = int(e2e_metrics.get("largest_group_fixtures", 0))
        median_group = int(e2e_metrics.get("median_group_fixtures", 0))
        normalized_median = max(median_group, 1)
        skew_ratio = largest_group / normalized_median
        if largest_group - median_group >= GROUP_SKEW_ABSOLUTE_DELTA and skew_ratio >= GROUP_SKEW_ADVISORY_RATIO:
            advisories.append("group skew is high; investigate batching balance or fixture clustering")

    return advisories


def directory_stats(path: Path) -> dict[str, int | str]:
    if not path.exists():
        return {"path": str(path), "bytes": 0, "files": 0}
    total_bytes = 0
    file_count = 0
    for child in path.rglob("*"):
        if child.is_file():
            file_count += 1
            total_bytes += child.stat().st_size
    return {"path": str(path), "bytes": total_bytes, "files": file_count}


def parse_log(path: Path) -> dict[str, Any]:
    suite_filters: list[dict[str, int | str]] = []
    lane_steps: list[dict[str, int | str]] = []
    lane_step_budgets: dict[str, dict[str, int | str]] = {}
    lane_step_cache: dict[str, dict[str, str]] = {}
    case_timings: list[dict[str, int | str]] = []
    artifact_cache: dict[str, dict[str, Any]] = {}
    hardening_summary: dict[str, int] | None = None
    e2e_metrics: dict[str, int] | None = None
    cache_dir: str | None = None

    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line:
            continue
        if match := LANE_STEP_RE.match(line):
            lane_steps.append(
                {
                    "name": match.group(1),
                    "elapsed_ms": int(match.group(2)),
                    "status": match.group(3),
                }
            )
            continue
        if match := LANE_STEP_BUDGET_RE.match(line):
            lane_step_budgets[match.group(1)] = {
                "elapsed_ms": int(match.group(2)),
                "budget_ms": int(match.group(3)),
                "enforcement": match.group(4),
                "status": match.group(5),
            }
            continue
        if match := LANE_STEP_CACHE_RE.match(line):
            lane_step_cache[match.group(1)] = {
                "state": match.group(2),
                "reason": match.group(3),
                "fingerprint": match.group(4),
            }
            continue
        if match := CASE_TIMING_RE.match(line):
            case_timings.append(
                {
                    "bucket": match.group(1),
                    "case": match.group(2),
                    "elapsed_ms": int(match.group(3)),
                    "status": match.group(4),
                }
            )
            continue
        if match := VALIDATION_SUITE_RE.match(line):
            suite_filters.append(
                {
                    "suite": match.group(1),
                    "rows": int(match.group(2)),
                    "elapsed_ms": int(match.group(3)),
                }
            )
            continue
        if match := VALIDATION_TOTAL_RE.match(line):
            suite_filters.append(
                {
                    "suite": "__total__",
                    "rows": int(match.group(1)),
                    "elapsed_ms": int(match.group(2)),
                }
            )
            continue
        if match := E2E_TIMING_RE.match(line):
            e2e_metrics = {
                "compile_ms": int(match.group(1)),
                "plan_ms": int(match.group(2)),
                "build_ms": int(match.group(3)),
                "build_sum_ms": int(match.group(4)),
                "run_ms": int(match.group(5)),
                "cache_hits": int(match.group(6)),
                "group_count": int(match.group(7)),
            }
            continue
        if match := E2E_GROUP_RE.match(line):
            e2e_metrics = dict(e2e_metrics or {})
            e2e_metrics.update(
                {
                    "groups": int(match.group(1)),
                    "largest_group_fixtures": int(match.group(2)),
                    "median_group_fixtures": int(match.group(3)),
                }
            )
            continue
        if match := ARTIFACT_CACHE_RE.match(line):
            namespace = match.group(1)
            entry = artifact_cache.setdefault(namespace, {"hits": 0, "misses": 0, "miss_reasons": {}})
            if match.group(3) == "true":
                entry["hits"] += 1
            else:
                entry["misses"] += 1
                miss_reason = match.group(5)
                if miss_reason:
                    miss_reasons = entry["miss_reasons"]
                    assert isinstance(miss_reasons, dict)
                    miss_reasons[miss_reason] = int(miss_reasons.get(miss_reason, 0)) + 1
            continue
        if match := HARDENING_OK_RE.match(line):
            entry = {
                "variants": int(match.group(1)),
                "failures": int(match.group(2)),
                "blocking_failures": int(match.group(3)),
                "non_blocking_failures": int(match.group(4)),
                "skipped": int(match.group(5) or 0),
            }
            if hardening_summary is None:
                hardening_summary = entry
            else:
                for key, value in entry.items():
                    hardening_summary[key] += value
            continue
        if match := CACHE_DIR_RE.match(line):
            cache_dir = match.group(1).strip()

    return {
        "lane_steps": lane_steps,
        "lane_step_budgets": lane_step_budgets,
        "lane_step_cache": lane_step_cache,
        "case_timings": case_timings,
        "suite_filters": suite_filters,
        "artifact_cache": artifact_cache,
        "hardening_summary": hardening_summary,
        "e2e_metrics": e2e_metrics,
        "e2e_cache_dir": cache_dir,
    }


def summarize(args: argparse.Namespace) -> int:
    profile, lane = resolve_profile(args.profile)
    log_path = Path(args.log).resolve()
    time_path = Path(args.time_file).resolve()

    parsed_log = parse_log(log_path)
    time_metrics = parse_time_file(time_path)
    e2e_cache_dir = parsed_log["e2e_cache_dir"]
    if isinstance(e2e_cache_dir, str):
        e2e_cache_path = Path(e2e_cache_dir)
        if not e2e_cache_path.is_absolute():
            e2e_cache_path = REPO_ROOT / e2e_cache_path
        e2e_cache_stats = directory_stats(e2e_cache_path)
    else:
        e2e_cache_stats = None
    artifact_cache_stats = directory_stats(ARTIFACT_CACHE_ROOT)

    real_seconds = float(time_metrics.get("real_seconds", 0.0))
    user_seconds = float(time_metrics.get("user_seconds", 0.0))
    sys_seconds = float(time_metrics.get("sys_seconds", 0.0))
    cpu_seconds = user_seconds + sys_seconds
    warm_target_minutes = int(lane.get("warm_wall_time_target_minutes", 0))
    within_budget = True
    if warm_target_minutes > 0 and real_seconds > 0:
        within_budget = real_seconds <= warm_target_minutes * 60
    workers = lane_workers(lane)
    e2e_metrics = parsed_log["e2e_metrics"] if isinstance(parsed_log["e2e_metrics"], dict) else None
    cache_hit_rate = None
    rebuild_groups = None
    group_skew_ratio = None
    if isinstance(e2e_metrics, dict):
        group_count = int(e2e_metrics.get("group_count", 0))
        cache_hits = int(e2e_metrics.get("cache_hits", 0))
        if group_count > 0:
            cache_hit_rate = cache_hits / group_count
            rebuild_groups = group_count - cache_hits
        largest_group = int(e2e_metrics.get("largest_group_fixtures", 0))
        median_group = int(e2e_metrics.get("median_group_fixtures", 0))
        group_skew_ratio = largest_group / max(median_group, 1) if largest_group > 0 else 0.0
    advisories = build_advisories(profile, warm_target_minutes, real_seconds, time_metrics, e2e_metrics)
    lane_steps = list(parsed_log["lane_steps"])
    lane_step_budgets = parsed_log["lane_step_budgets"]
    lane_step_cache = parsed_log["lane_step_cache"]
    if isinstance(lane_step_budgets, dict):
        for step in lane_steps:
            budget = lane_step_budgets.get(str(step["name"]))
            if isinstance(budget, dict):
                step["budget_ms"] = int(budget["budget_ms"])
                step["budget_enforcement"] = str(budget["enforcement"])
                step["budget_status"] = str(budget["status"])
            cache = lane_step_cache.get(str(step["name"]))
            if isinstance(cache, dict):
                step["cache_state"] = str(cache["state"])
                step["cache_reason"] = str(cache["reason"])
                step["cache_fingerprint"] = str(cache["fingerprint"])
    case_timings = list(parsed_log["case_timings"])
    slowest_cases = sorted(
        case_timings,
        key=lambda timing: int(timing["elapsed_ms"]),
        reverse=True,
    )[:10]

    payload = {
        "profile": profile,
        "requested_profile": args.profile,
        "lane": lane.get("name", profile),
        "description": lane.get("description", ""),
        "budget": {
            "warm_wall_time_target_minutes": warm_target_minutes,
            "cold_wall_time_target_minutes": int(lane.get("cold_wall_time_target_minutes", 0)),
            "within_warm_budget": within_budget,
        },
        "policy": {
            "thermal": lane.get("thermal_policy", ""),
            "memory": lane.get("memory_policy", ""),
        },
        "workers": workers,
        "time": time_metrics,
        "cpu_seconds": cpu_seconds,
        "e2e": e2e_metrics,
        "observations": {
            "cache_hit_rate": cache_hit_rate,
            "rebuild_groups": rebuild_groups,
            "group_skew_ratio": group_skew_ratio,
            "slowest_cases": slowest_cases,
        },
        "advisories": advisories,
        "lane_steps": lane_steps,
        "lane_step_budgets": lane_step_budgets,
        "lane_step_cache": lane_step_cache,
        "case_timings": case_timings,
        "suite_filters": parsed_log["suite_filters"],
        "artifact_cache": parsed_log["artifact_cache"],
        "hardening_summary": parsed_log["hardening_summary"],
        "cache_footprint": {
            "e2e": e2e_cache_stats,
            "generated_artifacts": artifact_cache_stats,
        },
        "log_path": str(log_path),
        "time_file": str(time_path),
    }

    if args.json_out:
        json_path = Path(args.json_out).resolve()
        json_path.parent.mkdir(parents=True, exist_ok=True)
        json_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    print("Validation lane report")
    print(f"  profile={profile}")
    print(
        "  wall_time="
        f"{real_seconds:.2f}s "
        f"cpu={cpu_seconds:.2f}s "
        f"(warm_target<={warm_target_minutes}m budget_ok={'yes' if within_budget else 'no'})"
    )
    if "max_rss_bytes" in time_metrics:
        print(
            "  resources="
            f"max_rss={format_bytes(int(time_metrics['max_rss_bytes']))} "
            f"swaps={int(time_metrics.get('swaps', 0))}"
        )
    if isinstance(e2e_metrics, dict):
        e2e = e2e_metrics
        rebuilt_groups_suffix = ""
        if rebuild_groups is not None:
            rebuilt_groups_suffix = f" rebuilt_groups={rebuild_groups}"
        skew_suffix = ""
        if group_skew_ratio is not None:
            skew_suffix = f" skew={group_skew_ratio:.1f}x"
        hit_rate_suffix = ""
        if cache_hit_rate is not None:
            hit_rate_suffix = f" hit_rate={cache_hit_rate * 100:.0f}%"
        print(
            "  e2e="
            f"compile={e2e.get('compile_ms', 0)}ms "
            f"plan={e2e.get('plan_ms', 0)}ms "
            f"build={e2e.get('build_ms', 0)}ms "
            f"run={e2e.get('run_ms', 0)}ms "
            f"cache_hits={e2e.get('cache_hits', 0)}/{e2e.get('group_count', 0)} "
            f"largest_group={e2e.get('largest_group_fixtures', 0)} "
            f"median_group={e2e.get('median_group_fixtures', 0)}"
            f"{hit_rate_suffix}{rebuilt_groups_suffix}{skew_suffix}"
        )
    print(
        "  workers="
        f"sifr={workers['sifr_jobs']} "
        f"rust={workers['rust_jobs']} "
        f"run={workers['run_jobs']} "
        f"cargo_build={workers['cargo_build_jobs']}"
    )
    if lane_steps:
        slowest_step = max(lane_steps, key=lambda step: int(step["elapsed_ms"]))
        print(
            f"  slowest_step={slowest_step['name']} {int(slowest_step['elapsed_ms'])}ms status={slowest_step['status']}"
        )
        slowest_steps = sorted(
            lane_steps,
            key=lambda step: int(step["elapsed_ms"]),
            reverse=True,
        )[:5]
        step_summaries = [
            f"{step['name']}={int(step['elapsed_ms'])}ms"
            + (f"/budget={int(step['budget_ms'])}ms/{step['budget_status']}" if "budget_ms" in step else "")
            for step in slowest_steps
        ]
        print("  slowest_steps=" + " ".join(step_summaries))
    if case_timings:
        by_bucket: dict[str, dict[str, int | str]] = {}
        for timing in case_timings:
            bucket = str(timing["bucket"])
            current = by_bucket.get(bucket)
            if current is None or int(timing["elapsed_ms"]) > int(current["elapsed_ms"]):
                by_bucket[bucket] = timing
        slowest_cases = [
            f"{bucket}:{timing['case']}={int(timing['elapsed_ms'])}ms" for bucket, timing in sorted(by_bucket.items())
        ]
        print("  slowest_cases=" + " ".join(slowest_cases))
        top_cases = [
            f"{timing['bucket']}:{timing['case']}={int(timing['elapsed_ms'])}ms"
            for timing in sorted(
                case_timings,
                key=lambda timing: int(timing["elapsed_ms"]),
                reverse=True,
            )[:10]
        ]
        print("  top_slowest_cases=" + " ".join(top_cases))
    if parsed_log["artifact_cache"]:
        summaries = []
        for namespace, stats in sorted(parsed_log["artifact_cache"].items()):
            reason_suffix = ""
            miss_reasons = stats.get("miss_reasons")
            if isinstance(miss_reasons, dict) and miss_reasons:
                reasons = ",".join(f"{reason}={count}" for reason, count in sorted(miss_reasons.items()))
                reason_suffix = f",miss_reasons={reasons}"
            summaries.append(f"{namespace}:hits={stats['hits']},misses={stats['misses']}{reason_suffix}")
        print("  generated_artifact_cache=" + " ".join(summaries))
    if e2e_cache_stats is not None:
        print(
            "  cache_footprint="
            f"e2e={format_bytes(int(e2e_cache_stats['bytes']))}/{e2e_cache_stats['files']}files "
            f"generated={format_bytes(int(artifact_cache_stats['bytes']))}/{artifact_cache_stats['files']}files"
        )
    if isinstance(parsed_log["hardening_summary"], dict):
        hardening = parsed_log["hardening_summary"]
        print(
            "  hardening="
            f"variants={hardening['variants']} "
            f"failures={hardening['failures']} "
            f"blocking_failures={hardening['blocking_failures']}"
        )
    if advisories:
        print("  advisories=" + "; ".join(advisories))
    else:
        print("  advisories=none")
    if args.json_out:
        json_path = Path(args.json_out).resolve()
        try:
            display_json_path = json_path.relative_to(REPO_ROOT)
        except ValueError:
            display_json_path = json_path
        print(f"  json={display_json_path}")
    return 0


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    if args.command == "summarize":
        return summarize(args)
    raise SystemExit(f"unsupported command: {args.command}")


if __name__ == "__main__":
    raise SystemExit(main())
