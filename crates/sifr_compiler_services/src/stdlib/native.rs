use super::SourceStdlibCompiled;
use super::bootstrap::{CheckedModuleObserver, check_stdlib_sources, signature_params};
use super::interop::{
    PendingStdlibInteropModule, build_stdlib_rust_interop, pending_private_interop_module,
};
use crate::diagnostics::{RenderedDiagnostic, run_codegen_with_boundary};
use crate::export_policy::should_export_callable;
use sifr_diagnostics::DiagnosticCode;
use sifr_stdlib_artifact::{StdlibCode, StdlibRustSource};
use sifr_stdlib_manifest::{LoadedStdlibSource, LoadedStdlibSourceKind, StdlibFeature};
use sifr_sysroot::{ResolvedSysroot, sha256_hex};
use sifr_type_system::{ParamConvention, Type};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;

pub fn compile_stdlib_sources_with_sysroot(
    sources: &[LoadedStdlibSource],
    sysroot: ResolvedSysroot,
) -> Result<SourceStdlibCompiled, Vec<RenderedDiagnostic>> {
    let mut native = NativeStdlibObserver {
        sysroot,
        code: StdlibCode::default(),
        metadata_features: HashMap::new(),
        private_interop_modules: Vec::new(),
        syntax_session: sifr_codegen::StdlibSyntaxSession::default(),
    };
    let checked = check_stdlib_sources(sources, &mut native)?;
    debug_assert_eq!(
        native.code.transitive_deps, checked.transitive_deps,
        "native observer must retain the canonical checked dependency map"
    );
    native.code.hir_modules = Arc::new(checked.hir_modules);
    native.code.transitive_deps = checked.transitive_deps;
    Ok(SourceStdlibCompiled {
        defs: checked.defs,
        metadata_features: native.metadata_features,
        code: native.code,
        interop: build_stdlib_rust_interop(Some(native.sysroot), &native.private_interop_modules),
    })
}

struct NativeStdlibObserver<'a> {
    sysroot: ResolvedSysroot,
    code: StdlibCode,
    metadata_features: HashMap<String, BTreeSet<StdlibFeature>>,
    private_interop_modules: Vec<PendingStdlibInteropModule<'a>>,
    syntax_session: sifr_codegen::StdlibSyntaxSession,
}

impl<'a> CheckedModuleObserver<'a> for NativeStdlibObserver<'a> {
    fn retain_declaration(
        &mut self,
        source: &'a LoadedStdlibSource,
        module: &Arc<sifr_ir::HirModule>,
    ) {
        if let Some(pending) = pending_private_interop_module(source, module) {
            self.private_interop_modules.push(pending);
        }
    }

    fn materialize(
        &mut self,
        stdlib_source: &'a LoadedStdlibSource,
        module: &sifr_ir::HirModule,
        transitive_deps: &HashSet<String>,
    ) -> Result<(), Vec<RenderedDiagnostic>> {
        let module_name = stdlib_source.module.as_str();
        let private_declaration = stdlib_source.kind == LoadedStdlibSourceKind::PrivateDeclaration;
        let has_pure_sifr_code = !module.functions.is_empty()
            || !module.constants.is_empty()
            || !module.classes.is_empty();
        if has_pure_sifr_code {
            let codegen_result = run_codegen_with_boundary(
                format!(
                    "internal compiler panic during stdlib code generation for '{module_name}'"
                ),
                || {
                    self.syntax_session
                        .generate_module(module, &self.code.emission, module_name)
                },
            )
            .map_err(|e| {
                let mut diagnostic = *e;
                diagnostic.message = format!("[stdlib:{module_name}] {}", diagnostic.message);
                vec![diagnostic]
            })?;
            self.metadata_features.insert(
                module_name.to_owned(),
                codegen_result.required_features.iter().copied().collect(),
            );
            let rust_source = stdlib_rust_source(
                module_name,
                stdlib_source,
                &self.sysroot,
                module
                    .classes
                    .iter()
                    .filter(|class| {
                        !class
                            .rust_interop
                            .iter()
                            .any(|declaration| declaration.abi_requirements.opaque_handle)
                    })
                    .map(|class| class.name.clone())
                    .collect(),
                codegen_result.rust_source,
            )?;
            self.code
                .module_rust_code
                .insert(module_name.to_string(), rust_source);
            if !codegen_result.constant_mappings.is_empty() {
                self.code
                    .module_constants
                    .insert(module_name.to_string(), codegen_result.constant_mappings);
            }
            let mut sig_map = HashMap::new();
            for func in &module.functions {
                if private_declaration || should_export_callable(module_name, &func.name) {
                    let param_info = signature_params(&func.params, None);
                    sig_map.insert(func.name.clone(), (param_info, func.return_type.clone()));
                }
            }
            for class in &module.classes {
                let mut has_constructor = false;
                for method in &class.methods {
                    let param_info = signature_params(
                        &method.params,
                        (method.name == "new").then_some(ParamConvention::own()),
                    );
                    sig_map.insert(
                        format!("{}::{}", class.name, method.name),
                        (param_info, method.return_type.clone()),
                    );
                    if method.name == "new" {
                        has_constructor = true;
                    }
                }
                if !has_constructor {
                    let ctor_params = class
                        .fields
                        .iter()
                        .map(|(_, ty)| (ty.clone(), ParamConvention::own()))
                        .collect::<Vec<_>>();
                    sig_map.insert(
                        format!("{}::new", class.name),
                        (
                            ctor_params,
                            Type::Class {
                                identity: None,
                                type_args: class
                                    .type_params
                                    .iter()
                                    .cloned()
                                    .map(Type::TypeVar)
                                    .collect(),
                                name: class.name.clone(),
                                fields: class.fields.clone().into(),
                                methods: Vec::new().into(),
                                parent_class: class.semantic_parent_chain(),
                            },
                        ),
                    );
                }
            }
            if !sig_map.is_empty() {
                self.code
                    .func_signatures
                    .insert(module_name.to_string(), sig_map);
            }

            let mut gen_fns = HashSet::new();
            for func in &module.functions {
                if (private_declaration || should_export_callable(module_name, &func.name))
                    && sifr_codegen::body_contains_yield(&func.body)
                {
                    gen_fns.insert(func.name.clone());
                }
            }
            if !gen_fns.is_empty() {
                self.code
                    .generator_functions
                    .insert(module_name.to_string(), gen_fns);
            }

            for class in &module.classes {
                if !class.type_params.is_empty() {
                    self.code.generic_classes.insert(class.name.clone());
                    self.code
                        .generic_class_params
                        .insert(class.name.clone(), class.type_params.clone());
                    self.code
                        .generic_class_templates
                        .insert(class.name.clone(), std::sync::Arc::new(class.clone()));
                }
            }
            let class_fields = module
                .classes
                .iter()
                .map(|class| (class.name.clone(), class.fields.clone()))
                .collect();
            self.code
                .module_class_fields
                .insert(module_name.to_string(), class_fields);
            let class_templates = module
                .classes
                .iter()
                .map(|class| {
                    (
                        class.name.clone(),
                        stdlib_class_template(module_name, class),
                    )
                })
                .collect();
            self.code
                .module_class_templates
                .insert(module_name.to_string(), class_templates);
        }

        // Emission sees only earlier successful modules. The checked loop owns
        // dependency discovery; retain this module's facts for later rendering.
        if !transitive_deps.is_empty() {
            self.code
                .transitive_deps
                .insert(module_name.to_owned(), transitive_deps.clone());
        }
        Ok(())
    }
}

fn stdlib_rust_source(
    module_name: &str,
    source: &LoadedStdlibSource,
    sysroot: &ResolvedSysroot,
    nominal_types: HashSet<String>,
    rust: String,
) -> Result<StdlibRustSource, Vec<RenderedDiagnostic>> {
    Ok(StdlibRustSource {
        module: module_name.to_string(),
        source_path: canonical_stdlib_source_path(source, sysroot)?,
        source_sha256: source_sha256(&source.source),
        nominal_types,
        rust,
    })
}

fn canonical_stdlib_source_path(
    source: &LoadedStdlibSource,
    sysroot: &ResolvedSysroot,
) -> Result<String, Vec<RenderedDiagnostic>> {
    let relative = source
        .path
        .strip_prefix(&sysroot.paths.stdlib_root)
        .map_err(|_| {
            vec![crate::diagnostics::diagnostic_with_code(
                format!(
                    "stdlib source path {} is outside resolved stdlib root {}",
                    source.path.display(),
                    sysroot.paths.stdlib_root.display()
                ),
                DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE,
            )]
        });
    relative.map(|path| format!("stdlib/{}", normalized_path_string(path)))
}

fn source_sha256(source: &str) -> String {
    sha256_hex(source.as_bytes())
}

fn normalized_path_string(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

pub fn stdlib_class_template(module_name: &str, class: &sifr_ir::HirClass) -> sifr_ir::HirClass {
    // Structural emission needs every declaration field but never method
    // statements. Project directly so bootstrap does not clone and discard
    // entire bodies or retain their empty allocation in its lifetime cache.
    sifr_ir::HirClass {
        name: class.name.clone(),
        identity: Some(format!("{module_name}.{}", class.name)),
        fields: class.fields.clone(),
        field_defaults: class.field_defaults.clone(),
        field_default_identities: class.field_default_identities.clone(),
        declaration_metadata: class.declaration_metadata.clone(),
        methods: class
            .methods
            .iter()
            .map(structural_method_template)
            .collect(),
        is_hashable: class.is_hashable,
        is_error_type: class.is_error_type,
        kind: class.kind.clone(),
        operator_impls: class
            .operator_impls
            .iter()
            .map(|(name, method)| (name.clone(), structural_method_template(method)))
            .collect(),
        newtype_inner: class.newtype_inner.clone(),
        implements_protocols: class.implements_protocols.clone(),
        parent_class: class.parent_class.clone(),
        parent_type: class.parent_type.clone(),
        type_params: class.type_params.clone(),
        enum_variants: class.enum_variants.clone(),
        rust_interop: class.rust_interop.clone(),
    }
}

fn structural_method_template(method: &sifr_ir::HirFunction) -> sifr_ir::HirFunction {
    sifr_ir::HirFunction {
        name: method.name.clone(),
        params: method.params.clone(),
        return_type: method.return_type.clone(),
        body: Vec::new(),
        is_async: method.is_async,
        method_kind: method.method_kind,
        receiver: method.receiver,
        decorators: method.decorators.clone(),
        rust_interop: method.rust_interop.clone(),
        python_interop: method.python_interop.clone(),
        compiler_intrinsic: method.compiler_intrinsic,
        type_params: method.type_params.clone(),
    }
}
