use super::entrypoint::compile_single_file_frontend;
use super::project_codegen::codegen_single_file_frontend;

#[test]
fn runtime_method_fixtures_emit_without_lowering_errors() {
    let compiler = crate::CompilerContext::for_test();
    let fixtures = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../sifr/tests/e2e/pass");
    let mut failures = Vec::new();
    for name in [
        "async_generator_aclose_before_start",
        "async_generator_aclose_result",
        "async_generator_basic",
        "join_set_add_blocking_task",
        "join_set_add_task_join_all",
        "join_set_bound_terminal_await",
        "join_set_cancel_all_evidence",
        "join_set_cancel_all_task_cancelled",
        "join_set_spawn_blocking",
        "join_set_spawn_cpu_join_all_ordered",
        "network_http_tls_loopback_split",
        "process_scoped_parent_cancel",
        "process_scoped_spawn_handle",
        "task_cancel_basic",
        "task_context_propagation_basic",
        "task_context_reuse_across_spawns",
        "task_group_basic",
        "task_group_spawn_cpu",
        "task_group_spawn_cpu_user_error",
        "task_handle_collection_loop_consumed",
        "task_handle_join",
        "task_scope_basic",
        "task_scope_spawn_blocking",
    ] {
        let path = fixtures.join(format!("{name}.sifr"));
        let source = match std::fs::read_to_string(&path) {
            Ok(source) => source,
            Err(error) => {
                failures.push(format!("{name}: fixture read failed: {error}"));
                continue;
            }
        };
        let generated = compile_single_file_frontend(&compiler, &source)
            .and_then(|frontend| codegen_single_file_frontend(&frontend));
        let output = match generated {
            Ok(output) => output,
            Err(errors) => {
                failures.push(format!("{name}: frontend/codegen failed: {errors:?}"));
                continue;
            }
        };
        let stats = output.lowering_stats;
        let errors = output
            .rust_source
            .lines()
            .filter(|line| line.contains("compile_error!"))
            .collect::<Vec<_>>();
        if stats.stmt_lowering_errors != 0
            || stats.expr_lowering_errors != 0
            || stats.item_lowering_errors != 0
            || !errors.is_empty()
        {
            failures.push(format!(
                "{name}: statement={}, expression={}, item={}\n{}",
                stats.stmt_lowering_errors,
                stats.expr_lowering_errors,
                stats.item_lowering_errors,
                errors.join("\n"),
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}
