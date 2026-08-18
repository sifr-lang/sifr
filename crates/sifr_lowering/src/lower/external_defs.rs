use crate::hir_nodes::{HirExpr, HirParam};
use sifr_ir::{CompilerIntrinsicId, MethodKind};
use sifr_type_system::{FunctionType, ReceiverConvention, Type};

/// Package-neutral method contract retained for imported structural shape descriptions.
#[derive(Debug, Clone)]
pub struct StructuralMethodExport {
    pub name: String,
    pub params: Vec<HirParam>,
    pub return_type: Type,
    pub is_async: bool,
    pub method_kind: MethodKind,
    pub receiver: Option<ReceiverConvention>,
}

pub type StructuralMethodExports = std::collections::HashMap<String, Vec<StructuralMethodExport>>;
type StructuralMethodModules = std::collections::HashMap<String, StructuralMethodExports>;

#[derive(Debug, Clone, Default)]
pub struct ModuleSpecializationMetadata {
    pub class_field_defaults: std::collections::HashMap<String, Vec<(usize, HirExpr)>>,
    pub declaration_metadata: Vec<sifr_ir::TypedDeclarationMetadata>,
    pub declaration_descriptors: Vec<sifr_ir::TypedDeclarationDescriptor>,
    pub class_adapter_providers: Vec<sifr_ir::ClassAdapterProviderDeclaration>,
    pub class_adapter_markers: Vec<sifr_ir::ClassAdapterMarkerDeclaration>,
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
        std::collections::HashMap<String, std::collections::HashMap<String, FunctionType>>,
    /// Map of `module_name` -> (`function_name` -> typed compiler intrinsic ID).
    pub compiler_intrinsics:
        std::collections::HashMap<String, std::collections::HashMap<String, CompilerIntrinsicId>>,
    /// Map of `module_name` -> (`class_name` -> Type)
    pub classes: std::collections::HashMap<String, std::collections::HashMap<String, Type>>,
    /// Map of `module_name` -> (`class_name` -> locally callable instance method names).
    pub class_instance_methods: std::collections::HashMap<
        String,
        std::collections::HashMap<String, std::collections::HashSet<String>>,
    >,
    /// Map of `module_name` -> (`class_name` -> consuming Rust opaque method names).
    pub rust_consuming_methods: std::collections::HashMap<
        String,
        std::collections::HashMap<String, std::collections::HashSet<String>>,
    >,
    /// Map of `module_name` -> Rust-backed opaque class names.
    pub rust_opaque_classes: std::collections::HashMap<String, std::collections::HashSet<String>>,
    /// Map of `module_name` -> Rust-backed opaque value classes with structural mappings.
    pub rust_structural_classes:
        std::collections::HashMap<String, std::collections::HashSet<String>>,
    /// Map of `module_name` -> (`class_name` -> `type_param_names`)
    pub class_type_params:
        std::collections::HashMap<String, std::collections::HashMap<String, Vec<String>>>,
    /// Map of `module_name` -> (`class_name` -> annotated-method-capable contracts).
    ///
    /// These are compiler-internal exports. They preserve declaration details that
    /// `Type::Class::methods` intentionally does not carry.
    structural_methods: Option<Box<StructuralMethodModules>>,
    /// Map of `module_name` -> (`class_name` -> declaration-order field defaults).
    ///
    /// Constructor defaults intentionally remain separate because an explicit constructor is
    /// not an authority for the required/defaulted state of class declarations.
    pub class_field_defaults:
        std::collections::HashMap<String, std::collections::HashMap<String, Vec<(usize, HirExpr)>>>,
    /// Map of module name to typed package-owned declaration metadata.
    pub declaration_metadata:
        std::collections::HashMap<String, Vec<sifr_ir::TypedDeclarationMetadata>>,
    /// Canonical class-adapter provider declarations keyed by module and function.
    pub class_adapter_providers: std::collections::HashMap<
        String,
        std::collections::HashMap<String, sifr_ir::ClassAdapterProviderDeclaration>,
    >,
    /// Erased adapter markers keyed by canonical declaring module and symbol.
    pub class_adapter_markers: std::collections::HashMap<
        String,
        std::collections::HashMap<String, sifr_ir::ClassAdapterMarkerDeclaration>,
    >,
    /// Adapted class selections keyed by declaring module and class symbol.
    pub class_adapter_selections: std::collections::HashMap<
        String,
        std::collections::HashMap<String, sifr_ir::ClassAdapterSelection>,
    >,
    /// Canonical descriptor function declarations keyed by module and function.
    pub descriptor_functions: std::collections::HashMap<
        String,
        std::collections::HashMap<String, sifr_ir::DeclarationDescriptorFunction>,
    >,
    /// Evaluated descriptor uses keyed by the declaring module.
    pub declaration_descriptors:
        std::collections::HashMap<String, Vec<sifr_ir::TypedDeclarationDescriptor>>,
    /// Typed metadata produced by validated early adapters.
    pub applied_adapter_metadata:
        std::collections::HashMap<String, Vec<sifr_ir::AppliedAdapterMetadata>>,
    /// Const-evaluable package function bodies keyed by module and function name.
    pub const_functions:
        std::collections::HashMap<String, std::collections::HashMap<String, sifr_ir::HirFunction>>,
    /// Specialization requests retained for single-file/project result reconstruction.
    pub specialization_requests:
        std::collections::HashMap<String, Vec<sifr_ir::ConstSpecializationRequest>>,
    /// Validated static specialization results keyed by the declaring module.
    pub specialization_outputs:
        std::collections::HashMap<String, Vec<sifr_ir::StaticSpecializationOutput>>,
    pub json_integer_boundary_requests:
        std::collections::HashMap<String, Vec<sifr_ir::JsonIntegerBoundaryRequest>>,
    /// Map of `module_name` -> (`constant_name` -> Type)
    pub constants: std::collections::HashMap<String, std::collections::HashMap<String, Type>>,
    /// Map of `module_name` -> (`constant_name` -> compile-time integer value)
    pub constant_integer_values:
        std::collections::HashMap<String, std::collections::HashMap<String, num_bigint::BigInt>>,
    /// Map of `module_name` -> error class names declared or re-exported by that module.
    pub error_types: std::collections::HashMap<String, std::collections::HashSet<String>>,
    /// Map of `module_name` -> (`owner_name` -> (`type_var_name` -> bounds))
    pub type_param_bounds: std::collections::HashMap<
        String,
        std::collections::HashMap<String, std::collections::HashMap<String, Vec<String>>>,
    >,
    /// Map of `module_name` -> (`function_name` -> `type_var_names`)
    pub generic_functions:
        std::collections::HashMap<String, std::collections::HashMap<String, Vec<String>>>,
    /// Map of `module_name` -> (`callable_name` -> vararg parameter index)
    pub function_varargs:
        std::collections::HashMap<String, std::collections::HashMap<String, usize>>,
    /// Map of `module_name` -> (`callable_name` -> Python declaration parameter kinds)
    pub function_python_call_shapes: std::collections::HashMap<
        String,
        std::collections::HashMap<String, Vec<sifr_ir::PythonParameterKind>>,
    >,
    /// Map of `module_name` -> (`callable_name` -> retained callback parameter indices).
    pub rust_threadsafe_callback_targets:
        std::collections::HashMap<String, std::collections::HashMap<String, Vec<usize>>>,
    /// Map of `module_name` -> (`callable_name` -> workload label)
    pub function_workloads:
        std::collections::HashMap<String, std::collections::HashMap<String, String>>,
    /// Map of `module_name` -> (`callable_name` -> default argument expressions by parameter index)
    pub function_defaults:
        std::collections::HashMap<String, std::collections::HashMap<String, Vec<(usize, HirExpr)>>>,
}

impl ExternalDefs {
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
