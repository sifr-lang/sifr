//! The completed checking family is deliberately shallower than typed/editor work.
use super::{
    CanonicalDiagnostic, CapturingSourceProvider, Family, ModuleInputs, ModuleResults, Observation,
    ResolutionResult, SemanticInputs, SourceOutcome, observations_match,
};
use crate::SourceProvider;
use serde::{Deserialize, Serialize};
use sifr_diagnostics::{RenderedDiagnostic, Severity, SourceMap};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompletedCheck {
    pub schema: u32,
    pub result: ModuleResults<(), (), CanonicalDiagnostic>,
    pub displays: BTreeMap<String, String>,
}
impl CompletedCheck {
    pub fn capture(
        entrypoint: &Path,
        semantic_inputs: SemanticInputs,
        capture: &CapturingSourceProvider<'_>,
        diagnostics: &[RenderedDiagnostic],
    ) -> Result<Self, String> {
        if !semantic_inputs.complete()
            || !capture.observations_complete()
            || !deterministic_diagnostics(diagnostics)
        {
            return Err("incomplete semantic inputs or transient diagnostic".into());
        }
        let sources = capture.sources();
        let source = sources
            .iter()
            .find(|source| source.path == entrypoint)
            .ok_or("entrypoint was not captured")?
            .clone();
        let mut names = BTreeMap::new();
        for source in &sources {
            names.insert(source.path.to_string_lossy().to_string(), source.clone());
        }
        // The ordinary project frontend renders logical module names. Bind each
        // observed spelling to one captured source; ambiguity prevents caching.
        for diagnostic in diagnostics {
            for span in diagnostic.spans.iter().chain(
                diagnostic
                    .suggestions
                    .iter()
                    .flat_map(|suggestion| suggestion.edits.iter().map(|edit| &edit.span)),
            ) {
                if let Some(file) = &span.file {
                    if names.contains_key(file) {
                        continue;
                    }
                    if file == "main" && sources.len() == 1 {
                        names.insert(file.clone(), source.clone());
                        continue;
                    }
                    let candidates: Vec<_> = sources
                        .iter()
                        .filter(|source| {
                            source
                                .path
                                .ends_with(format!("{}.sifr", file.replace('.', "/")))
                        })
                        .collect();
                    if candidates.len() != 1 {
                        return Err("ambiguous diagnostic source".into());
                    }
                    names.insert(file.clone(), candidates[0].clone());
                }
            }
        }
        let diagnostics = diagnostics
            .iter()
            .map(|diagnostic| CanonicalDiagnostic::capture(diagnostic, &names))
            .collect::<Result<Vec<_>, _>>()?;
        let displays = names
            .iter()
            .map(|(display, source)| (source.identity(), display.clone()))
            .collect();
        let observations = capture.observations().to_vec();
        if observations
            .iter()
            .any(|item| matches!(item, Observation::Failed { .. }))
        {
            return Err("environmental observation did not complete".into());
        }
        Ok(Self {
            schema: 1,
            displays,
            result: ModuleResults {
                inputs: ModuleInputs {
                    source,
                    observations: observations.clone(),
                    semantic_inputs: semantic_inputs.clone(),
                    imported_interfaces: BTreeMap::new(),
                },
                outcome: if diagnostics.iter().any(CanonicalDiagnostic::is_error) {
                    SourceOutcome::DeterministicErrors
                } else {
                    SourceOutcome::Success
                },
                resolution: Family::Complete(ResolutionResult {
                    observations,
                    sources,
                    semantic_inputs,
                }),
                interface: Family::Pending,
                checked: Family::Pending,
                diagnostics: Family::Complete(diagnostics),
                codegen: Family::Pending,
            },
        })
    }
    pub fn validate(&self, context: &SemanticInputs, provider: &mut dyn SourceProvider) -> bool {
        self.schema == 1
            && self.result.check_complete()
            && &self.result.inputs.semantic_inputs == context
            && self.result.resolution.ready().is_some_and(|resolution| {
                resolution.observations == self.result.inputs.observations
                    && resolution.sources.iter().all(|source| {
                        resolution.observations.iter().any(|observation| {
                            matches!(observation, Observation::File { path, identity }
                            if path == &source.path && identity == &source.identity())
                        })
                    })
            })
            && observations_match(&self.result.inputs.observations, provider)
    }
    /// Restore the canonical diagnostic query only. No typed module, flow graph,
    /// analysis index or executable readiness is inferred from this family.
    pub fn diagnostics(&self) -> Result<Vec<RenderedDiagnostic>, String> {
        if !self.result.check_complete() {
            return Err("checking family is incomplete".into());
        }
        let resolution = self.result.resolution.ready().ok_or("missing resolution")?;
        let sources = resolution
            .sources
            .iter()
            .map(|source| {
                let identity = source.identity();
                let display = self
                    .displays
                    .get(&identity)
                    .cloned()
                    .unwrap_or_else(|| source.path.display().to_string());
                (identity, (source.clone(), display))
            })
            .collect();

        let diagnostics = self
            .result
            .diagnostics
            .ready()
            .ok_or("missing diagnostics")?
            .iter()
            .map(|diagnostic| diagnostic.render(&sources, &mut SourceMap::default()))
            .collect::<Result<Vec<_>, _>>()?;
        let outcome = if diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
        {
            SourceOutcome::DeterministicErrors
        } else {
            SourceOutcome::Success
        };
        if outcome != self.result.outcome || !deterministic_diagnostics(&diagnostics) {
            return Err("invalid completed source outcome".into());
        }
        Ok(diagnostics)
    }
}
fn deterministic_diagnostics(diagnostics: &[RenderedDiagnostic]) -> bool {
    // Positive source-language families only: environment, provider, internal and
    // native failures must execute again, even when a prior failure was complete.
    diagnostics.iter().all(|diagnostic| {
        [
            "SIFR-PARSE-",
            "SIFR-NAME-",
            "SIFR-IMPORT-",
            "SIFR-TYPE-",
            "SIFR-OWNERSHIP-",
            "SIFR-FLOW-",
            "SIFR-EFFECT-",
            "SIFR-ASYNC-",
        ]
        .iter()
        .any(|prefix| diagnostic.code.starts_with(prefix))
    })
}
