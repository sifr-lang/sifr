//! Python authoring and declaration validation shared by compiler and editor.
mod certification;
mod environment;
mod interop;
mod package_diagnostics;

pub use environment::{
    PythonEditorEnvironment, PythonEditorResolutionError, python_environment_selection,
    resolve_editor_environment, resolve_editor_environment_from_snapshot,
};
pub use package_diagnostics::render_package_diagnostic;
pub use sifr_codegen::{PythonInteropPlan, PythonTargetProbeStatus};

pub use certification::{
    validate_binding_distributions, validate_certification_distributions,
    validate_protocol_certifications_for_plan,
};
pub use interop::{
    PythonInteropPlanDiagnostic, PythonTargetInspection, PythonTargetParameter,
    apply_python_target_inspection, inspect_python_target, inspect_python_target_if_active,
    mark_embedded_bridge_targets, probe_python_interop_plan, validate_signature,
};

pub trait PythonCertificationRuntime {
    fn interpreter(&self) -> &std::path::Path;
    fn arrow_certification(&self, target: &str) -> Option<&sifr_package::ArrowCertification>;
    fn dlpack_certification(&self, target: &str) -> Option<&sifr_package::DlpackCertification>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PythonAuthoringCancelled;

/// The editor needs only the selected environment and its exact certification
/// state; native build and runtime code generation remain driver-owned.
#[derive(Clone, Debug)]
pub struct PythonAuthoringRuntime {
    interpreter: std::path::PathBuf,
    probe_digest: String,
    authoring_environment_digest: String,
    arrow_certifications: Vec<sifr_package::ArrowCertification>,
    dlpack_certifications: Vec<sifr_package::DlpackCertification>,
    binding_identity: String,
}

impl PythonAuthoringRuntime {
    pub fn from_probe(
        request: &sifr_package::PythonEnvironmentProbeRequest,
        probe: &sifr_package::PythonEnvironmentProbe,
    ) -> Result<Self, String> {
        let probe_digest = sifr_package::digest_python_environment_probe(request, probe)?.hex;
        let authoring_environment_digest =
            sifr_package::digest_python_authoring_environment_probe(request, probe)?.hex;
        Ok(Self {
            interpreter: request.interpreter.clone(),
            probe_digest,
            authoring_environment_digest,
            arrow_certifications: Vec::new(),
            dlpack_certifications: Vec::new(),
            binding_identity: String::new(),
        })
    }

    pub fn interpreter(&self) -> &std::path::Path {
        &self.interpreter
    }

    pub fn environment_digest(&self) -> &str {
        &self.probe_digest
    }

    pub fn authoring_environment_digest(&self) -> &str {
        &self.authoring_environment_digest
    }

    pub fn set_binding_identity(&mut self, identity: String) {
        self.binding_identity = identity;
    }

    pub fn set_arrow_certifications(
        &mut self,
        certifications: Vec<sifr_package::ArrowCertification>,
    ) -> Result<(), String> {
        serde_json::to_string(&certifications)
            .map_err(|error| format!("could not serialize Python certifications: {error}"))?;
        self.arrow_certifications = certifications;
        Ok(())
    }

    pub fn set_dlpack_certifications(
        &mut self,
        certifications: Vec<sifr_package::DlpackCertification>,
    ) -> Result<(), String> {
        serde_json::to_string(&certifications)
            .map_err(|error| format!("could not serialize Python certifications: {error}"))?;
        self.dlpack_certifications = certifications;
        Ok(())
    }
}

impl PythonCertificationRuntime for PythonAuthoringRuntime {
    fn interpreter(&self) -> &std::path::Path {
        &self.interpreter
    }

    fn arrow_certification(&self, target: &str) -> Option<&sifr_package::ArrowCertification> {
        self.arrow_certifications
            .iter()
            .find(|item| item.target == target)
    }

    fn dlpack_certification(&self, target: &str) -> Option<&sifr_package::DlpackCertification> {
        self.dlpack_certifications
            .iter()
            .find(|item| item.target == target)
    }
}

pub const fn status_name(status: PythonTargetProbeStatus) -> &'static str {
    match status {
        PythonTargetProbeStatus::Planned => "deferred",
        PythonTargetProbeStatus::Verified => "verified",
        PythonTargetProbeStatus::RuntimeChecked => "runtime-checked",
    }
}

pub fn policy_help(kind: &str) -> &'static str {
    match kind {
        "Coroutine" => "Runs on the application-owned Python loop with typed cancellation.",
        "Opaque" => {
            "Preserves sealed Python identity and the declaration's consuming cleanup policy."
        }
        "ContextEnter" | "ContextExit" | "ContextAsyncEnter" | "ContextAsyncExit" => {
            "Context cleanup is consuming and follows the declared suppression/error precedence."
        }
        "Callback" => {
            "Callback lifetime, dispatch, concurrency, and owner policy are compiler checked."
        }
        "Buffer" => "Buffer access is borrow-scoped with checked layout and exact release.",
        "Arrow" => "Arrow transfer is affine, certified, no-copy, and exact-release.",
        "Dlpack" | "DlpackStream" => {
            "DLPack transfer is one-shot, certified, no-copy, and exact-deleter."
        }
        _ => "Arguments and results use the compiler's closed typed Python conversion grammar.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sifr_codegen::PythonTargetProbe;
    use sifr_package::{
        ArrowCertifiedDistribution, PYTHON_CERTIFICATION_SCHEMA_VERSION,
        PYTHON_CERTIFICATIONS_FILE, PythonCertificationArtifact,
    };

    #[test]
    fn wrong_distribution_or_environment_digest_rejects_certification() {
        let root = tempfile::tempdir().expect("temporary package");
        let artifact = PythonCertificationArtifact {
            schema_version: PYTHON_CERTIFICATION_SCHEMA_VERSION,
            environment_digest: "environment-a".to_string(),
            arrow: Vec::new(),
            dlpack: Vec::new(),
        };
        std::fs::write(
            root.path().join(PYTHON_CERTIFICATIONS_FILE),
            serde_json::to_vec(&artifact).expect("artifact JSON"),
        )
        .expect("artifact");
        let error = sifr_package::load_python_certifications(root.path(), "environment-b")
            .expect_err("another environment cannot reuse this certification");
        assert!(error.contains("environment"), "{error}");
        let expected = [ArrowCertifiedDistribution {
            name: "pyarrow".to_string(),
            version: "25.0.1".to_string(),
        }];
        let error =
            certification::validate_distribution_versions(&expected, |_| Ok("25.0.0".to_string()))
                .expect_err("an installed distribution change must reject certification");
        assert!(error.contains("does not match"), "{error}");
    }

    #[test]
    fn uninspectable_target_remains_runtime_checked() {
        let mut plan = PythonInteropPlan::default();
        plan.target_probes.push(PythonTargetProbe {
            import_root: Some("builtins".to_string()),
            target_path: "builtins.dir".to_string(),
            requires_inspectable_signature: false,
            expects_type: false,
            status: PythonTargetProbeStatus::Planned,
        });
        let inspection = PythonTargetInspection {
            ok: true,
            callable: true,
            is_type: false,
            inspectable: false,
            parameters: Vec::new(),
            error: None,
        };
        assert!(
            apply_python_target_inspection(&mut plan, "builtins.dir", Ok(&inspection)).is_empty()
        );
        assert_eq!(
            plan.target_probes[0].status,
            PythonTargetProbeStatus::RuntimeChecked
        );
    }

    #[test]
    fn cancelled_probe_has_no_side_effect() {
        let calls = std::cell::Cell::new(0);
        let result = interop::inspect_if_active(
            std::path::Path::new("/unused/python"),
            "math.sqrt",
            || true,
            |_, _| {
                calls.set(calls.get() + 1);
                Err("probe must not run".to_string())
            },
        );
        assert_eq!(result, Err(PythonAuthoringCancelled));
        assert_eq!(calls.get(), 0);
    }
}
