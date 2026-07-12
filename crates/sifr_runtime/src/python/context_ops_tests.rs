use super::context_ops::*;
use super::foreign_object::pending_release_count;
use super::object_ops::{close_object, copy_list_bool, copy_list_str, store_object};
use super::{
    attach, initialize_runtime, reset_runtime_state_for_tests, resource_diagnostics, test_config,
    test_guard, PythonError, PythonResourceDiagnostics,
};
use pyo3::prelude::*;
use pyo3::types::PyModule;

#[test]
fn python_exception_replay_preserves_exact_triple_across_nested_exits() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("context-replay")).expect("init should succeed");

    let (first, second, log, error) = attach(|py| {
        let module = PyModule::from_code(
            py,
            c"log = []\nclass Marker(Exception):\n    pass\nORIGINAL = Marker('original')\nclass Manager:\n    def __exit__(self, exc_type, exc_value, tb):\n        log.append(exc_type is Marker and exc_value is ORIGINAL and tb is ORIGINAL.__traceback__)\n        return False\ndef fail():\n    raise ORIGINAL\n",
            c"context_replay.py",
            c"context_replay",
        )
        .expect("module should build");
        let manager = module.getattr("Manager").expect("manager class");
        let first = manager.call0().expect("first manager");
        let second = manager.call0().expect("second manager");
        let log = module.getattr("log").expect("log");
        let error = module
            .getattr("fail")
            .and_then(|fail| fail.call0())
            .expect_err("fail should raise");
        (
            store_object(first.unbind()).expect("first should store"),
            store_object(second.unbind()).expect("second should store"),
            store_object(log.unbind()).expect("log should store"),
            PythonError::from_pyerr(py, error, "call", "nested context body"),
        )
    })
    .expect("attach should succeed");

    let first_owner = error.clone();
    let final_owner = error.clone();
    assert_eq!(
        context_exit_python_error(first, &first_owner).expect("first exit should succeed"),
        PythonExitDecision::Propagate
    );
    assert_eq!(
        context_exit_python_error(second, &final_owner).expect("second exit should succeed"),
        PythonExitDecision::Propagate
    );
    assert_eq!(
        copy_list_bool(&log).expect("log should copy"),
        vec![true, true]
    );
    assert_eq!(resource_diagnostics().expect("diagnostics").live_objects, 2);

    drop(error);
    drop(first_owner);
    assert_eq!(resource_diagnostics().expect("diagnostics").live_objects, 2);
    assert_eq!(pending_release_count(), 0);
    drop(final_owner);
    assert_eq!(pending_release_count(), 1);
    attach(|_| ()).expect("attach should drain final replay owner");
    assert_eq!(resource_diagnostics().expect("diagnostics").live_objects, 1);
    close_object(log).expect("log should close");
    assert_no_live_objects();
}

#[test]
fn normal_and_sifr_exit_calls_translate_truthiness_and_boundary_metadata() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("context-decisions")).expect("init should succeed");

    let (false_manager, true_manager, boundary_manager, log) = attach(|py| {
        let module = PyModule::from_code(
            py,
            c"log = []\nclass Toggle:\n    def __init__(self, decision):\n        self.decision = decision\n    def __exit__(self, exc_type, exc_value, tb):\n        return self.decision\nclass Boundary:\n    def __exit__(self, exc_type, exc_value, tb):\n        log.extend([exc_type.__name__ == 'SifrBoundaryError', exc_value.cause_kind == 'timeout', exc_value.sifr_type == 'DeadlineExceeded', exc_value.message == 'deadline elapsed', tb is None])\n        return True\n",
            c"context_decisions.py",
            c"context_decisions",
        )
        .expect("module should build");
        let toggle = module.getattr("Toggle").expect("toggle class");
        let boundary = module.getattr("Boundary").expect("boundary class");
        (
            store_object(toggle.call1((false,)).expect("false manager").unbind())
                .expect("false manager should store"),
            store_object(toggle.call1((true,)).expect("true manager").unbind())
                .expect("true manager should store"),
            store_object(boundary.call0().expect("boundary manager").unbind())
                .expect("boundary manager should store"),
            store_object(module.getattr("log").expect("log").unbind())
                .expect("log should store"),
        )
    })
    .expect("attach should succeed");

    assert_eq!(
        context_exit_normal(false_manager).expect("false exit should succeed"),
        PythonExitDecision::Propagate
    );
    assert_eq!(
        context_exit_normal(true_manager).expect("true exit should succeed"),
        PythonExitDecision::Suppress
    );
    let cause = SifrExitCause {
        kind: SifrExitCauseKind::Timeout,
        sifr_type: "DeadlineExceeded".to_string(),
        message: "deadline elapsed".to_string(),
    };
    assert_eq!(
        context_exit_sifr_cause(boundary_manager, &cause).expect("boundary exit should succeed"),
        PythonExitDecision::Suppress
    );
    assert_eq!(
        copy_list_bool(&log).expect("boundary log should copy"),
        vec![true, true, true, true, true]
    );
    close_object(log).expect("log should close");
    assert_no_live_objects();
}

#[test]
fn failing_exit_poison_releases_manager_once_and_retains_cleanup_error() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();
    initialize_runtime(test_config("context-exit-failure")).expect("init should succeed");

    let (manager, log) = attach(|py| {
        let module = PyModule::from_code(
            py,
            c"log = []\nclass Failing:\n    def __exit__(self, exc_type, exc_value, tb):\n        raise RuntimeError('cleanup failed')\n    def __del__(self):\n        log.append('released')\n",
            c"context_exit_failure.py",
            c"context_exit_failure",
        )
        .expect("module should build");
        (
            store_object(
                module
                    .getattr("Failing")
                    .and_then(|class| class.call0())
                    .expect("manager")
                    .unbind(),
            )
            .expect("manager should store"),
            store_object(module.getattr("log").expect("log").unbind())
                .expect("log should store"),
        )
    })
    .expect("attach should succeed");

    let error = context_exit_normal(manager).expect_err("exit should fail");
    assert_eq!(error.exception_type, "RuntimeError");
    assert_eq!(error.context, "__exit__ normal");
    assert!(copy_list_str(&log)
        .expect("release log should copy")
        .is_empty());
    assert_eq!(resource_diagnostics().expect("diagnostics").live_objects, 2);
    drop(error);
    attach(|_| ()).expect("attach should drain cleanup replay");
    assert_eq!(
        copy_list_str(&log).expect("release log should copy"),
        vec!["released"]
    );
    assert_eq!(resource_diagnostics().expect("diagnostics").live_objects, 1);
    close_object(log).expect("log should close");
    assert_no_live_objects();
}

#[test]
fn cleanup_failures_preserve_primary_error_and_record_secondary_evidence() {
    let _guard = test_guard();
    reset_runtime_state_for_tests();

    let mut primary = PythonError::without_replay(
        "call",
        "PrimaryError",
        "body failed",
        "traceback",
        "context body",
    );
    let secondary = PythonError::without_replay(
        "context",
        "CleanupError",
        "cleanup failed",
        "cleanup traceback",
        "__exit__",
    );
    attach_secondary_python_error(&mut primary, &secondary);
    assert_eq!(primary.exception_type, "PrimaryError");
    assert!(primary.context.contains("context body"));
    assert!(primary.context.contains("CleanupError: cleanup failed"));

    record_context_cleanup_evidence("DeadlineExceeded", &secondary);
    assert_eq!(
        take_context_cleanup_evidence(),
        vec![ContextCleanupEvidence {
            primary_cause: "DeadlineExceeded".to_string(),
            exception_type: "CleanupError".to_string(),
            message: "cleanup failed".to_string(),
            context: "__exit__".to_string(),
        }]
    );
    assert!(take_context_cleanup_evidence().is_empty());
}

fn assert_no_live_objects() {
    assert_eq!(
        resource_diagnostics().expect("diagnostics should be available"),
        PythonResourceDiagnostics {
            initialized: true,
            live_objects: 0,
            leaked_objects: 0,
        }
    );

    record_context_ignored_suppression("DeadlineExceeded");
    assert_eq!(
        take_context_cleanup_evidence(),
        vec![ContextCleanupEvidence {
            primary_cause: "DeadlineExceeded".to_string(),
            exception_type: "SifrBoundaryError".to_string(),
            message: "Python context suppression was ignored for a non-Python Sifr cause"
                .to_string(),
            context: "context exit decision".to_string(),
        }]
    );
}
