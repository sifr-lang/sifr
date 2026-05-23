#[test]
fn test_task_group_basic_lowers_to_scope_runtime_substrate() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def worker() -> int:\n    return 41\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.TaskGroup() as group:\n        handle = group.spawn(worker())\n        result = await handle.join()\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("struct __SifrTaskScope"));
    assert!(result
        .rust_source
        .contains("let mut group = __SifrTaskScope::new_task_group();"));
    assert!(result.rust_source.contains("fail_fast"));
    assert!(result
        .rust_source
        .contains("group.__sifr_spawn_infallible(worker());"));
    assert!(result
        .rust_source
        .contains("if let Err(__sifr_scope_failure) = group.__sifr_join_all().await"));
    assert!(result.required_crates.contains("tokio"));
}

#[test]
fn test_task_gather_lowers_to_private_gather_helper() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def first() -> int:\n    return 1\n\nasync def second() -> int:\n    return 2\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        result = await task.gather([scope.spawn(first()), scope.spawn(second())])\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("async fn __sifr_task_gather"));
    assert!(result.rust_source.contains("__sifr_task_gather(vec!["));
    assert!(result.rust_source.contains("abort_handle.abort();"));
    assert!(result.rust_source.contains("failure_results"));
    assert!(result.rust_source.contains("push_secondary_message"));
    assert!(result.rust_source.contains("ordered_values.push(value);"));
    assert!(result
        .rust_source
        .contains("let result: __SifrTaskResult<Vec<i64>, std::convert::Infallible>"));
    assert!(result.required_crates.contains("tokio"));
}

#[test]
fn test_scope_spawn_fallible_coroutine_lowers_to_result_spawn_helper() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def worker() -> Result[int, ValueError]:\n    raise ValueError(\"bad\")\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        handle = scope.spawn(worker())\n        result = await handle\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result
        .rust_source
        .contains("tokio::sync::oneshot::Receiver<__SifrTaskResult<T, E>>"));
    assert!(result
        .rust_source
        .contains("scope.__sifr_spawn_result(worker());"));
    assert!(result.rust_source.contains("enum __SifrTaskResult<T, E>"));
    assert!(result.rust_source.contains("Err(__SifrFailure<E>)"));
    assert!(result
        .rust_source
        .contains("__SifrTaskResult::Err(__SifrFailure::new(err))"));
    assert!(result
        .rust_source
        .contains("let result: __SifrTaskResult<i64, ValueError>"));
    assert!(result.required_crates.contains("tokio"));
}

#[test]
fn test_task_gather_fallible_tasks_keeps_error_parameter_unwrapped() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def worker() -> Result[int, ValueError]:\n    raise ValueError(\"bad\")\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        result = await task.gather([scope.spawn(worker()), scope.spawn(worker())])\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("async fn __sifr_task_gather"));
    assert!(result.rust_source.contains("__SifrTaskResult<Vec<T>, E>"));
    assert!(result.rust_source.contains("Err(__SifrFailure<E>)"));
    assert!(result
        .rust_source
        .contains("sibling task failed\".to_string()"));
    assert!(result
        .rust_source
        .contains("sibling task was cancelled\".to_string()"));
    assert!(result
        .rust_source
        .contains("__SifrTaskResult::Err(__SifrFailure::new(err))"));
    assert!(result
        .rust_source
        .contains("let result: __SifrTaskResult<Vec<i64>, ValueError>"));
}

#[test]
fn test_task_race_lowers_to_private_race_helper() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def first() -> int:\n    return 1\n\nasync def second() -> int:\n    await task.sleep(1.0)\n    return 2\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        result = await task.race([scope.spawn(first()), scope.spawn(second())])\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("async fn __sifr_task_race"));
    assert!(result.rust_source.contains("__sifr_task_race(vec!["));
    assert!(result.rust_source.contains("let Some(mut first)"));
    assert!(result
        .rust_source
        .contains("race loser task failed\".to_string()"));
    assert!(result
        .rust_source
        .contains("let result: __SifrTaskResult<i64, std::convert::Infallible>"));
    assert!(result.required_crates.contains("tokio"));
}

#[test]
fn test_task_race_fallible_tasks_keeps_error_parameter_unwrapped() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def worker() -> Result[int, ValueError]:\n    raise ValueError(\"bad\")\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        result = await task.race([scope.spawn(worker()), scope.spawn(worker())])\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("async fn __sifr_task_race"));
    assert!(result.rust_source.contains("__SifrTaskResult<T, E>"));
    assert!(result.rust_source.contains("Err(__SifrFailure<E>)"));
    assert!(result
        .rust_source
        .contains("race loser task was cancelled\".to_string()"));
    assert!(result
        .rust_source
        .contains("__SifrTaskResult::Err(__SifrFailure::new(err))"));
    assert!(result
        .rust_source
        .contains("let result: __SifrTaskResult<i64, ValueError>"));
}

#[test]
fn test_task_select_lowers_to_private_select_helper() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def first() -> int:\n    return 1\n\nasync def second() -> str:\n    await task.sleep(1.0)\n    return \"two\"\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        first_handle = scope.spawn(first())\n        second_handle = scope.spawn(second())\n        result = await task.select(first_handle, second_handle)\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("enum __SifrSelect2<A, B>"));
    assert!(result.rust_source.contains("async fn __sifr_task_select"));
    assert!(result
        .rust_source
        .contains("__sifr_task_select(first_handle, second_handle)"));
    assert!(result
        .rust_source
        .contains("select loser task failed\".to_string()"));
    assert!(result
        .rust_source
        .contains("select loser task was cancelled\".to_string()"));
    assert!(result
        .rust_source
        .contains("second_observed.store(false, std::sync::atomic::Ordering::SeqCst)"));
    assert!(result
        .rust_source
        .contains("first_observed.store(false, std::sync::atomic::Ordering::SeqCst)"));
    assert!(result.rust_source.contains(
        "let result: __SifrSelect2<__SifrTaskResult<i64, std::convert::Infallible>, __SifrTaskResult<String, std::convert::Infallible>>"
    ));
    assert!(result.required_crates.contains("tokio"));
}

#[test]
fn test_task_select_fallible_tasks_preserves_distinct_error_parameters() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def first() -> Result[int, ValueError]:\n    raise ValueError(\"first\")\n\nasync def second() -> Result[str, IOError]:\n    raise IOError(\"second\")\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        first_handle = scope.spawn(first())\n        second_handle = scope.spawn(second())\n        result = await task.select(first_handle, second_handle)\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("__SifrTaskResult<A, EA>"));
    assert!(result.rust_source.contains("__SifrTaskResult<B, EB>"));
    assert!(result.rust_source.contains("Err(__SifrFailure<E>)"));
    assert!(result.rust_source.contains(
        "let result: __SifrSelect2<__SifrTaskResult<i64, ValueError>, __SifrTaskResult<String, IOError>>"
    ));
}

#[test]
fn test_task_handle_join_lowers_to_task_result_observation() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def worker() -> int:\n    return 41\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        handle = scope.spawn(worker())\n        result = await handle.join()\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("enum __SifrTaskResult<T, E>"));
    assert!(result.rust_source.contains("async fn join(self)"));
    assert!(result
        .rust_source
        .contains("Cancelled(__SifrFailure<CancellationError>)"));
    assert!(result.rust_source.contains("fn cancelled() -> Self"));
    assert!(result.rust_source.contains("handle.join().await"));
    assert!(result
        .rust_source
        .contains("let result: __SifrTaskResult<i64, std::convert::Infallible>"));
}

#[test]
fn test_await_task_handle_desugars_to_join_observation() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def worker() -> int:\n    return 41\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        handle = scope.spawn(worker())\n        result = await handle\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("handle.join().await"));
    assert!(result
        .rust_source
        .contains("let result: __SifrTaskResult<i64, std::convert::Infallible>"));
}

#[test]
fn test_task_handle_cancel_borrows_handle_and_aborts_child() {
    let source = concat!(
        "async def worker() -> int:\n    await task.sleep(10.0)\n    return 41\n\n",
        "async def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n",
        "        handle = scope.spawn(worker())\n        handle.",
        "cancel",
        "()\n        result = await handle\n    return None\n",
    );
    let result = generate_rust_with_metadata(
        &lower_module(parse_module(source).expect("parse failed").suite())
            .expect("lowering failed")
            .module,
    );

    assert!(result
        .rust_source
        .contains("abort_handle: tokio::task::AbortHandle"));
    assert!(result
        .rust_source
        .contains(&format!("fn {}{}", "can", "cel(&self)")));
    assert!(result
        .rust_source
        .contains(&format!("handle.{}{}", "can", "cel();")));
    assert!(result.rust_source.contains("struct CancellationError"));
    assert!(result.rust_source.contains("__SifrTaskResult::cancelled()"));
    assert!(result.rust_source.contains("handle.join().await"));
}

#[test]
fn test_task_timeout_handle_lowers_to_private_timeout_result() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def worker() -> int:\n    return 41\n\nasync def main() -> Result[None, ScopeFailure]:\n    async with task.scope() as scope:\n        handle = scope.spawn(worker())\n        result = await task.timeout(handle, 1.0)\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("enum __SifrTimeoutResult<E>"));
    assert!(result.rust_source.contains("async fn __sifr_timeout"));
    assert!(result.rust_source.contains("biased;"));
    assert!(result.rust_source.contains("handle.__sifr_timeout"));
    assert!(result
        .rust_source
        .contains("failure.map_primary(__SifrTimeoutResult::Inner)"));
    assert!(result
        .rust_source
        .contains("__SifrFailure::new(__SifrTimeoutResult::Timeout)"));
    assert!(result.rust_source.contains(
        "let result: __SifrTaskResult<i64, __SifrTimeoutResult<std::convert::Infallible>>"
    ));
}

#[test]
fn test_failure_cancellation_error_annotation_lowers_to_private_evidence_type() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "def observe(failure: Failure[CancellationError]) -> None:\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("struct __SifrFailure<E>"));
    assert!(result.rust_source.contains("struct CancellationError"));
    assert!(result
        .rust_source
        .contains("fn observe(failure: &__SifrFailure<CancellationError>)"));
}

#[test]
fn test_failure_annotation_lowers_to_private_failure_type() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module("def observe(failure: Failure[ValueError]) -> None:\n    return None\n")
                .expect("parse failed")
                .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("struct __SifrFailure<E>"));
    assert!(result.rust_source.contains("primary: E"));
    assert!(result
        .rust_source
        .contains("secondary: Vec<SecondaryError>"));
    assert!(result
        .rust_source
        .contains("fn observe(failure: &__SifrFailure<ValueError>)"));
}

#[test]
fn test_task_timeout_context_manager_wraps_awaits() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def main() -> Result[None, TimeoutError]:\n    async with task.timeout(1.0):\n        await task.sleep(0.0)\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("match tokio::time::timeout"));
    assert!(result.rust_source.contains("return Err(TimeoutError::new"));
    assert!(result.rust_source.contains("struct TimeoutError"));
    assert!(result.required_crates.contains("tokio"));
}

#[test]
fn test_try_except_with_async_body_lowers_to_async_try_closure() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def main() -> Result[None, Error]:\n    try:\n        async with task.timeout(1.0):\n            await task.sleep(0.0)\n    except TimeoutError:\n        return None\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result.rust_source.contains("async ||"));
    assert!(result.rust_source.contains(")().await"));
    assert!(result.rust_source.contains("let __sifr_try_res"));
}

#[test]
fn test_try_finally_runs_cleanup_before_timeout_propagates() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def main() -> Result[None, Error]:\n    try:\n        async with task.timeout(0.0):\n            try:\n                await task.sleep(10.0)\n            finally:\n                marker: int = 1\n    except TimeoutError:\n        return None\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    let cleanup_pos = result
        .rust_source
        .find("let marker: i64 = 1 as i64;")
        .expect("cleanup marker should be emitted");
    let rethrow_pos = result
        .rust_source
        .find("if let Err(__sifr_finally_err) = __sifr_try_finally_res")
        .expect("try/finally should rethrow after cleanup");

    assert!(result.rust_source.contains("let __sifr_try_finally_res"));
    assert!(cleanup_pos < rethrow_pos);
}

#[test]
fn test_try_finally_cleanup_try_except_lowers_question_mark_calls() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                r#"def fallible() -> Result[str, Error]:
    return "ok"

def main() -> Result[None, Error]:
    try:
        body_out: str = fallible()
    except Error as e:
        body_out = str(e.message)
    finally:
        try:
            cleanup_out: str = fallible()
        except Error as e:
            cleanup_out = str(e.message)
    return None
"#,
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(!result.rust_source.contains("compile_error!"));
    assert!(result.rust_source.contains("let __sifr_try_finally_res"));
    assert!(result.rust_source.contains("let __sifr_try_res"));
    assert!(result.rust_source.contains("fallible()?"));
}

#[test]
fn test_async_generated_errors_convert_to_error_return_type() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module(
                "async def worker() -> int:\n    return 41\n\nasync def main() -> Result[None, Error]:\n    async with task.timeout(1.0):\n        await task.sleep(0.0)\n    async with task.scope() as scope:\n        handle = scope.spawn(worker())\n    return None\n",
            )
            .expect("parse failed")
            .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(result
        .rust_source
        .contains("impl From<TimeoutError> for Error"));
    assert!(result
        .rust_source
        .contains("impl From<ScopeFailure> for Error"));
    assert!(result
        .rust_source
        .contains("return Err(TimeoutError::new(\"task timeout expired\".to_string()).into())"));
    assert!(result
        .rust_source
        .contains("return Err(__sifr_scope_failure.into());"));
}

#[test]
fn test_sync_main_does_not_require_tokio() {
    let result = generate_rust_with_metadata(
        &lower_module(
            parse_module("def main() -> None:\n    return None\n")
                .expect("parse failed")
                .suite(),
        )
        .expect("lowering failed")
        .module,
    );

    assert!(!result
        .rust_source
        .contains("#[tokio::main(flavor = \"current_thread\")]"));
    assert!(!result.required_crates.contains("tokio"));
}

#[test]
fn test_generate_project_emits_tokio_dependency_when_required() {
    let module = empty_module();
    let required_crates = HashSet::from(["tokio".to_string()]);
    let (cargo_toml, _main_rs) = generate_project_with_deps_and_crates(
        &module,
        "sifr_output",
        &HashSet::new(),
        &required_crates,
    );

    assert!(cargo_toml.contains(
        "tokio = { version = \"1.52.3\", features = [\"macros\", \"rt\", \"sync\", \"time\"] }"
    ));
}

#[test]
fn test_module_constants_flow_through_assembled_body_items() {
    let module_constants_src = include_str!("module_constants.rs");
    let entrypoints_src = include_str!("entrypoints.rs");
    let lib_src = include_str!("lib.rs");

    assert!(module_constants_src.contains("self.body_items.push(item);"));
    assert!(module_constants_src
        .contains("structured module constant emission missing for production path"));
    assert!(!module_constants_src.contains("push_syn_items_from_source"));
    assert!(!module_constants_src.contains("render_items(&[item])"));

    assert!(entrypoints_src.contains("if !emitter.body_items.is_empty() {"));
    assert!(lib_src.contains("if !emitter.body_items.is_empty() {"));
    assert!(!entrypoints_src.contains("assert_output_drained("));
    assert!(!entrypoints_src.contains("emitter.output"));
    assert!(!lib_src.contains("emitter.output"));
}

#[test]
fn test_module_body_flows_through_assembled_body_items() {
    let module_body_src = include_str!("module_body.rs");
    let lib_src = include_str!("lib.rs");

    assert!(!module_body_src.contains("self.drain_emitted_output_items("));
    assert!(!module_body_src.contains("self.push_syn_items_from_source(&emitted"));
    assert!(module_body_src.contains("self.emit_class(class, module, module_public);"));
    assert!(module_body_src.contains("self.emit_function(func, module_public, test_mode);"));
    assert!(!module_body_src.contains("self.output"));
    assert!(lib_src.contains("if !emitter.body_items.is_empty() {"));
}

#[test]
fn test_generator_init_emission_is_structured_only() {
    let stmt_support_src = include_str!("stmt_support_emitter.rs");
    assert!(stmt_support_src.contains("self.lower_stmt_expr_for_ir(value)"));
    assert!(stmt_support_src.contains("self.try_lower_structured_stmt(stmt)"));
    assert!(stmt_support_src
        .contains("structured generator-init expression emission missing for production path"));
    assert!(stmt_support_src
        .contains("structured generator-init statement emission missing for production path"));
    assert!(!stmt_support_src.contains("self.try_emit_expr_string_"));
    assert!(!stmt_support_src.contains("self.try_emit_stmt_string_"));
    assert!(!stmt_support_src.contains("self.emit_expr(value);"));
    assert!(!stmt_support_src.contains("self.emit_stmt(stmt);"));
}

#[test]
fn test_expr_side_effect_emitter_layer_is_removed() {
    let expr_render_helpers_src = include_str!("expr_render_helpers.rs");
    let output_helpers_src = include_str!("output_helpers.rs");
    let intrinsic_emitters_src = include_str!("intrinsic_method_emitters.rs");
    let lib_support_src = include_str!("lib_support.rs");

    assert!(!expr_render_helpers_src.contains("fn try_emit_structured_"));
    assert!(!expr_render_helpers_src.contains("emit_fstring_macro("));
    assert!(!output_helpers_src.contains("expression string emission is forbidden"));
    assert!(!intrinsic_emitters_src.contains("write_registry_expr"));
    assert!(!lib_support_src.contains("reserved_plain_builtin"));
}

#[test]
fn test_production_codegen_source_has_no_non_ir_tokens() {
    let src_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let banned_tokens = [
        "RawCode",
        "SynItem",
        "self.write(",
        "self.writeln(",
        "emit_rust_expr(",
        "emit_rust_stmt_with_current_indent(",
        "write_registry_expr(",
    ];

    let mut stack = vec![src_root];
    let mut violations = Vec::new();
    while let Some(dir) = stack.pop() {
        let entries = std::fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("failed to read source dir {}: {e}", dir.display()));
        for entry in entries {
            let path = entry
                .unwrap_or_else(|e| {
                    panic!("failed to read directory entry in {}: {e}", dir.display())
                })
                .path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                continue;
            }
            let file_name = path.file_name().and_then(|name| name.to_str());
            if file_name == Some("lib_codegen_tests.rs")
                || file_name.is_some_and(|name| name.ends_with("_tests.rs"))
            {
                continue;
            }
            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
            for token in banned_tokens {
                if content.contains(token) {
                    violations.push(format!(
                        "{} contains forbidden token `{token}`",
                        path.display()
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "production codegen source contains forbidden non-IR tokens:\n{}",
        violations.join("\n")
    );
}

#[test]
fn test_round_parenthesizes_cast_receiver() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Expr {
                expr: HirExpr::Call {
                    func: "print".to_string(),
                    args: vec![HirExpr::Call {
                        func: "round".to_string(),
                        args: vec![HirExpr::Call {
                            func: "float".to_string(),
                            args: vec![HirExpr::IntLiteral(3)],
                            ty: Type::Float,
                        }],
                        ty: Type::Int,
                    }],
                    ty: Type::None,
                },
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let rust_code = generate_rust(&module);
    assert!(
        rust_code.contains("((3 as i64) as f64).round() as i64"),
        "expected round receiver to be parenthesized; got: {rust_code}"
    );
    assert!(
        !rust_code.contains("as f64.round()"),
        "invalid Rust precedence should not be emitted"
    );
}

#[test]
fn test_float_min_max_parenthesize_cast_receivers() {
    let float_one = HirExpr::Call {
        func: "float".to_string(),
        args: vec![HirExpr::IntLiteral(1)],
        ty: Type::Float,
    };
    let float_two = HirExpr::Call {
        func: "float".to_string(),
        args: vec![HirExpr::IntLiteral(2)],
        ty: Type::Float,
    };

    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![
                HirStmt::Expr {
                    expr: HirExpr::Call {
                        func: "print".to_string(),
                        args: vec![HirExpr::Call {
                            func: "min".to_string(),
                            args: vec![float_one.clone(), float_two.clone()],
                            ty: Type::Float,
                        }],
                        ty: Type::None,
                    },
                },
                HirStmt::Expr {
                    expr: HirExpr::Call {
                        func: "print".to_string(),
                        args: vec![HirExpr::Call {
                            func: "max".to_string(),
                            args: vec![float_one, float_two],
                            ty: Type::Float,
                        }],
                        ty: Type::None,
                    },
                },
            ],
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let rust_code = generate_rust(&module);
    assert!(
        rust_code.contains("((1 as i64) as f64).min((2 as i64) as f64)"),
        "expected min receiver to be parenthesized; got: {rust_code}"
    );
    assert!(
        rust_code.contains("((1 as i64) as f64).max((2 as i64) as f64)"),
        "expected max receiver to be parenthesized; got: {rust_code}"
    );
    assert!(
        !rust_code.contains("as f64.min(") && !rust_code.contains("as f64.max("),
        "invalid Rust precedence should not be emitted"
    );
}

#[test]
fn test_variadic_min_max_lower_to_nested_calls() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![
                HirStmt::Expr {
                    expr: HirExpr::Call {
                        func: "print".to_string(),
                        args: vec![HirExpr::Call {
                            func: "min".to_string(),
                            args: vec![
                                HirExpr::IntLiteral(3),
                                HirExpr::IntLiteral(1),
                                HirExpr::IntLiteral(2),
                            ],
                            ty: Type::Int,
                        }],
                        ty: Type::None,
                    },
                },
                HirStmt::Expr {
                    expr: HirExpr::Call {
                        func: "print".to_string(),
                        args: vec![HirExpr::Call {
                            func: "max".to_string(),
                            args: vec![
                                HirExpr::IntLiteral(1),
                                HirExpr::IntLiteral(5),
                                HirExpr::IntLiteral(2),
                                HirExpr::IntLiteral(4),
                            ],
                            ty: Type::Int,
                        }],
                        ty: Type::None,
                    },
                },
            ],
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let rust_code = generate_rust(&module);
    assert!(
        rust_code.matches("std::cmp::min").count() >= 2,
        "variadic min should lower to nested std::cmp::min calls: {rust_code}"
    );
    assert!(
        rust_code.matches("std::cmp::max").count() >= 3,
        "variadic max should lower to nested std::cmp::max calls: {rust_code}"
    );
}
