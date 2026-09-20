pub(crate) fn resolved_cargo_output(
    plan: &sifr_package::CargoCommandPlan,
) -> Result<std::process::Output, String> {
    let tools = sifr_sysroot::NativeToolchain::resolve_at(&plan.current_dir)?;
    sifr_driver::process_execution::output(
        tools
            .cargo_command()?
            .args(&plan.args)
            .current_dir(&plan.current_dir),
    )
    .map_err(|error| error.to_string())
}

pub(crate) struct Timing(pub(crate) Option<std::time::Instant>);
impl Drop for Timing {
    fn drop(&mut self) {
        if let Some(start) = self.0 {
            use std::io::Write;
            let _ = writeln!(
                std::io::stderr(),
                "[sifr-timing] total_ms={} cache_root={}",
                start.elapsed().as_millis(),
                sifr_driver::cache_storage::root().display()
            );
        }
    }
}

pub(super) fn render_project_cache_report(
    compiler: &sifr_driver::CompilerContext,
    report: &sifr_driver::project_cache::ProjectCacheReport,
    timings: bool,
) {
    use std::io::{self, Write as _};
    crate::trace_artifacts::project_report(compiler, report);
    if timings {
        if let Ok(report) = serde_json::to_string(report) {
            let _ = writeln!(io::stderr(), "[sifr-project-cache] {report}");
        }
        if let Some(metadata) = compiler.metadata_stats() {
            let _ = writeln!(io::stderr(), "[sifr-metadata] {metadata}");
        }
    }
}
