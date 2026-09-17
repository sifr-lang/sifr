use crate::hir_nodes::{HirExpr, HirParam};
use sifr_ir::{CompilerIntrinsicId, MethodKind};
use sifr_type_system::{FunctionType, ReceiverConvention, Type};

/// Package-neutral method contract retained for imported structural shape descriptions.
#[derive(Debug, Clone)]
pub struct StructuralMethodExport {
    /// Checked handler target when this method was exported for an adapted owner.
    pub handler_target: Option<sifr_ir::CallableIdentity>,
    pub name: String,
    pub params: Vec<HirParam>,
    pub return_type: Type,
    pub is_async: bool,
    pub method_kind: MethodKind,
    pub receiver: Option<ReceiverConvention>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attached_api_set_membership_checks_the_stored_canonical_identity() {
        let mut defs = ExternalDefs::default();
        defs.attached_api_sets
            .entry("declared".to_string())
            .or_default()
            .insert(
                "Api".to_string(),
                sifr_ir::AttachedApiSetDeclaration {
                    identity: sifr_ir::AttachedApiSetIdentity {
                        module: "actual".to_string(),
                        symbol: "Api".to_string(),
                    },
                    range: ruff_text_size::TextRange::default(),
                },
            );

        assert!(
            !defs.contains_attached_api_set(&sifr_ir::AttachedApiSetIdentity {
                module: "declared".to_string(),
                symbol: "Api".to_string(),
            })
        );
    }
}

pub type StructuralMethodExports = std::collections::HashMap<String, Vec<StructuralMethodExport>>;
type StructuralMethodModules = super::external_layers::ModuleMap<String, StructuralMethodExports>;

#[derive(Debug, Clone, Default)]
pub struct ModuleSpecializationMetadata {
    pub class_field_defaults: std::collections::HashMap<String, Vec<(usize, HirExpr)>>,
    pub declaration_metadata: Vec<sifr_ir::TypedDeclarationMetadata>,
    pub declaration_descriptors: Vec<sifr_ir::TypedDeclarationDescriptor>,
    pub class_adapter_providers: Vec<sifr_ir::ClassAdapterProviderDeclaration>,
    pub class_adapter_markers: Vec<sifr_ir::ClassAdapterMarkerDeclaration>,
    pub attached_api_sets: Vec<sifr_ir::AttachedApiSetDeclaration>,
    pub attached_apis: Vec<sifr_ir::AttachedApiDeclaration>,
    pub class_adapter_selections: Vec<sifr_ir::ClassAdapterSelection>,
    pub descriptor_functions: Vec<sifr_ir::DeclarationDescriptorFunction>,
    pub applied_adapter_metadata: Vec<sifr_ir::AppliedAdapterMetadata>,
    pub specialization_requests: Vec<sifr_ir::ConstSpecializationRequest>,
    pub specialization_outputs: Vec<sifr_ir::StaticSpecializationOutput>,
    pub json_integer_boundary_requests: Vec<sifr_ir::JsonIntegerBoundaryRequest>,
}

/// External module definitions that can be imported.
#[derive(Debug, Clone, Default)]
pub struct ExternalDefs {
    /// Map of `module_name` -> (`function_name` -> `FunctionType`)
    pub functions:
        super::external_layers::ModuleMap<String, std::collections::HashMap<String, FunctionType>>,
    /// Map of `module_name` -> (`function_name` -> typed compiler intrinsic ID).
    pub compiler_intrinsics: super::external_layers::ModuleMap<
        String,
        std::collections::HashMap<String, CompilerIntrinsicId>,
    >,
    /// Map of `module_name` -> (`class_name` -> Type)
    pub classes: super::external_layers::ModuleMap<String, std::collections::HashMap<String, Type>>,
    /// Map of `module_name` -> (`alias_name` -> (`type_params`, resolved alias type)).
    pub generic_type_aliases: super::external_layers::ModuleMap<
        String,
        std::collections::HashMap<String, (Vec<String>, Type)>,
    >,
    /// Map of `module_name` -> (`class_name` -> locally callable instance method names).
    pub class_instance_methods: super::external_layers::ModuleMap<
        String,
        std::collections::HashMap<String, std::collections::HashSet<String>>,
    >,
    /// Map of `module_name` -> (`class_name` -> consuming Rust opaque method names).
    pub rust_consuming_methods: super::external_layers::ModuleMap<
        String,
        std::collections::HashMap<String, std::collections::HashSet<String>>,
    >,
    /// Map of `module_name` -> Rust-backed opaque class names.
    pub rust_opaque_classes:
        super::external_layers::ModuleMap<String, std::collections::HashSet<String>>,
    /// Map of `module_name` -> Rust-backed opaque value classes with structural mappings.
    pub rust_structural_classes:
        super::external_layers::ModuleMap<String, std::collections::HashSet<String>>,
    /// Map of `module_name` -> (`class_name` -> `type_param_names`)
    pub class_type_params:
        super::external_layers::ModuleMap<String, std::collections::HashMap<String, Vec<String>>>,
    /// Map of `module_name` -> (`class_name` -> annotated-method-capable contracts).
    ///
    /// These are compiler-internal exports. They preserve declaration details that
    /// `Type::Class::methods` intentionally does not carry.
    structural_methods: Option<Box<StructuralMethodModules>>,
    /// Map of `module_name` -> (`class_name` -> declaration-order field defaults).
    ///
    /// Constructor defaults intentionally remain separate because an explicit constructor is
    /// not an authority for the required/defaulted state of class declarations.
    pub class_field_defaults: super::external_layers::ModuleMap<
        String,
        std::collections::HashMap<String, Vec<(usize, HirExpr)>>,
    >,
    /// Map of module name to typed package-owned declaration metadata.
    pub declaration_metadata:
        super::external_layers::ModuleMap<String, Vec<sifr_ir::TypedDeclarationMetadata>>,
    /// Canonical class-adapter provider declarations keyed by module and function.
    pub class_adapter_providers: super::external_layers::ModuleMap<
        String,
        std::collections::HashMap<String, sifr_ir::ClassAdapterProviderDeclaration>,
    >,
    /// Erased adapter markers keyed by canonical declaring module and symbol.
    pub class_adapter_markers: super::external_layers::ModuleMap<
        String,
        std::collections::HashMap<String, sifr_ir::ClassAdapterMarkerDeclaration>,
    >,
    /// Erased attached-API namespaces keyed by canonical declaring module and symbol.
    pub attached_api_sets: super::external_layers::ModuleMap<
        String,
        std::collections::HashMap<String, sifr_ir::AttachedApiSetDeclaration>,
    >,
    /// Checked attached package functions keyed by declaring module and function.
    pub attached_apis: super::external_layers::ModuleMap<
        String,
        std::collections::HashMap<String, sifr_ir::AttachedApiDeclaration>,
    >,
    /// Adapted class selections keyed by declaring module and class symbol.
    pub class_adapter_selections: super::external_layers::ModuleMap<
        String,
        std::collections::HashMap<String, sifr_ir::ClassAdapterSelection>,
    >,
    /// Canonical descriptor function declarations keyed by module and function.
    pub descriptor_functions: super::external_layers::ModuleMap<
        String,
        std::collections::HashMap<String, sifr_ir::DeclarationDescriptorFunction>,
    >,
    /// Evaluated descriptor uses keyed by the declaring module.
    pub declaration_descriptors:
        super::external_layers::ModuleMap<String, Vec<sifr_ir::TypedDeclarationDescriptor>>,
    /// Typed metadata produced by validated early adapters.
    pub applied_adapter_metadata:
        super::external_layers::ModuleMap<String, Vec<sifr_ir::AppliedAdapterMetadata>>,
    /// Const-evaluable package function bodies keyed by module and function name.
    pub const_functions: super::external_layers::ModuleMap<
        String,
        std::collections::HashMap<String, sifr_ir::HirFunction>,
    >,
    /// Specialization requests retained for single-file/project result reconstruction.
    pub specialization_requests:
        super::external_layers::ModuleMap<String, Vec<sifr_ir::ConstSpecializationRequest>>,
    /// Validated static specialization results keyed by the declaring module.
    pub specialization_outputs:
        super::external_layers::ModuleMap<String, Vec<sifr_ir::StaticSpecializationOutput>>,
    pub json_integer_boundary_requests:
        super::external_layers::ModuleMap<String, Vec<sifr_ir::JsonIntegerBoundaryRequest>>,
    /// Map of `module_name` -> (`constant_name` -> Type)
    pub constants:
        super::external_layers::ModuleMap<String, std::collections::HashMap<String, Type>>,
    /// Map of `module_name` -> (`constant_name` -> compile-time integer value)
    pub constant_integer_values: super::external_layers::ModuleMap<
        String,
        std::collections::HashMap<String, num_bigint::BigInt>,
    >,
    /// Map of `module_name` -> error class names declared or re-exported by that module.
    pub error_types: super::external_layers::ModuleMap<String, std::collections::HashSet<String>>,
    /// Map of `module_name` -> (`owner_name` -> (`type_var_name` -> bounds))
    pub type_param_bounds: super::external_layers::ModuleMap<
        String,
        std::collections::HashMap<String, std::collections::HashMap<String, Vec<String>>>,
    >,
    /// Map of `module_name` -> (`function_name` -> `type_var_names`)
    pub generic_functions:
        super::external_layers::ModuleMap<String, std::collections::HashMap<String, Vec<String>>>,
    /// Map of `module_name` -> (`callable_name` -> vararg parameter index)
    pub function_varargs:
        super::external_layers::ModuleMap<String, std::collections::HashMap<String, usize>>,
    /// Map of `module_name` -> (`callable_name` -> Python declaration parameter kinds)
    pub function_python_call_shapes: super::external_layers::ModuleMap<
        String,
        std::collections::HashMap<String, Vec<sifr_ir::PythonParameterKind>>,
    >,
    /// Map of `module_name` -> (`callable_name` -> retained callback parameter indices).
    pub rust_threadsafe_callback_targets:
        super::external_layers::ModuleMap<String, std::collections::HashMap<String, Vec<usize>>>,
    /// Map of `module_name` -> (`callable_name` -> workload label)
    pub function_workloads:
        super::external_layers::ModuleMap<String, std::collections::HashMap<String, String>>,
    /// Map of `module_name` -> (`callable_name` -> default argument expressions by parameter index)
    pub function_defaults: super::external_layers::ModuleMap<
        String,
        std::collections::HashMap<String, Vec<(usize, HirExpr)>>,
    >,
}

impl ExternalDefs {
    /// Invalidate one project module atomically; immutable baseline authority survives.
    pub fn remove_module_overlay(&mut self, module: &str) {
        self.functions.remove(module);
        self.compiler_intrinsics.remove(module);
        self.classes.remove(module);
        self.generic_type_aliases.remove(module);
        self.class_instance_methods.remove(module);
        self.rust_consuming_methods.remove(module);
        self.rust_opaque_classes.remove(module);
        self.rust_structural_classes.remove(module);
        self.class_type_params.remove(module);
        self.class_field_defaults.remove(module);
        self.declaration_metadata.remove(module);
        self.class_adapter_providers.remove(module);
        self.class_adapter_markers.remove(module);
        self.attached_api_sets.remove(module);
        self.attached_apis.remove(module);
        self.class_adapter_selections.remove(module);
        self.descriptor_functions.remove(module);
        self.declaration_descriptors.remove(module);
        self.applied_adapter_metadata.remove(module);
        self.const_functions.remove(module);
        self.specialization_requests.remove(module);
        self.specialization_outputs.remove(module);
        self.json_integer_boundary_requests.remove(module);
        self.constants.remove(module);
        self.constant_integer_values.remove(module);
        self.error_types.remove(module);
        self.type_param_bounds.remove(module);
        self.generic_functions.remove(module);
        self.function_varargs.remove(module);
        self.function_python_call_shapes.remove(module);
        self.rust_threadsafe_callback_targets.remove(module);
        self.function_workloads.remove(module);
        self.function_defaults.remove(module);
        if let Some(methods) = &mut self.structural_methods {
            methods.remove(module);
        }
    }

    /// Freeze source-built baseline modules; clones share immutable definitions.
    pub fn freeze_baseline(&mut self) {
        if let Some(methods) = &mut self.structural_methods {
            methods.freeze();
        }
        self.functions.freeze();
        self.compiler_intrinsics.freeze();
        self.classes.freeze();
        self.generic_type_aliases.freeze();
        self.class_instance_methods.freeze();
        self.rust_consuming_methods.freeze();
        self.rust_opaque_classes.freeze();
        self.rust_structural_classes.freeze();
        self.class_type_params.freeze();
        self.class_field_defaults.freeze();
        self.declaration_metadata.freeze();
        self.class_adapter_providers.freeze();
        self.class_adapter_markers.freeze();
        self.attached_api_sets.freeze();
        self.attached_apis.freeze();
        self.class_adapter_selections.freeze();
        self.descriptor_functions.freeze();
        self.declaration_descriptors.freeze();
        self.applied_adapter_metadata.freeze();
        self.const_functions.freeze();
        self.specialization_requests.freeze();
        self.specialization_outputs.freeze();
        self.json_integer_boundary_requests.freeze();
        self.constants.freeze();
        self.constant_integer_values.freeze();
        self.error_types.freeze();
        self.type_param_bounds.freeze();
        self.generic_functions.freeze();
        self.function_varargs.freeze();
        self.function_python_call_shapes.freeze();
        self.rust_threadsafe_callback_targets.freeze();
        self.function_workloads.freeze();
        self.function_defaults.freeze();
    }

    #[must_use]
    pub fn contains_attached_api_set(&self, identity: &sifr_ir::AttachedApiSetIdentity) -> bool {
        self.attached_api_sets
            .get(&identity.module)
            .and_then(|sets| sets.get(&identity.symbol))
            .is_some_and(|declaration| declaration.identity == *identity)
    }

    pub fn replace_structural_methods(
        &mut self,
        module_name: &str,
        methods: StructuralMethodExports,
    ) {
        if methods.is_empty() {
            let remove_storage = self.structural_methods.as_mut().is_some_and(|modules| {
                modules.remove(module_name);
                modules.is_empty()
            });
            if remove_storage {
                self.structural_methods = None;
            }
        } else {
            self.structural_methods
                .get_or_insert_with(Box::default)
                .insert(module_name.to_string(), methods);
        }
    }

    #[must_use]
    pub fn structural_methods_for(&self, module_name: &str) -> Option<&StructuralMethodExports> {
        self.structural_methods.as_deref()?.get(module_name)
    }

    #[must_use]
    pub fn has_structural_methods(&self) -> bool {
        self.structural_methods.is_some()
    }

    pub fn insert_error_type(&mut self, module_name: &str, class_name: &str) {
        self.error_types
            .entry(module_name.to_string())
            .or_default()
            .insert(class_name.to_string());
    }

    #[must_use]
    pub fn is_error_type(&self, module_name: &str, class_name: &str) -> bool {
        self.error_types
            .get(module_name)
            .is_some_and(|names| names.contains(class_name))
    }

    #[must_use]
    pub fn take_module_specialization_metadata(
        &mut self,
        module_name: &str,
    ) -> ModuleSpecializationMetadata {
        let mut class_adapter_providers = self
            .class_adapter_providers
            .remove(module_name)
            .unwrap_or_default()
            .into_values()
            .collect::<Vec<_>>();
        class_adapter_providers.sort_by(|left, right| left.function.cmp(&right.function));
        let mut descriptor_functions = self
            .descriptor_functions
            .remove(module_name)
            .unwrap_or_default()
            .into_values()
            .collect::<Vec<_>>();
        descriptor_functions.sort_by(|left, right| left.function.cmp(&right.function));
        let mut class_adapter_markers = self
            .class_adapter_markers
            .remove(module_name)
            .unwrap_or_default()
            .into_values()
            .collect::<Vec<_>>();
        class_adapter_markers.sort_by(|left, right| left.symbol.cmp(&right.symbol));
        let mut attached_api_sets = self
            .attached_api_sets
            .remove(module_name)
            .unwrap_or_default()
            .into_values()
            .collect::<Vec<_>>();
        attached_api_sets.sort_by(|left, right| left.identity.symbol.cmp(&right.identity.symbol));
        let mut attached_apis = self
            .attached_apis
            .remove(module_name)
            .unwrap_or_default()
            .into_values()
            .collect::<Vec<_>>();
        attached_apis.sort_by(|left, right| left.function.cmp(&right.function));
        let mut class_adapter_selections = self
            .class_adapter_selections
            .remove(module_name)
            .unwrap_or_default()
            .into_values()
            .collect::<Vec<_>>();
        class_adapter_selections.sort_by(|left, right| left.owner.cmp(&right.owner));
        ModuleSpecializationMetadata {
            class_field_defaults: self
                .class_field_defaults
                .remove(module_name)
                .unwrap_or_default(),
            declaration_metadata: self
                .declaration_metadata
                .remove(module_name)
                .unwrap_or_default(),
            declaration_descriptors: self
                .declaration_descriptors
                .remove(module_name)
                .unwrap_or_default(),
            class_adapter_providers,
            class_adapter_markers,
            attached_api_sets,
            attached_apis,
            class_adapter_selections,
            descriptor_functions,
            applied_adapter_metadata: self
                .applied_adapter_metadata
                .remove(module_name)
                .unwrap_or_default(),
            specialization_requests: self
                .specialization_requests
                .remove(module_name)
                .unwrap_or_default(),
            specialization_outputs: self
                .specialization_outputs
                .remove(module_name)
                .unwrap_or_default(),
            json_integer_boundary_requests: self
                .json_integer_boundary_requests
                .remove(module_name)
                .unwrap_or_default(),
        }
    }
}
