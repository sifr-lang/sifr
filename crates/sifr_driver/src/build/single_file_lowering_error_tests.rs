//! Lowering admission must not depend on a currently unsupported language feature.
use super::cargo_invocation_trace::capture_cargo_invocations;
use super::single_file_interop_cache::resolve_single_file_metadata;
use super::stdlib_interop_demand_tests::generated;
use sifr_diagnostics::{DiagnosticCode, Severity};

const SOURCE: &str = "def main():\n    assert 1 + 1 == 2\n";

#[test]
fn lowering_error_counters_reject_before_and_after_successful_resolution() {
    // Other driver tests also record Cargo calls. An isolated test process
    // makes an empty trace meaningful even under parallel test execution.
    const CHILD: &str = "SIFR_LOWERING_COUNTER_CONTROL_CHILD";
    if std::env::var_os(CHILD).is_none() {
        let output = std::process::Command::new(std::env::current_exe().expect("test executable"))
            .args([
                "build::single_file_lowering_error_tests::lowering_error_counters_reject_before_and_after_successful_resolution",
                "--exact", "--nocapture", "--test-threads=1",
            ])
            .env(CHILD, "1")
            .output()
            .expect("isolated lowering counter control");
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed; 0 failed"));
        return;
    }
    let session = super::SingleFileInteropSession::new(Err(
        "native preparation must not run for lowering errors".to_owned(),
    ));
    for after_success in [false, true] {
        if after_success {
            let (output, stdlib) = generated(SOURCE);
            let expected_rust = output.rust_source.clone();
            let resolved =
                resolve_single_file_metadata(output, None, &stdlib.interop, Some(&session))
                    .expect("zero-error output must resolve successfully");
            assert_eq!(resolved.rust_source, expected_rust);
            assert_eq!(resolved.lowering_stats.stmt_lowering_errors, 0);
            assert_eq!(resolved.lowering_stats.expr_lowering_errors, 0);
            assert_eq!(resolved.lowering_stats.item_lowering_errors, 0);
        }
        // Each independent counter must reject, including after the identical
        // interop resolution was cached successfully. Totals are not error flags.
        for (statements, expressions, items) in [(1, 0, 0), (0, 1, 0), (0, 0, 1), (2, 3, 5)] {
            let (mut output, stdlib) = generated(SOURCE);
            output.lowering_stats.stmt_lowering_errors = statements;
            output.lowering_stats.expr_lowering_errors = expressions;
            output.lowering_stats.item_lowering_errors = items;
            let (result, invocations) = capture_cargo_invocations(|| {
                resolve_single_file_metadata(output, None, &stdlib.interop, Some(&session))
            });
            assert!(
                invocations.is_empty(),
                "lowering errors reached Cargo: {invocations:?}"
            );
            let errors = match result {
                Ok(_) => panic!(
                    "lowering errors became successful metadata; after_success={after_success}"
                ),
                Err(errors) => errors,
            };
            assert_eq!(errors.len(), 1);
            assert_eq!(
                errors[0].code,
                DiagnosticCode::BUILD_MATERIALIZATION_FAILURE.code()
            );
            assert_eq!(errors[0].severity, Severity::Error);
            assert_eq!(
                errors[0].message,
                format!(
                    "Rust lowering failed: {statements} statement, {expressions} expression, {items} item error(s)"
                )
            );
        }
    }
}
