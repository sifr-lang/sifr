use super::project_codegen::{GeneratedBinaryProject, generated_single_file_binary_project};
use super::rust_interop::PackageRustInteropContext;
use super::rust_interop_resolution::resolve_package_rust_interop_metadata;
use super::sysroot_interop::attach_stdlib_rust_interop;
use crate::diagnostics::RenderedDiagnostic;
use crate::stdlib::StdlibRustInterop;
use sifr_codegen::{CodegenResult, InteropBuildPlan, LoweringStats};
use sifr_stdlib_manifest::StdlibFeature;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, OnceLock};

pub(crate) struct CompiledSingleFileMetadata {
    pub(crate) rust_source: String,
    pub(crate) used_stdlib_modules: HashSet<String>,
    pub(crate) required_features: HashSet<StdlibFeature>,
    pub(crate) interop: InteropBuildPlan,
    pub(crate) lowering_stats: LoweringStats,
}

type ResolvedInterop = Result<InteropBuildPlan, Vec<RenderedDiagnostic>>;
type ResolvedInteropCell = Arc<OnceLock<ResolvedInterop>>;

static RESOLVED_STDLIB_INTEROP: OnceLock<Mutex<HashMap<String, ResolvedInteropCell>>> =
    OnceLock::new();

pub(crate) fn resolve_single_file_metadata(
    codegen_result: CodegenResult,
    rust_interop_context: Option<PackageRustInteropContext>,
    stdlib_interop: &StdlibRustInterop,
) -> Result<CompiledSingleFileMetadata, Vec<RenderedDiagnostic>> {
    let lowering_stats = codegen_result.lowering_stats;
    let rust_source = codegen_result.rust_source.clone();
    let used_stdlib_modules = codegen_result.used_stdlib_modules.clone();
    let required_features = codegen_result.required_features.clone();

    let interop = if codegen_result.interop == InteropBuildPlan::default() {
        resolve_cached_stdlib_interop(codegen_result, rust_interop_context, stdlib_interop)?
    } else {
        resolve_interop(codegen_result, rust_interop_context, stdlib_interop)?
    };

    Ok(CompiledSingleFileMetadata {
        rust_source,
        used_stdlib_modules,
        required_features,
        interop,
        lowering_stats,
    })
}

fn resolve_cached_stdlib_interop(
    codegen_result: CodegenResult,
    rust_interop_context: Option<PackageRustInteropContext>,
    stdlib_interop: &StdlibRustInterop,
) -> ResolvedInterop {
    let key = stdlib_interop_cache_key(stdlib_interop);
    let cell = {
        let cache = RESOLVED_STDLIB_INTEROP.get_or_init(|| Mutex::new(HashMap::new()));
        let mut entries = cache
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Arc::clone(
            entries
                .entry(key)
                .or_insert_with(|| Arc::new(OnceLock::new())),
        )
    };
    cell.get_or_init(|| resolve_interop(codegen_result, rust_interop_context, stdlib_interop))
        .clone()
}

fn resolve_interop(
    codegen_result: CodegenResult,
    rust_interop_context: Option<PackageRustInteropContext>,
    stdlib_interop: &StdlibRustInterop,
) -> ResolvedInterop {
    let generated = generated_single_file_binary_project(codegen_result)?;
    let (generated, rust_interop_context) =
        attach_stdlib_rust_interop(generated, rust_interop_context, stdlib_interop);
    resolve_package_rust_interop_metadata(generated, rust_interop_context)
        .map(|generated: GeneratedBinaryProject| generated.interop)
}

fn stdlib_interop_cache_key(stdlib_interop: &StdlibRustInterop) -> String {
    let sysroot_identity = stdlib_interop.sysroot.as_ref().map_or_else(
        || "<no-sysroot>".to_string(),
        |sysroot| {
            format!(
                "root={}\ntoolchain={}\ncontent={}",
                sysroot.root.display(),
                sysroot.toolchain_id(),
                sysroot.manifest.sysroot_content_sha256
            )
        },
    );
    format!(
        "{sysroot_identity}\n{}",
        stdlib_interop.plan.cache_key_fragment()
    )
}
