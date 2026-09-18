use super::{CapturedSource, Observation, SemanticInputs, identity};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Family completion is independent of source success and other families.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Family<T> {
    Pending,
    Cancelled,
    Crashed,
    EnvironmentBlocked,
    Complete(T),
}
impl<T> Family<T> {
    pub fn ready(&self) -> Option<&T> {
        match self {
            Self::Complete(value) => Some(value),
            _ => None,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceOutcome {
    Success,
    DeterministicErrors,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolutionResult {
    pub observations: Vec<Observation>,
    pub sources: Vec<CapturedSource>,
    pub semantic_inputs: SemanticInputs,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleInputs {
    pub source: CapturedSource,
    pub observations: Vec<Observation>,
    pub semantic_inputs: SemanticInputs,
    /// Exact consumed imported summaries. Unknown dependency scope must include
    /// all potentially consumed modules; DX.14 alone may narrow this boundary.
    pub imported_interfaces: BTreeMap<String, String>,
}
impl ModuleInputs {
    pub fn identity(&self) -> Result<String, serde_json::Error> {
        identity("module-input-v1", self)
    }
}
/// Full checked semantic export projection plus a conservative dependency stamp.
/// Until interface equivalence is qualified, any source/dependency edit changes
/// the interface, including bodies, inferred effects, defaults and constants.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticInterface<T> {
    pub module_identity: String,
    pub input_identity: String,
    pub exports: T,
}
impl<T: Serialize> SemanticInterface<T> {
    pub fn identity(&self) -> Result<String, serde_json::Error> {
        identity("module-interface-v1", self)
    }
}
/// Reuse consumers are explicit. HIR supports codegen and importer exports;
/// absent flow/editor/syntax families must be recomputed by the existing engine.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedModule<T> {
    pub module_identity: String,
    pub input_identity: String,
    pub hir: T,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CodegenHandoff {
    pub checked_module_identity: String,
    pub codegen_identity: String,
    pub required_specializations: BTreeMap<String, String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleResults<I, H, D> {
    pub inputs: ModuleInputs,
    pub outcome: SourceOutcome,
    pub resolution: Family<ResolutionResult>,
    pub interface: Family<SemanticInterface<I>>,
    pub checked: Family<CheckedModule<H>>,
    pub diagnostics: Family<Vec<D>>,
    pub codegen: Family<CodegenHandoff>,
}
impl<I, H, D> ModuleResults<I, H, D> {
    /// This contract never implies a rich editor index or executable exists.
    pub fn check_complete(&self) -> bool {
        self.inputs.semantic_inputs.complete()
            && self.resolution.ready().is_some_and(|resolution| {
                resolution.semantic_inputs == self.inputs.semantic_inputs
                    && resolution.sources.contains(&self.inputs.source)
            })
            && self.diagnostics.ready().is_some()
    }
    pub fn codegen_complete(&self, identity: &str) -> bool {
        self.outcome == SourceOutcome::Success
            && self.check_complete()
            && self.checked.ready().is_some_and(|checked| {
                self.inputs
                    .identity()
                    .is_ok_and(|id| checked.input_identity == id)
            })
            && self.codegen.ready().is_some_and(|handoff| {
                handoff.codegen_identity == identity
                    && self.checked.ready().is_some_and(|checked| {
                        checked.input_identity == handoff.checked_module_identity
                    })
            })
    }
}
