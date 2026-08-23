use super::{PythonTargetInspection, apply_python_target_inspection};
use sifr_codegen::{PythonInteropPlan, PythonTargetProbe, PythonTargetProbeStatus};

#[test]
fn shared_target_inspection_updates_every_probe_status() {
    let probe = PythonTargetProbe {
        import_root: Some("math".to_string()),
        target_path: "math.sqrt".to_string(),
        requires_inspectable_signature: false,
        expects_type: false,
        status: PythonTargetProbeStatus::Planned,
    };
    let mut plan = PythonInteropPlan {
        target_probes: vec![probe.clone(), probe],
        ..PythonInteropPlan::default()
    };
    let inspection = PythonTargetInspection {
        ok: true,
        callable: true,
        is_type: false,
        inspectable: true,
        parameters: Vec::new(),
        error: None,
    };

    assert!(apply_python_target_inspection(&mut plan, "math.sqrt", Ok(&inspection)).is_empty());
    assert!(
        plan.target_probes
            .iter()
            .all(|probe| probe.status == PythonTargetProbeStatus::Verified)
    );
}

#[test]
fn shared_target_inspection_combines_type_constraints() {
    let unconstrained = PythonTargetProbe {
        import_root: Some("math".to_string()),
        target_path: "math.sqrt".to_string(),
        requires_inspectable_signature: false,
        expects_type: false,
        status: PythonTargetProbeStatus::Planned,
    };
    let mut constrained = unconstrained.clone();
    constrained.expects_type = true;
    let mut plan = PythonInteropPlan {
        target_probes: vec![unconstrained, constrained],
        ..PythonInteropPlan::default()
    };
    let inspection = PythonTargetInspection {
        ok: true,
        callable: true,
        is_type: false,
        inspectable: true,
        parameters: Vec::new(),
        error: None,
    };

    let diagnostics = apply_python_target_inspection(&mut plan, "math.sqrt", Ok(&inspection));

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].diagnostic.code, "SIFR-PYCALL-0001");
    assert!(
        plan.target_probes
            .iter()
            .all(|probe| probe.status == PythonTargetProbeStatus::Planned)
    );
}

#[test]
fn shared_target_inspection_combines_inspectability_constraints() {
    let unconstrained = PythonTargetProbe {
        import_root: Some("builtins".to_string()),
        target_path: "builtins.print".to_string(),
        requires_inspectable_signature: false,
        expects_type: false,
        status: PythonTargetProbeStatus::Planned,
    };
    let mut constrained = unconstrained.clone();
    constrained.requires_inspectable_signature = true;
    let mut plan = PythonInteropPlan {
        target_probes: vec![unconstrained, constrained],
        ..PythonInteropPlan::default()
    };
    let inspection = PythonTargetInspection {
        ok: true,
        callable: true,
        is_type: false,
        inspectable: false,
        parameters: Vec::new(),
        error: None,
    };

    let diagnostics = apply_python_target_inspection(&mut plan, "builtins.print", Ok(&inspection));

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].diagnostic.code, "SIFR-PYCALL-0001");
    assert!(
        plan.target_probes
            .iter()
            .all(|probe| probe.status == PythonTargetProbeStatus::Planned)
    );
}
