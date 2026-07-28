"""Manifest and harness policy for retained callback subscription evidence."""

from __future__ import annotations

import shutil
import tempfile
from pathlib import Path
from typing import Any, Callable

from _scenario_async_reqwest import (
    _require_dependency,
    _require_path_dependency,
    _require_trust,
)

ScenarioValidator = Callable[
    [list[str], str, Path, dict[str, Any]],
    int,
]


def validate_callback_subscription_scenario(
    failures: list[str],
    fixture_id: str,
    raw_path: str,
    rust: dict[str, Any],
    dependencies: dict[str, Any],
    trust: dict[str, Any],
) -> None:
    if rust.get("bridges") != ["src/bridges"]:
        failures.append(
            f"{fixture_id}: {raw_path}/sifr.toml must declare "
            '[rust] bridges = ["src/bridges"]'
        )
    _require_path_dependency(
        failures,
        fixture_id,
        raw_path,
        dependencies,
        "sifr_runtime",
        "../../../../../../../crates/sifr_runtime",
    )
    for name, version, features, default_features in (
        ("futures", "=0.3.33", None, None),
        ("notify", "=8.2.0", None, None),
        ("redis", "=1.4.1", ["tokio-comp"], False),
        (
            "tokio",
            "=1.52.3",
            ["io-util", "net", "rt", "sync", "time"],
            None,
        ),
        ("tokio-tungstenite", "=0.30.0", None, False),
    ):
        if features is None:
            entry = dependencies.get(name)
            if not isinstance(entry, dict) or entry.get("version") != version:
                failures.append(
                    f"{fixture_id}: {raw_path}/Cargo.toml dependency {name} "
                    f"must pin version {version}"
                )
            if (
                default_features is not None
                and isinstance(entry, dict)
                and entry.get("default-features") is not default_features
            ):
                failures.append(
                    f"{fixture_id}: {raw_path}/Cargo.toml dependency {name} "
                    f"must set default-features = {str(default_features).lower()}"
                )
            continue
        _require_dependency(
            failures,
            fixture_id,
            raw_path,
            dependencies,
            name,
            version,
            features,
            default_features=default_features,
        )
    _require_trust(
        failures,
        fixture_id,
        raw_path,
        trust,
        "rust-no-panic",
        [
            "bridge.events.aclose",
            "bridge.events.close_observation",
            "bridge.events.verify",
        ],
    )


def run_callback_subscription_self_test(
    area_root: Path,
    validate_scenarios: ScenarioValidator,
) -> tuple[int, str | None]:
    source = area_root / "fixtures/callback_subscription_ecosystem"
    raw_examples = {
        "subscription_lifecycle_runtime": "examples/subscription_lifecycle_runtime"
    }
    cases = 0
    with tempfile.TemporaryDirectory(
        prefix="sifr-rust-callback-scenario-self-test-"
    ) as raw_temp:
        fixture_dir = Path(raw_temp) / "callback_subscription_ecosystem"
        shutil.copytree(
            source,
            fixture_dir,
            ignore=shutil.ignore_patterns("target"),
        )
        baseline_failures: list[str] = []
        validate_scenarios(
            baseline_failures,
            "callback_subscription_ecosystem",
            fixture_dir,
            raw_examples,
        )
        if baseline_failures:
            return cases, f"callback subscription baseline failed: {baseline_failures}"
        cases += 1

        mutation_cases = (
            (
                "tokio-tungstenite pin drift",
                "examples/subscription_lifecycle_runtime/Cargo.toml",
                'version = "=0.30.0"',
                'version = "0.30.0"',
                "must pin version =0.30.0",
            ),
            (
                "Redis feature drift",
                "examples/subscription_lifecycle_runtime/Cargo.toml",
                'features = ["tokio-comp"]',
                "features = []",
                "must declare features",
            ),
            (
                "callback no-panic trust drift",
                "examples/subscription_lifecycle_runtime/sifr.toml",
                '  "bridge.events.verify",',
                '  "bridge.events.unverified",',
                "trust.rust-no-panic",
            ),
            (
                "bounded callback policy drift",
                "examples/subscription_lifecycle_runtime/src/main.sifr",
                "backpressure=bounded(2)",
                "backpressure=unbounded",
                "missing scenario token 'backpressure=bounded(2)'",
            ),
            (
                "callback policy consumption drift",
                "examples/subscription_lifecycle_runtime/src/bridges/events.rs",
                "CallbackQueue::from_policy(callback.policy())",
                "CallbackQueue::from_policy(ThreadsafeCallbackPolicy::default())",
                "missing scenario token "
                "'CallbackQueue::from_policy(callback.policy())'",
            ),
            (
                "foreign-thread callback drift",
                "examples/subscription_lifecycle_runtime/src/bridges/events.rs",
                "std::thread::current().id() != owner_thread",
                "std::thread::current().id() == owner_thread",
                "missing scenario token "
                "'std::thread::current().id() != owner_thread'",
            ),
            (
                "subscription RAII drift",
                "examples/subscription_lifecycle_runtime/src/bridges/events.rs",
                "impl Drop for Subscription",
                "impl SubscriptionDrop",
                "missing scenario token 'impl Drop for Subscription'",
            ),
        )
        for name, relative_path, before, after, expected in mutation_cases:
            path = fixture_dir / relative_path
            original = path.read_text(encoding="utf-8")
            if before not in original:
                return cases, f"{name} self-test setup token is missing"
            path.write_text(original.replace(before, after, 1), encoding="utf-8")
            failures: list[str] = []
            validate_scenarios(
                failures,
                "callback_subscription_ecosystem",
                fixture_dir,
                raw_examples,
            )
            path.write_text(original, encoding="utf-8")
            if not any(expected in failure for failure in failures):
                return cases, f"{name} did not report {expected!r}: {failures}"
            cases += 1

    return cases, None
