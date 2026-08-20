use crate::{ModuleFuncSignatures, StdlibRustSource};
use sifr_type_system::Type;
use std::collections::{HashMap, HashSet};

/// Compiled stdlib information for code generation.
#[derive(Clone, Default)]
pub struct StdlibCode {
    /// Checked Rust source for each stdlib module.
    pub module_rust_code: HashMap<String, StdlibRustSource>,
    /// Exported constants and their generated Rust names by module.
    pub module_constants: HashMap<String, HashMap<String, (Type, String)>>,
    /// Checked function and method signatures by module.
    pub func_signatures: HashMap<String, ModuleFuncSignatures>,
    /// Transitive intrinsic dependencies by module.
    pub transitive_deps: HashMap<String, HashSet<String>>,
    /// Generator functions by module.
    pub generator_functions: HashMap<String, HashSet<String>>,
    /// Generic stdlib class names.
    pub generic_classes: HashSet<String>,
    /// Declared type parameters for each generic stdlib class.
    pub generic_class_params: HashMap<String, Vec<String>>,
    /// Generic stdlib class templates used for concrete inference.
    pub generic_class_templates: HashMap<String, sifr_ir::HirClass>,
    /// Ordered fields for each stdlib class by module.
    pub module_class_fields: HashMap<String, HashMap<String, Vec<(String, Type)>>>,
    /// Checked stdlib classes retained for late project-policy implementations.
    pub module_class_templates: HashMap<String, HashMap<String, sifr_ir::HirClass>>,
}
