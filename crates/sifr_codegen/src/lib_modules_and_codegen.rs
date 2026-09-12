use super::{ModuleSupportDemand, RustEmitter};
use crate::StdlibCode;
use crate::stdlib_import_signatures::register_imported_stdlib_signature;
use sifr_ir::HirModule;
use sifr_stdlib_manifest::StdlibFeature;
use sifr_type_system::{ParamConvention, Type};
use std::collections::{HashMap, HashSet};

mod deferred_codegen;
mod project_imports;
use deferred_codegen::{deferred_codegen_result, inline_codegen_result};
#[path = "lib_modules_structural_policy.rs"]
mod structural_policy;
pub(crate) use structural_policy::generate_rust_with_stdlib_for_module_with_structural_policy;

pub(crate) type FuncSignature = (Vec<(Type, ParamConvention)>, Type);
pub(crate) type ModuleFuncSignatures = HashMap<String, FuncSignature>;
pub(crate) type UnionVariantTypes = Vec<(String, Type)>;
pub(crate) type IsinstanceUnionMatch = (String, String, String, UnionVariantTypes);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProjectStructuralLayoutLocation {
    Local,
    CrateRoot,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SupportEmission {
    Inline,
    Deferred,
}

pub use crate::entrypoints::{generate_rust, generate_rust_test, generate_rust_with_metadata};

#[derive(Clone)]
pub(crate) struct NestedFnCapture {
    pub(crate) name: String,
    pub(crate) ty: Type,
    pub(crate) convention: ParamConvention,
}

/// Result of code generation, including the Rust source and metadata.
pub struct CodegenResult {
    pub rust_source: String,
    /// The module body without compiler-owned runtime or stdlib support.
    pub(crate) module_body_source: String,
    /// Compiler-verified package outputs that must be materialized as static data.
    pub static_programs: Vec<sifr_ir::StaticSpecializationOutput>,
    /// Static-program owners proven eligible for every required structural implementation.
    pub static_program_structural_owners: std::collections::BTreeSet<String>,
    pub used_stdlib_modules: HashSet<String>,
    pub used_intrinsic_modules: HashSet<String>,
    /// Required stdlib/runtime features discovered during structured lowering/codegen.
    pub required_features: HashSet<StdlibFeature>,
    /// Structured interop metadata required before generated project materialization.
    pub interop: crate::InteropBuildPlan,
    /// Map of `constant_name` -> (type, `rust_name`) for module-level constants
    pub constant_mappings: HashMap<String, (Type, String)>,
    /// Counters for structured lowering usage during emission.
    pub lowering_stats: LoweringStats,
    /// Canonical support demand retained for project-level single-owner assembly.
    pub(crate) support_demand: ModuleSupportDemand,
}

/// Result of multi-module code generation, including aggregate dependency metadata.
pub struct MultiModuleCodegenResult {
    pub rust_files: HashMap<String, String>,
    /// The single crate-root prelude that owns all non-optional project union enums.
    pub project_union_prelude: String,
    pub used_stdlib_modules: HashSet<String>,
    pub required_features: HashSet<StdlibFeature>,
    /// Structured interop metadata required before generated project materialization.
    pub interop: crate::InteropBuildPlan,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LoweringStats {
    pub stmt_total: u64,
    pub stmt_structured: u64,
    pub stmt_lowering_errors: u64,
    pub expr_total: u64,
    pub expr_structured: u64,
    pub expr_lowering_errors: u64,
    pub item_lowering_errors: u64,
    pub stmt_candidate_total: u64,
    pub stmt_candidate_structured: u64,
    pub expr_candidate_total: u64,
    pub expr_candidate_structured: u64,
}

/// Generate Rust source code from a HIR module with compiled stdlib code.
pub fn generate_rust_with_stdlib(module: &HirModule, stdlib_code: &StdlibCode) -> CodegenResult {
    generate_rust_with_stdlib_for_module(module, stdlib_code, None)
}

/// Generate Rust source code from a named HIR module with compiled stdlib code.
pub fn generate_rust_with_stdlib_for_module(
    module: &HirModule,
    stdlib_code: &StdlibCode,
    module_name: Option<&str>,
) -> CodegenResult {
    generate_rust_with_stdlib_for_module_with_structural_policy(
        module,
        stdlib_code,
        module_name,
        crate::rust_interop_plan::module_uses_structural_interop(module),
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn generate_rust_with_stdlib_for_module_with_project_policy(
    module: &HirModule,
    stdlib_code: &StdlibCode,
    module_name: Option<&str>,
    structural_identity_module_name: Option<&str>,
    structural_interop_enabled: bool,
    owned_union_enums: Option<&HashSet<String>>,
    project_ordinary_union_enums: Option<&HashSet<String>>,
    project_try_error_carrier_enums: Option<&HashSet<String>>,
    project_structural_record_identities: Option<&HashSet<String>>,
    project_structural_layout_location: ProjectStructuralLayoutLocation,
    project_structural_identity_expressions: Option<&HashMap<String, String>>,
    support_emission: SupportEmission,
) -> CodegenResult {
    let mut emitter = RustEmitter::new();
    emitter.structural_interop_enabled = structural_interop_enabled;
    emitter.project_structural_record_identities = project_structural_record_identities.cloned();
    emitter.project_structural_identity_expressions =
        project_structural_identity_expressions.cloned();
    emitter.structural_identity_module_name = structural_identity_module_name.map(str::to_string);
    // Register stdlib generic classes so user code skips explicit type annotations
    emitter
        .generic_classes
        .extend(stdlib_code.generic_classes.iter().cloned());
    emitter
        .generic_class_params
        .extend(stdlib_code.generic_class_params.clone());
    emitter
        .generic_class_templates
        .extend(stdlib_code.generic_class_templates.clone());

    // Pre-register imported constants and function signatures so user code can reference them correctly.
    crate::project_constants::register_imported_constants(&mut emitter, module, stdlib_code);
    for import in &module.imports {
        if let Some(sig_map) = stdlib_code.func_signatures.get(&import.module) {
            for name in &import.names {
                register_imported_stdlib_signature(&mut emitter, stdlib_code, import, name);
                // Also load class method signatures (ClassName::method entries)
                let prefix = format!("{name}::");
                for (key, sig) in sig_map {
                    if let Some(method) = key.strip_prefix(&prefix) {
                        let local_name = import
                            .aliases
                            .iter()
                            .find(|(original, _)| original == name)
                            .map_or(name.as_str(), |(_, alias)| alias.as_str());
                        emitter
                            .func_signatures
                            .insert(format!("{local_name}::{method}"), sig.clone());
                    }
                }
            }
            // Load class method signatures for classes returned by imported functions.
            // This handles cases like `compile_flags` returning `Pattern` - we need
            // `Pattern::search` etc. to be available for correct borrow prefix emission.
            for (key, sig) in sig_map {
                if key.contains("::") && !emitter.func_signatures.contains_key(key) {
                    emitter.func_signatures.insert(key.clone(), sig.clone());
                }
            }
        } else {
            for name in &import.names {
                register_imported_stdlib_signature(&mut emitter, stdlib_code, import, name);
            }
        }
        if let Some(class_fields) = stdlib_code.module_class_fields.get(&import.module) {
            for name in &import.names {
                if let Some(fields) = class_fields.get(name) {
                    let local_name = import
                        .aliases
                        .iter()
                        .find(|(original, _)| original == name)
                        .map(|(_, alias)| alias.as_str())
                        .unwrap_or(name);
                    emitter.register_external_class_fields(local_name, name, fields);
                }
            }
        }
        // Pre-register stdlib generator functions so .collect() is emitted at call sites
        if let Some(gen_set) = stdlib_code.generator_functions.get(&import.module) {
            for name in &import.names {
                if gen_set.contains(name) {
                    emitter.generator_functions.insert(name.clone());
                }
            }
        }
    }

    // First pass: collect all union types used in the module
    emitter.collect_union_types(module);
    crate::lib_project_codegen::register_imported_union_types(&mut emitter, module, stdlib_code);
    if let Some(project_ordinary_union_enums) = project_ordinary_union_enums {
        emitter
            .ordinary_union_enums
            .extend(project_ordinary_union_enums.iter().cloned());
    }
    if let Some(project_try_error_carrier_enums) = project_try_error_carrier_enums {
        emitter
            .try_error_carrier_enums
            .extend(project_try_error_carrier_enums.iter().cloned());
    }
    if structural_interop_enabled && owned_union_enums.is_none() {
        emitter.structural_union_enums.extend(
            crate::structural_impl_codegen::structural_union_names(module, &emitter.union_enums),
        );
    }
    if let Some(owned_union_enums) = owned_union_enums {
        emitter.suppressed_union_enum_definitions = emitter
            .union_enums
            .keys()
            .filter(|name| !owned_union_enums.contains(*name))
            .cloned()
            .collect();
    }
    // Detect recursive (self-referential) class fields that need Box<T>
    emitter.detect_recursive_fields(module);

    // Generate enum definitions for non-Option union types
    emitter.generate_enum_definitions();
    if owned_union_enums.is_none() {
        emitter.generate_structural_record_definitions();
    }

    // Second pass: emit the actual code
    emitter.emit_named_module(module, false, false, module_name);
    emitter.emit_imported_stdlib_structural_impls(module, stdlib_code);
    // Expression lowering can introduce canonical intermediate error unions.
    if let Some(owned_union_enums) = owned_union_enums {
        emitter.suppressed_union_enum_definitions.extend(
            emitter
                .union_enums
                .keys()
                .filter(|name| !owned_union_enums.contains(*name))
                .cloned(),
        );
    }
    emitter.generate_enum_definitions();
    let support_demand = ModuleSupportDemand::from_emitter(module, &emitter, module_name);
    if support_emission == SupportEmission::Deferred {
        return deferred_codegen_result(
            module,
            stdlib_code,
            emitter,
            support_demand,
            project_structural_layout_location,
            project_structural_record_identities.is_some(),
        );
    }
    inline_codegen_result(
        module,
        stdlib_code,
        emitter,
        support_demand,
        project_structural_layout_location,
        project_structural_record_identities.is_some(),
    )
}
