use serde::Serialize;
use sifr_frontend::{
    CacheStatus, DocumentVersion, FrontendContext, FrontendInput, FrontendMode, ModuleId,
    PositionEncoding, ProjectRoot, SourcePath, SourceText,
};
use sifr_syntax::TextPosition;
use std::env;
use std::fs;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::process;
use std::time::Instant;

#[derive(Serialize)]
struct QueryBenchReport {
    scenario: String,
    samples_ms: Vec<f64>,
    cache_hits: u64,
    cache_misses: u64,
    diagnostics_count: usize,
    timed_out: bool,
}

struct QueryBenchState {
    report: QueryBenchReport,
}

impl QueryBenchState {
    fn new(scenario: String) -> Self {
        Self {
            report: QueryBenchReport {
                scenario,
                samples_ms: Vec::new(),
                cache_hits: 0,
                cache_misses: 0,
                diagnostics_count: 0,
                timed_out: false,
            },
        }
    }

    fn record_cache(&mut self, status: CacheStatus) {
        match status {
            CacheStatus::Hit => self.report.cache_hits += 1,
            CacheStatus::Miss => self.report.cache_misses += 1,
        }
    }
}

fn main() {
    if let Err(error) = run() {
        let _ = writeln!(io::stderr(), "{error}");
        process::exit(2);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let scenario = args.next().ok_or_else(|| {
        "usage: frontend_query_bench <scenario> <entrypoint> <iterations>".to_string()
    })?;
    let entrypoint = PathBuf::from(args.next().ok_or_else(|| {
        "usage: frontend_query_bench <scenario> <entrypoint> <iterations>".to_string()
    })?);
    let iterations = args
        .next()
        .ok_or_else(|| {
            "usage: frontend_query_bench <scenario> <entrypoint> <iterations>".to_string()
        })?
        .parse::<usize>()
        .map_err(|error| format!("invalid iteration count: {error}"))?;
    let inner_repetitions = args.next().map_or(Ok(1_usize), |value| {
        value
            .parse::<usize>()
            .map_err(|error| format!("invalid inner repetition count: {error}"))
    })?;

    if iterations == 0 || inner_repetitions == 0 {
        return Err("iteration and inner repetition counts must be greater than zero".to_string());
    }

    let mut state = QueryBenchState::new(scenario.clone());
    for iteration in 0..iterations {
        let started = Instant::now();
        for repetition in 0..inner_repetitions {
            let logical_iteration = iteration
                .checked_mul(inner_repetitions)
                .and_then(|base| base.checked_add(repetition))
                .ok_or_else(|| "iteration count overflowed".to_string())?;
            run_iteration(&scenario, &entrypoint, logical_iteration, &mut state)?;
        }
        let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
        let repetitions = u32::try_from(inner_repetitions)
            .map_err(|_| "inner repetition count exceeded supported range".to_string())?;
        state
            .report
            .samples_ms
            .push(elapsed_ms / f64::from(repetitions));
    }

    serde_json::to_writer(io::stdout(), &state.report)
        .map_err(|error| format!("failed to write query benchmark json: {error}"))?;
    let _ = writeln!(io::stdout());
    Ok(())
}

fn run_iteration(
    scenario: &str,
    entrypoint: &Path,
    iteration: usize,
    state: &mut QueryBenchState,
) -> Result<(), String> {
    match scenario {
        "incremental.unchanged_file_update" => {
            let source = fs::read_to_string(entrypoint)
                .map_err(|error| format!("failed to read entrypoint: {error}"))?;
            let mut context = load_project_context(entrypoint)?;
            let module = module_by_stem(&context, "main")?;
            let cold = context.parse_module(module);
            state.record_cache(cold.metadata().cache_status);
            context
                .update_module_source(
                    module,
                    SourceText::new(source),
                    Some(DocumentVersion::new(iteration_i64(iteration)?)),
                )
                .map_err(render_frontend_errors)?;
            let result = context.parse_module(module);
            state.record_cache(result.metadata().cache_status);
        }
        "incremental.leaf_module_change" => {
            let mut context = load_project_context(entrypoint)?;
            let helper = module_by_stem(&context, "helper")?;
            let changed = format!("def value() -> int:\n    return {}\n", iteration + 10);
            context
                .update_module_source(
                    helper,
                    SourceText::new(changed),
                    Some(DocumentVersion::new(iteration_i64(iteration)?)),
                )
                .map_err(render_frontend_errors)?;
            let result = context.parse_module(helper);
            state.record_cache(result.metadata().cache_status);
        }
        "incremental.imported_module_change" => {
            let mut context = load_project_context(entrypoint)?;
            let api = module_by_stem(&context, "api")?;
            let changed = format!(
                "def public_value() -> int:\n    return {}\n",
                iteration + 20
            );
            context
                .update_module_source(
                    api,
                    SourceText::new(changed),
                    Some(DocumentVersion::new(iteration_i64(iteration)?)),
                )
                .map_err(render_frontend_errors)?;
            let result = context.analysis_for_project();
            state.record_cache(result.metadata().cache_status);
        }
        "incremental.public_api_change" => {
            let mut context = load_project_context(entrypoint)?;
            let api = module_by_stem(&context, "api")?;
            let changed = format!(
                "def public_value() -> int:\n    return {}\n\ndef extra_{}() -> int:\n    return 0\n",
                iteration + 30,
                iteration
            );
            context
                .update_module_source(
                    api,
                    SourceText::new(changed),
                    Some(DocumentVersion::new(iteration_i64(iteration)?)),
                )
                .map_err(render_frontend_errors)?;
            let result = context.diagnostics_for_project();
            state.record_cache(result.metadata().cache_status);
            state.report.diagnostics_count += result.value().diagnostics.len();
        }
        "incremental.failure_recovery" => {
            let original = fs::read_to_string(entrypoint)
                .map_err(|error| format!("failed to read entrypoint: {error}"))?;
            let mut context = load_project_context(entrypoint)?;
            let module = module_by_stem(&context, "main")?;
            context
                .update_module_source(
                    module,
                    SourceText::new("def broken(:\n"),
                    Some(DocumentVersion::new(iteration_i64(iteration)?)),
                )
                .map_err(render_frontend_errors)?;
            let failed = context.diagnostics_for_module(module);
            state.record_cache(failed.metadata().cache_status);
            state.report.diagnostics_count += failed.value().diagnostics.len();
            context
                .update_module_source(
                    module,
                    SourceText::new(original),
                    Some(DocumentVersion::new(iteration_i64(iteration + 10_000)?)),
                )
                .map_err(render_frontend_errors)?;
            let recovered = context.diagnostics_for_module(module);
            state.record_cache(recovered.metadata().cache_status);
            state.report.diagnostics_count += recovered.value().diagnostics.len();
        }
        "interactive.cold_context_load" => {
            let mut context = load_single_context(entrypoint)?;
            let module = context.module_graph().entrypoint;
            let result = context.parse_module(module);
            state.record_cache(result.metadata().cache_status);
        }
        "interactive.warm_diagnostics_query" => {
            let mut context = load_single_context(entrypoint)?;
            let module = context.module_graph().entrypoint;
            let cold = context.diagnostics_for_module(module);
            state.record_cache(cold.metadata().cache_status);
            let warm = context.diagnostics_for_module(module);
            state.record_cache(warm.metadata().cache_status);
            state.report.diagnostics_count += warm.value().diagnostics.len();
        }
        "interactive.unchanged_file_update" => {
            let source = fs::read_to_string(entrypoint)
                .map_err(|error| format!("failed to read entrypoint: {error}"))?;
            let mut context = load_single_context(entrypoint)?;
            let module = context.module_graph().entrypoint;
            let cold = context.parse_module(module);
            state.record_cache(cold.metadata().cache_status);
            context
                .update_module_source(
                    module,
                    SourceText::new(source),
                    Some(DocumentVersion::new(iteration_i64(iteration)?)),
                )
                .map_err(render_frontend_errors)?;
            let warm = context.parse_module(module);
            state.record_cache(warm.metadata().cache_status);
        }
        "interactive.changed_file_invalidation" => {
            let mut context = load_single_context(entrypoint)?;
            let module = context.module_graph().entrypoint;
            let cold = context.analysis_for_module(module);
            state.record_cache(cold.metadata().cache_status);
            let changed = format!("def main():\n    value: int = {}\n", iteration + 1);
            context
                .update_module_source(
                    module,
                    SourceText::new(changed),
                    Some(DocumentVersion::new(iteration_i64(iteration)?)),
                )
                .map_err(render_frontend_errors)?;
            let next = context.analysis_for_module(module);
            state.record_cache(next.metadata().cache_status);
        }
        "interactive.source_map_lookup" => {
            let context = load_single_context(entrypoint)?;
            let graph = context.module_graph();
            let source_map = context.source_map();
            let Some(file) = source_map
                .files
                .iter()
                .find(|file| file.id == graph.modules[0].file)
            else {
                return Err("source map did not expose entrypoint file".to_string());
            };
            let target = TextPosition {
                line: 0,
                character: 0,
            };
            let span = source_map
                .text_position_to_span(file.id, &target, PositionEncoding::UTF8)
                .ok_or_else(|| "source map lookup failed for entrypoint start".to_string())?;
            let round_trip = source_map
                .span_to_text_range(file.id, span, PositionEncoding::UTF8)
                .ok_or_else(|| "source map round trip failed for entrypoint start".to_string())?;
            if round_trip.start != target || round_trip.end != target {
                return Err("source map round trip changed the entrypoint start".to_string());
            }
        }
        other => {
            return Err(format!(
                "unknown frontend query benchmark scenario: {other}"
            ));
        }
    }
    Ok(())
}

fn load_single_context(entrypoint: &Path) -> Result<FrontendContext, String> {
    let source = fs::read_to_string(entrypoint).map_err(|error| {
        format!(
            "failed to read entrypoint '{}': {error}",
            entrypoint.display()
        )
    })?;
    FrontendContext::load_single_file(FrontendInput {
        path: SourcePath::new(entrypoint),
        source: SourceText::new(source),
        mode: FrontendMode::SingleFile,
    })
    .map_err(render_frontend_errors)
}

fn load_project_context(entrypoint: &Path) -> Result<FrontendContext, String> {
    let Some(root) = entrypoint.parent() else {
        return Err("project entrypoint must have a parent directory".to_string());
    };
    let mut provider = sifr_frontend::DiskSourceProvider::new();
    FrontendContext::load_project(
        &ProjectRoot {
            root: SourcePath::new(root),
            entrypoint: SourcePath::new(entrypoint),
        },
        &mut provider,
    )
    .map_err(render_frontend_errors)
}

fn module_by_stem(context: &FrontendContext, stem: &str) -> Result<ModuleId, String> {
    context
        .module_graph()
        .modules
        .iter()
        .find(|module| {
            module
                .canonical_path
                .as_path()
                .file_stem()
                .is_some_and(|candidate| candidate == stem)
        })
        .map(|module| module.id)
        .ok_or_else(|| format!("project benchmark fixture is missing module '{stem}'"))
}

fn iteration_i64(iteration: usize) -> Result<i64, String> {
    i64::try_from(iteration)
        .map_err(|_| "iteration count exceeded document version range".to_string())
}

fn render_frontend_errors(errors: Vec<sifr_diagnostics::RenderedDiagnostic>) -> String {
    errors
        .into_iter()
        .map(|diagnostic| format!("{}: {}", diagnostic.code, diagnostic.message))
        .collect::<Vec<_>>()
        .join("; ")
}
