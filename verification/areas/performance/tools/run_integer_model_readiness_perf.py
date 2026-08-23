#!/usr/bin/env python3
"""Run INT-8 integer-model readiness performance and allocation probes."""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[4]
PERF_FIXTURE = REPO_ROOT / "verification/areas/performance/fixtures/sifr_int_loop.sifr"
PROBE_DIR = REPO_ROOT / "target/integer_model_readiness_perf_probe"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--iterations", type=int, default=200_000)
    parser.add_argument("--repeats", type=int, default=5)
    parser.add_argument(
        "--max-slowdown",
        type=float,
        default=10.0,
        help="Maximum allowed SifrInt small-loop slowdown versus i64 baseline.",
    )
    return parser.parse_args()


def run(args: list[str], *, cwd: Path = REPO_ROOT) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        args,
        cwd=cwd,
        text=True,
        capture_output=True,
        check=False,
    )


def require_success(proc: subprocess.CompletedProcess[str], label: str) -> None:
    if proc.returncode == 0:
        return
    sys.stderr.write(f"{label} failed with exit code {proc.returncode}\n")
    if proc.stdout:
        sys.stderr.write("--- stdout ---\n")
        sys.stderr.write(proc.stdout)
    if proc.stderr:
        sys.stderr.write("--- stderr ---\n")
        sys.stderr.write(proc.stderr)
    raise SystemExit(1)


def write_probe_project() -> None:
    src_dir = PROBE_DIR / "src"
    src_dir.mkdir(parents=True, exist_ok=True)
    cargo_toml = f"""[package]
name = "integer_model_readiness_perf_probe"
version = "0.1.0"
edition = "2024"

[dependencies]
sifr_runtime = {{ path = "{(REPO_ROOT / 'crates/sifr_runtime').as_posix()}" }}

[workspace]
"""
    main_rs = r'''use sifr_runtime::SifrInt;
use std::alloc::{GlobalAlloc, Layout, System};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::hint::black_box;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Instant;

struct CountingAllocator;

static ALLOC_ACTIVE: AtomicBool = AtomicBool::new(false);
static ALLOC_COUNT: AtomicU64 = AtomicU64::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if ALLOC_ACTIVE.load(Ordering::Relaxed) {
            ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
        }
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

fn measured<T>(f: impl FnOnce() -> T) -> (T, u64) {
    ALLOC_COUNT.store(0, Ordering::Relaxed);
    ALLOC_ACTIVE.store(true, Ordering::Relaxed);
    let value = f();
    ALLOC_ACTIVE.store(false, Ordering::Relaxed);
    (value, ALLOC_COUNT.load(Ordering::Relaxed))
}

fn sifr_accumulate(iterations: i64) -> SifrInt {
    let mut total = SifrInt::from_i64(0);
    let mut i = SifrInt::from_i64(0);
    let limit = SifrInt::from_i64(black_box(iterations));
    let one = SifrInt::from_i64(1);
    while i < limit {
        total = black_box(total + black_box(&i));
        i = black_box(i + &one);
    }
    total
}

fn sifr_counter(iterations: i64) -> SifrInt {
    let mut i = SifrInt::from_i64(0);
    let limit = SifrInt::from_i64(black_box(iterations));
    let one = SifrInt::from_i64(1);
    while i < limit {
        i = black_box(i + &one);
    }
    i
}

fn i64_accumulate(iterations: i64) -> i64 {
    let mut total = 0_i64;
    let mut i = 0_i64;
    let limit = black_box(iterations);
    while i < limit {
        total = black_box(total.wrapping_add(black_box(i)));
        i = black_box(i + 1);
    }
    total
}

fn hash_loop(iterations: i64) -> u64 {
    let mut combined = 0_u64;
    let mut i = 0_i64;
    while i < iterations {
        let value = SifrInt::from_i64(i);
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        combined ^= hasher.finish();
        i += 1;
    }
    combined
}

fn format_loop(iterations: i64) -> usize {
    let mut total_len = 0_usize;
    let mut i = 0_i64;
    while i < iterations {
        let value = SifrInt::from_i64(i);
        total_len = total_len.wrapping_add(value.to_string().len());
        i += 1;
    }
    total_len
}

fn best_ns<T>(repeats: usize, f: impl Fn() -> T) -> (T, u128) {
    let mut best: Option<(T, u128)> = None;
    for _ in 0..repeats {
        let started = Instant::now();
        let value = f();
        let elapsed = started.elapsed().as_nanos();
        if best.as_ref().is_none_or(|(_, best_elapsed)| elapsed < *best_elapsed) {
            best = Some((value, elapsed));
        }
    }
    best.expect("at least one repeat is required")
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let iterations = args.get(1).and_then(|value| value.parse::<i64>().ok()).unwrap_or(200_000);
    let repeats = args.get(2).and_then(|value| value.parse::<usize>().ok()).unwrap_or(5);
    assert!(iterations > 0);
    assert!(repeats > 0);

    let expected = iterations * (iterations - 1) / 2;
    let (sifr_total, sifr_loop_ns) = best_ns(repeats, || black_box(sifr_accumulate(iterations)));
    let (i64_total, i64_loop_ns) = best_ns(repeats, || black_box(i64_accumulate(iterations)));
    let (sifr_alloc_total, sifr_loop_allocs) = measured(|| sifr_accumulate(iterations));
    let (counter_total, counter_allocs) = measured(|| sifr_counter(iterations));
    let (hash_total, hash_allocs) = measured(|| hash_loop(iterations));
    let (format_total, format_allocs) = measured(|| format_loop(1024));

    assert_eq!(sifr_total, SifrInt::from_i64(expected));
    assert_eq!(sifr_alloc_total, SifrInt::from_i64(expected));
    assert_eq!(i64_total, expected);
    assert_eq!(counter_total, SifrInt::from_i64(iterations));

    println!("iterations={iterations}");
    println!("repeats={repeats}");
    println!("sifr_loop_ns={sifr_loop_ns}");
    println!("i64_loop_ns={i64_loop_ns}");
    println!("sifr_loop_allocs={sifr_loop_allocs}");
    println!("counter_allocs={counter_allocs}");
    println!("hash_allocs={hash_allocs}");
    println!("hash_checksum={hash_total}");
    println!("format_sample_allocs={format_allocs}");
    println!("format_checksum={format_total}");
}
'''
    (PROBE_DIR / "Cargo.toml").write_text(cargo_toml, encoding="utf-8")
    (src_dir / "main.rs").write_text(main_rs, encoding="utf-8")


def parse_probe_output(stdout: str) -> dict[str, str]:
    values: dict[str, str] = {}
    for line in stdout.splitlines():
        if match := re.fullmatch(r"([a-z0-9_]+)=(.+)", line.strip()):
            values[match.group(1)] = match.group(2)
    return values


def main() -> None:
    args = parse_args()
    if args.iterations <= 0:
        raise SystemExit("--iterations must be positive")
    if args.repeats <= 0:
        raise SystemExit("--repeats must be positive")
    if args.max_slowdown <= 0:
        raise SystemExit("--max-slowdown must be positive")

    fixture = run(["cargo", "run", "-q", "-p", "sifr", "--", "run", str(PERF_FIXTURE)])
    require_success(fixture, "Sifr perf fixture")
    if "sifr_int_loop: passed" not in fixture.stdout:
        sys.stderr.write("Sifr perf fixture did not report success\n")
        raise SystemExit(1)

    write_probe_project()
    probe = run(
        [
            "cargo",
            "run",
            "--release",
            "--quiet",
            "--manifest-path",
            str(PROBE_DIR / "Cargo.toml"),
            "--",
            str(args.iterations),
            str(args.repeats),
        ]
    )
    require_success(probe, "integer runtime perf probe")
    values = parse_probe_output(probe.stdout)
    required = {
        "sifr_loop_ns",
        "i64_loop_ns",
        "sifr_loop_allocs",
        "counter_allocs",
        "hash_allocs",
    }
    missing = sorted(required.difference(values))
    if missing:
        raise SystemExit(f"perf probe missing keys: {', '.join(missing)}")

    sifr_ns = int(values["sifr_loop_ns"])
    i64_ns = max(1, int(values["i64_loop_ns"]))
    slowdown = sifr_ns / i64_ns
    failures: list[str] = []
    for key in ("sifr_loop_allocs", "counter_allocs", "hash_allocs"):
        if int(values[key]) != 0:
            failures.append(f"{key} expected 0, got {values[key]}")
    if slowdown > args.max_slowdown:
        failures.append(
            f"SifrInt small-loop slowdown {slowdown:.2f}x exceeds {args.max_slowdown:.2f}x"
        )

    print("Integer model readiness performance probe")
    for key in sorted(values):
        print(f"  {key}={values[key]}")
    print(f"  sifr_vs_i64_slowdown={slowdown:.2f}x")
    print(f"  max_slowdown={args.max_slowdown:.2f}x")

    if failures:
        for failure in failures:
            print(f"FAIL: {failure}", file=sys.stderr)
        raise SystemExit(1)

    print("Integer model readiness performance probe: PASS")


if __name__ == "__main__":
    main()
