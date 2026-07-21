use super::report::BuildStageReport;
use crate::diagnostics::RenderedDiagnostic;
use std::time::Instant;

pub(super) fn measure_stage<T>(
    stages: &mut Vec<BuildStageReport>,
    label: impl Into<String>,
    f: impl FnOnce() -> Result<T, Vec<RenderedDiagnostic>>,
) -> Result<T, Vec<RenderedDiagnostic>> {
    let start = Instant::now();
    let value = f()?;
    stages.push(BuildStageReport::new(label, start.elapsed()));
    Ok(value)
}

pub(super) fn import_closure_label(module_count: usize) -> String {
    format!(
        "Parsing import closure ({})",
        module_count_label(module_count)
    )
}

pub(super) fn module_analysis_label(module_count: usize) -> String {
    format!("Analyzing {}", module_count_label(module_count))
}

fn module_count_label(module_count: usize) -> String {
    if module_count == 1 {
        "1 module".to_string()
    } else {
        format!("{module_count} modules")
    }
}
