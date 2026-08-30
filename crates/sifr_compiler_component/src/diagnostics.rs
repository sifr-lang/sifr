use crate::{ComponentError, ComponentErrorKind, DiagnosticLifecycle};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum DiagnosticRegistryOwner {
    Compiler,
    Provider { namespace: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticCodeDeclaration {
    pub code: String,
    pub lifecycle: DiagnosticLifecycle,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticRegistry {
    pub owner: DiagnosticRegistryOwner,
    pub declarations: Vec<DiagnosticCodeDeclaration>,
}

impl DiagnosticRegistry {
    #[must_use]
    pub fn compiler() -> Self {
        Self {
            owner: DiagnosticRegistryOwner::Compiler,
            declarations: ComponentErrorKind::ALL
                .into_iter()
                .map(|kind| DiagnosticCodeDeclaration {
                    code: kind.code().to_string(),
                    lifecycle: DiagnosticLifecycle::Active,
                })
                .collect(),
        }
    }

    pub fn validate_with(&self, others: &[Self]) -> Result<(), ComponentError> {
        self.validate_local()?;
        let mut codes = self
            .declarations
            .iter()
            .map(|item| item.code.as_str())
            .collect::<BTreeSet<_>>();
        for other in others {
            other.validate_local()?;
            for declaration in &other.declarations {
                if !codes.insert(declaration.code.as_str()) {
                    return Err(ComponentError::new(
                        ComponentErrorKind::DiagnosticRegistry,
                        format!("duplicate component diagnostic code '{}'", declaration.code),
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn lifecycle_for(&self, code: &str) -> Option<DiagnosticLifecycle> {
        self.declarations
            .iter()
            .find(|declaration| declaration.code == code)
            .map(|declaration| declaration.lifecycle)
    }

    fn validate_local(&self) -> Result<(), ComponentError> {
        let prefix = match &self.owner {
            DiagnosticRegistryOwner::Compiler => "SIFR-COMPONENT-".to_string(),
            DiagnosticRegistryOwner::Provider { namespace } => {
                let compiler_owned = sifr_diagnostics::compiler_diagnostic_namespaces()
                    .any(|compiler_namespace| compiler_namespace == namespace);
                if !valid_namespace(namespace) || compiler_owned {
                    return Err(ComponentError::new(
                        ComponentErrorKind::DiagnosticRegistry,
                        "provider diagnostic namespace is invalid or compiler-owned",
                    ));
                }
                format!("SIFR-{namespace}-")
            }
        };
        let mut local = BTreeSet::new();
        let mut previous = None;
        for declaration in &self.declarations {
            if !valid_code(&declaration.code, &prefix) {
                return Err(ComponentError::new(
                    ComponentErrorKind::DiagnosticRegistry,
                    format!(
                        "diagnostic code '{}' has the wrong namespace",
                        declaration.code
                    ),
                ));
            }
            if !local.insert(declaration.code.as_str()) {
                return Err(ComponentError::new(
                    ComponentErrorKind::DiagnosticRegistry,
                    format!("diagnostic code '{}' is duplicated", declaration.code),
                ));
            }
            if previous.is_some_and(|code: &str| code >= declaration.code.as_str()) {
                return Err(ComponentError::new(
                    ComponentErrorKind::DiagnosticRegistry,
                    "diagnostic declarations must be sorted by code",
                ));
            }
            previous = Some(declaration.code.as_str());
        }
        Ok(())
    }
}

fn valid_namespace(namespace: &str) -> bool {
    !namespace.is_empty()
        && namespace.len() <= 24
        && namespace
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'-')
}

fn valid_code(code: &str, prefix: &str) -> bool {
    code.strip_prefix(prefix)
        .is_some_and(|suffix| suffix.len() == 4 && suffix.bytes().all(|byte| byte.is_ascii_digit()))
}
