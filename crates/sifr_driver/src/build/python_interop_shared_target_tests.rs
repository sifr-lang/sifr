use super::{apply_python_target_inspection, PythonTargetInspection};
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
    assert!(plan
        .target_probes
        .iter()
        .all(|probe| probe.status == PythonTargetProbeStatus::Verified));
}
