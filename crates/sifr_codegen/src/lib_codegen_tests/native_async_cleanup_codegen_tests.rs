use super::generate_rust_from_source;

#[test]
fn native_async_context_emits_abnormal_cleanup_envelope() {
    let rust_code = generate_rust_from_source(
        "class ResourceError(Error):\n    message: str\n\nclass Resource:\n    async def __aenter__(self) -> Result[int, ResourceError]:\n        return 1\n\n    async def __aexit__(self, cause: AsyncExitCause) -> Result[None, ResourceError]:\n        return None\n\nasync def use_resource() -> Result[int, ResourceError]:\n    async with Resource() as value:\n        if value == 1:\n            raise ResourceError(\"body\")\n        return value\n    return 0\n",
    );

    assert!(rust_code.contains("CancellationScopeLease::claim"));
    assert!(rust_code.contains("catch_unwind_future"));
    assert!(rust_code.contains("AsyncExitCause::Return"));
    assert!(rust_code.contains("AsyncExitCause::OrdinaryError"));
    assert!(rust_code.contains("AsyncExitCause::Cancellation"));
    assert!(rust_code.contains("AsyncExitCause::RuntimeFault"));
    assert!(rust_code.contains("record_async_cleanup_failed"));
    assert!(rust_code.contains("record_async_cleanup_timed_out"));
    assert!(rust_code.contains("tokio::time::timeout"));
    syn::parse_file(&rust_code).expect("native async context Rust should parse");
}

#[test]
fn native_async_context_classifies_timeout_before_error_erasure() {
    let rust_code = generate_rust_from_source(
        "class Resource:\n    async def __aenter__(self) -> Result[None, TimeoutError]:\n        return None\n\n    async def __aexit__(self, cause: AsyncExitCause) -> Result[None, TimeoutError]:\n        return None\n\nasync def timed() -> Result[None, TimeoutError]:\n    async with Resource():\n        async with task.timeout(0.0):\n            await task.sleep(1.0)\n    return None\n",
    );

    assert!(rust_code.contains("AsyncExitCause::Timeout"));
    assert!(!rust_code.contains("AsyncExitCause::OrdinaryError(format!(\"{}\", body_error))"));
    syn::parse_file(&rust_code).expect("timeout-classified async context Rust should parse");
}

#[test]
fn closable_async_for_emits_exact_once_abnormal_cleanup_envelope() {
    let rust_code = generate_rust_from_source(
        "class StreamError(Error):\n    message: str\n\nclass Stream:\n    current: int\n\n    def __init__(self):\n        self.current = 0\n\n    async def anext(mut self) -> Result[Option[int], StreamError]:\n        if self.current > 1:\n            return None\n        value: int = self.current\n        self.current = self.current + 1\n        item: Option[int] = value\n        return item\n\n    async def aclose(mut self) -> Result[None, StreamError]:\n        return None\n\nasync def consume() -> Result[None, StreamError]:\n    stream = Stream()\n    async for value in stream:\n        if value == 0:\n            continue\n        raise StreamError(\"body\")\n    return None\n",
    );

    assert!(rust_code.contains("__sifr_native_async_for_iteration_future"));
    assert!(rust_code.contains("catch_unwind_future"));
    assert!(rust_code.contains(".aclose().await?"));
    assert!(rust_code.contains("record_async_cleanup_failed"));
    assert!(rust_code.contains("record_async_cleanup_timed_out"));
    assert!(rust_code.contains("release_and_resume_parent"));
    syn::parse_file(&rust_code).expect("closable async-for Rust should parse");
}
