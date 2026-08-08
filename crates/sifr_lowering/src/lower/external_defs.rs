use crate::hir_nodes::HirExpr;
use sifr_ir::CompilerIntrinsicId;
use sifr_type_system::{FunctionType, Type};

#[derive(Debug, Clone, Default)]
pub struct ModuleSpecializationMetadata {
    pub class_field_defaults: std::collections::HashMap<String, Vec<(usize, HirExpr)>>,
    pub declaration_metadata: Vec<sifr_ir::TypedDeclarationMetadata>,
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
    /// Map of `module_name` -> (`class_name` -> `type_param_names`)
    pub class_type_params:
        std::collections::HashMap<String, std::collections::HashMap<String, Vec<String>>>,
    /// Map of `module_name` -> (`class_name` -> declaration-order field defaults).
    ///
    /// Constructor defaults intentionally remain separate because an explicit constructor is
    /// not an authority for the required/defaulted state of class declarations.
    pub class_field_defaults:
        std::collections::HashMap<String, std::collections::HashMap<String, Vec<(usize, HirExpr)>>>,
    /// Map of module name to typed package-owned declaration metadata.
    pub declaration_metadata:
        std::collections::HashMap<String, Vec<sifr_ir::TypedDeclarationMetadata>>,
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
    /// Set of class names that are error types (class Foo(Error)) across all modules
    pub error_types: std::collections::HashSet<String>,
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
    #[must_use]
    pub fn take_module_specialization_metadata(
        &mut self,
        module_name: &str,
    ) -> ModuleSpecializationMetadata {
        ModuleSpecializationMetadata {
            class_field_defaults: self
                .class_field_defaults
                .remove(module_name)
                .unwrap_or_default(),
            declaration_metadata: self
                .declaration_metadata
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
