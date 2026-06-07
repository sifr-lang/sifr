use crate::hir_nodes::HirExpr;
use sifr_type_system::{FunctionType, Type};

/// External module definitions that can be imported.
#[derive(Debug, Clone, Default)]
pub struct ExternalDefs {
    /// Map of `module_name` -> (`function_name` -> `FunctionType`)
    pub functions:
        std::collections::HashMap<String, std::collections::HashMap<String, FunctionType>>,
    /// Map of `module_name` -> (`class_name` -> Type)
    pub classes: std::collections::HashMap<String, std::collections::HashMap<String, Type>>,
    /// Map of `module_name` -> (`class_name` -> `type_param_names`)
    pub class_type_params:
        std::collections::HashMap<String, std::collections::HashMap<String, Vec<String>>>,
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
    /// Map of `module_name` -> (`callable_name` -> workload label)
    pub function_workloads:
        std::collections::HashMap<String, std::collections::HashMap<String, String>>,
    /// Map of `module_name` -> (`callable_name` -> default argument expressions by parameter index)
    pub function_defaults:
        std::collections::HashMap<String, std::collections::HashMap<String, Vec<(usize, HirExpr)>>>,
}
