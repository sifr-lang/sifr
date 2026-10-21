use serde::{Deserialize, Serialize};
use sifr_frontend::persistence::{CheckedModule, SemanticInterface};
use sifr_metadata as wire;
use std::collections::BTreeMap;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectInterfacePayload {
    pub semantic: wire::Ref<wire::SemanticExports>,
    /// Full typed body inventory is deliberately conservative: generic/const
    /// bodies, defaults and inferred ownership/effects cannot be omitted.
    pub bodies: wire::Ref<wire::HirModule>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectModuleReferences {
    pub module: wire::Ref<wire::Module>,
    pub interface: SemanticInterface<ProjectInterfacePayload>,
    pub checked: CheckedModule<wire::Ref<wire::HirModule>>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectTypedArtifact {
    pub sources: BTreeMap<wire::Ref<wire::SourceFile>, sifr_frontend::persistence::CapturedSource>,
    pub modules: BTreeMap<String, ProjectModuleReferences>,
    pub bytes: Vec<u8>,
}
