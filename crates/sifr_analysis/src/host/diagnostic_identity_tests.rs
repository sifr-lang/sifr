use super::*;

#[test]
fn analysis_lint_diagnostics_match_lint_engine_for_policy_rules() {
    let source = "# TODO: follow up\ndef main():\n    configure(True)\n";
    let mut host = AnalysisHost::open_single_file(
        &sifr_driver::CompilerContext::for_test_tokens(
            crate::compiled_input_tokens(),
            "sifr_analysis-tests",
        ),
        single_file_input(source),
    )
    .expect("host should load");
    let file = host.files()[0];
    let analysis_codes = host
        .diagnostics(file)
        .expect("diagnostics should query")
        .into_value()
        .into_iter()
        .filter(|diagnostic| matches!(diagnostic.args.get("rule"), Some(DiagnosticArg::String(_))))
        .map(|diagnostic| diagnostic.code)
        .collect::<Vec<_>>();
    let engine_codes = sifr_lint::lint_source(source, None, &sifr_lint::LintOptions::default())
        .diagnostics
        .into_iter()
        .map(|diagnostic| diagnostic.code)
        .collect::<Vec<_>>();
    assert_eq!(analysis_codes, engine_codes);
}

#[test]
fn workspace_diagnostic_order_is_stable_across_repeated_queries() {
    let source = "# TODO: follow up\ndef main():\n    return 1  \n";
    let mut host = AnalysisHost::open_single_file(
        &sifr_driver::CompilerContext::for_test_tokens(
            crate::compiled_input_tokens(),
            "sifr_analysis-tests",
        ),
        single_file_input(source),
    )
    .expect("host should load");

    let first = host
        .workspace_diagnostics()
        .expect("workspace diagnostics should query")
        .into_value();
    let second = host
        .workspace_diagnostics()
        .expect("workspace diagnostics should query again")
        .into_value();

    assert_eq!(first, second);
    assert!(first.iter().any(|file| !file.diagnostics.is_empty()));
}

#[test]
fn workspace_diagnostic_order_is_stable_under_parallel_readers() {
    let dir = temp_project_dir("parallel_diagnostic_order");
    std::fs::write(
        dir.join("main.sifr"),
        "# TODO: main follow up\ndef main():\n    return 1  \n",
    )
    .expect("main source should be written");
    std::fs::write(
        dir.join("helper.sifr"),
        "# TODO: helper follow up\ndef helper() -> int:\n    return 2  \n",
    )
    .expect("helper source should be written");
    let root = ProjectRoot {
        root: SourcePath::new(dir.clone()),
        entrypoint: SourcePath::new(dir.join("main.sifr")),
    };

    let mut host = AnalysisHost::open_project(
        &sifr_driver::CompilerContext::for_test_tokens(
            crate::compiled_input_tokens(),
            "sifr_analysis-tests",
        ),
        &root,
    )
    .expect("project host should load");
    let expected = host
        .workspace_diagnostics()
        .expect("workspace diagnostics should query")
        .into_value();
    let snapshot = host.snapshot();
    let shared_host = std::sync::Arc::new(std::sync::Mutex::new(host));
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(8));

    let handles = (0..8)
        .map(|_| {
            let snapshot = snapshot.clone();
            let shared_host = std::sync::Arc::clone(&shared_host);
            let barrier = std::sync::Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                let mut host = shared_host
                    .lock()
                    .expect("shared host should not be poisoned");
                snapshot
                    .workspace_diagnostics(&mut host)
                    .expect("shared snapshot diagnostics should query")
                    .into_value()
            })
        })
        .collect::<Vec<_>>();

    for handle in handles {
        let diagnostics = handle.join().expect("parallel reader should not panic");
        assert_eq!(diagnostics, expected);
    }
    assert!(expected.iter().any(|file| !file.diagnostics.is_empty()));

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn code_actions_offer_policy_suppression_and_explain_not_found_is_explicit() {
    let source = "def main():\n    return 1  \n";
    let mut host = AnalysisHost::open_single_file(
        &sifr_driver::CompilerContext::for_test_tokens(
            crate::compiled_input_tokens(),
            "sifr_analysis-tests",
        ),
        single_file_input(source),
    )
    .expect("host should load");
    let file = host.files()[0];
    let range = full_range(source).expect("source should fit in range");
    let actions = host
        .code_actions(
            file,
            range,
            &CodeActionContext {
                diagnostics: vec![DiagnosticId::policy(
                    "SIFR-LINT-0004",
                    "trailing-whitespace",
                )],
            },
        )
        .expect("code actions should query")
        .into_value();
    assert!(
        actions
            .iter()
            .any(|action| action.kind == "quickfix.sifr.suppress" && action.edit.is_some()),
        "lint diagnostics should offer explicit suppression edits"
    );
    assert!(
        actions
            .iter()
            .any(|action| action.kind == "quickfix.sifr.applySafeFix" && action.edit.is_some()),
        "safe policy diagnostics should offer explicit fix edits"
    );
    assert!(
        actions
            .iter()
            .any(|action| action.kind == "source.fixAll.sifr" && action.edit.is_none()),
        "safe policy diagnostics should offer deferred fix-all"
    );

    let hard_actions = host
        .code_actions(
            file,
            range,
            &CodeActionContext {
                diagnostics: vec![DiagnosticId::hard("SIFR-TYPE-0001")],
            },
        )
        .expect("hard diagnostic code actions should query")
        .into_value();
    assert!(
        hard_actions.is_empty(),
        "hard diagnostics must not offer policy suppression or fix actions"
    );

    let explanation = host
        .explain_diagnostic(&DiagnosticId::hard("SIFR-NOPE-0000"))
        .expect("explain diagnostic should query")
        .into_value();
    assert!(explanation.diagnostic.is_none());
    assert!(explanation.unavailable_reason.is_some());
}
