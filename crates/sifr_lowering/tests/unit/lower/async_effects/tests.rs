
use super::{AsyncSuspensionSummary, collect_async_suspension_summaries};
use sifr_python_parser::parse_module;

fn summaries(source: &str) -> std::collections::HashMap<String, AsyncSuspensionSummary> {
    let parsed = parse_module(source).expect("parse failed");
    collect_async_suspension_summaries(&parsed.into_suite())
}

#[test]
fn marks_direct_timer_wait_as_suspending() {
    let summaries = summaries(
        r"
async def main() -> None:
    await task.sleep(0.0)
",
    );
    assert_eq!(
        summaries.get("main"),
        Some(&AsyncSuspensionSummary::Suspends)
    );
}

#[test]
fn propagates_transitive_same_task_await_summaries() {
    let summaries = summaries(
        r"
async def leaf() -> int:
    await task.sleep(0.0)
    return 1

async def wrapper() -> int:
    return await leaf()
",
    );
    assert_eq!(
        summaries.get("wrapper"),
        Some(&AsyncSuspensionSummary::Suspends)
    );
}

#[test]
fn keeps_fake_async_wrapper_chain_no_suspend() {
    let summaries = summaries(
        r"
async def leaf() -> int:
    return 1

async def wrapper() -> int:
    return await leaf()
",
    );
    assert_eq!(
        summaries.get("leaf"),
        Some(&AsyncSuspensionSummary::NoSuspend)
    );
    assert_eq!(
        summaries.get("wrapper"),
        Some(&AsyncSuspensionSummary::NoSuspend)
    );
}

#[test]
fn marks_async_generator_yield_as_suspending() {
    let summaries = summaries(
        r"
async def values() -> AsyncGenerator[int, GeneratorCloseError]:
    yield 1
",
    );
    assert_eq!(
        summaries.get("values"),
        Some(&AsyncSuspensionSummary::Suspends)
    );
}

#[test]
fn marks_bodyless_python_coroutine_stub_as_suspending() {
    let summaries = summaries(
        r"
@python.coroutine(pkg.compute)
async def compute(value: int) -> int: ...
",
    );
    assert_eq!(
        summaries.get("compute"),
        Some(&AsyncSuspensionSummary::Suspends)
    );
}
