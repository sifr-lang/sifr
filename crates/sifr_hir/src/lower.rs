//! AST to HIR lowering with type checking and name resolution.

use sifr_python_ast::*;
use sifr_type_system::{
    Type, FunctionType, OwnershipKind, ParamConvention,
    type_check_binary_op, type_check_unary_op, type_check_comparison, type_check_bool_op,
    make_union, NarrowingCondition, narrow_type,
};
use sifr_type_system::infer::resolve_type_annotation;
use crate::hir_nodes::*;
use crate::scope::Scope;
use std::collections::HashMap;

/// Errors produced during lowering.
#[derive(Debug, Clone)]
pub struct LoweringError {
    pub message: String,
    pub line: Option<u32>,
    pub col: Option<u32>,
}

impl std::fmt::Display for LoweringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let (Some(line), Some(col)) = (self.line, self.col) {
            write!(f, "{}:{}: {}", line, col, self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

/// The lowering context that tracks state during AST->HIR conversion.
struct LowerCtx {
    /// Function signatures (name -> type)
    functions: HashMap<String, FunctionType>,
    /// Default parameter values for functions (name -> vec of (param_index, default_expr))
    function_defaults: HashMap<String, Vec<(usize, HirExpr)>>,
    /// Class type definitions (name -> Type::Class)
    class_types: HashMap<String, Type>,
    /// Current scope for name resolution
    scope: Scope,
    /// Collected errors
    errors: Vec<LoweringError>,
    /// Loop nesting depth (for break/continue validation)
    loop_depth: usize,
    /// reveal_type() diagnostics (informational, not errors)
    reveal_types: Vec<String>,
    /// Compiler warnings (non-fatal diagnostics printed to stderr)
    warnings: Vec<String>,
    /// Whether we're currently inside a class method (tracks `self` type)
    current_class: Option<String>,
    /// The parent class name of the current class (for super() resolution)
    current_parent_class: Option<String>,
    /// Whether we're inside a try block (auto-unwrap Result values)
    in_try_block: bool,
    /// Error types collected from Result-returning calls during try body lowering.
    /// Each entry is the name of an error class encountered via auto-unwrap in the current try block.
    try_block_error_types: std::collections::HashSet<String>,
    /// Set of class names that are error types (class Foo(Error))
    error_types: std::collections::HashSet<String>,
    /// Map of parent error type -> list of known child error types (for exhaustiveness checking)
    error_hierarchy: HashMap<String, Vec<String>>,
    /// Set of function names that have *args (vararg) parameters
    vararg_functions: std::collections::HashSet<String>,
    /// Set of registered type variable names (e.g., T, K, V from TypeVar declarations)
    type_vars: std::collections::HashSet<String>,
    /// Map of generic function names to their type variable names
    generic_functions: HashMap<String, Vec<String>>,
    /// Map of owner (function or class name) -> (type_var_name -> protocol bounds)
    type_param_bounds: HashMap<String, HashMap<String, Vec<String>>>,
    /// Whether _sifr.* intrinsic imports are allowed (true for stdlib .sifr files)
    allow_intrinsic_imports: bool,
    /// Set of parameter names that are immutably borrowed (&T) in the current function.
    /// Used for escape analysis: returning or storing a borrowed param is a compile error.
    borrowed_params: std::collections::HashSet<String>,
    /// Map of class names to their declared type parameters (from PEP 695 class C[T])
    class_declared_type_params: HashMap<String, Vec<String>>,
}

impl LowerCtx {
    fn new() -> Self {
        Self {
            functions: HashMap::new(),
            function_defaults: HashMap::new(),
            class_types: HashMap::new(),
            scope: Scope::new(),
            errors: Vec::new(),
            loop_depth: 0,
            reveal_types: Vec::new(),
            warnings: Vec::new(),
            current_class: None,
            current_parent_class: None,
            in_try_block: false,
            try_block_error_types: std::collections::HashSet::new(),
            error_types: std::collections::HashSet::new(),
            error_hierarchy: HashMap::new(),
            vararg_functions: std::collections::HashSet::new(),
            type_vars: std::collections::HashSet::new(),
            generic_functions: HashMap::new(),
            type_param_bounds: HashMap::new(),
            allow_intrinsic_imports: false,
            borrowed_params: std::collections::HashSet::new(),
            class_declared_type_params: HashMap::new(),
        }
    }

    fn warn(&mut self, message: String) {
        self.warnings.push(message);
    }

    fn error(&mut self, message: String) {
        self.errors.push(LoweringError {
            message,
            line: None,
            col: None,
        });
    }

    fn in_loop(&self) -> bool {
        self.loop_depth > 0
    }
}

/// Collect all TypeVar names used in a type.
fn collect_type_vars(ty: &Type, vars: &mut Vec<String>) {
    match ty {
        Type::TypeVar(name) => {
            if !vars.contains(name) {
                vars.push(name.clone());
            }
        }
        Type::List(elem) | Type::Set(elem) => collect_type_vars(elem, vars),
        Type::Dict(k, v) => {
            collect_type_vars(k, vars);
            collect_type_vars(v, vars);
        }
        Type::Tuple(elems) => {
            for e in elems {
                collect_type_vars(e, vars);
            }
        }
        Type::Union(members) => {
            for m in members {
                collect_type_vars(m, vars);
            }
        }
        Type::Callable(params, _, ret) => {
            for p in params {
                collect_type_vars(p, vars);
            }
            collect_type_vars(ret, vars);
        }
        _ => {}
    }
}

/// Substitute type variables in a type with concrete types.
fn substitute_type_vars(ty: &Type, bindings: &HashMap<String, Type>) -> Type {
    match ty {
        Type::TypeVar(name) => {
            bindings.get(name).cloned().unwrap_or_else(|| ty.clone())
        }
        Type::List(elem) => Type::List(Box::new(substitute_type_vars(elem, bindings))),
        Type::Set(elem) => Type::Set(Box::new(substitute_type_vars(elem, bindings))),
        Type::Dict(k, v) => Type::Dict(
            Box::new(substitute_type_vars(k, bindings)),
            Box::new(substitute_type_vars(v, bindings)),
        ),
        Type::Tuple(elems) => Type::Tuple(
            elems.iter().map(|e| substitute_type_vars(e, bindings)).collect(),
        ),
        Type::Union(members) => make_union(
            members.iter().map(|m| substitute_type_vars(m, bindings)).collect(),
        ),
        Type::Callable(params, conventions, ret) => Type::Callable(
            params.iter().map(|p| substitute_type_vars(p, bindings)).collect(),
            conventions.clone(),
            Box::new(substitute_type_vars(ret, bindings)),
        ),
        _ => ty.clone(),
    }
}

/// Try to infer type variable bindings from a concrete argument type and a parameterized type.
fn infer_type_var_bindings(param_ty: &Type, arg_ty: &Type, bindings: &mut HashMap<String, Type>) {
    match (param_ty, arg_ty) {
        (Type::TypeVar(name), concrete) => {
            if !bindings.contains_key(name) {
                bindings.insert(name.clone(), concrete.clone());
            }
        }
        (Type::List(p_elem), Type::List(a_elem)) => {
            infer_type_var_bindings(p_elem, a_elem, bindings);
        }
        (Type::Set(p_elem), Type::Set(a_elem)) => {
            infer_type_var_bindings(p_elem, a_elem, bindings);
        }
        (Type::Dict(pk, pv), Type::Dict(ak, av)) => {
            infer_type_var_bindings(pk, ak, bindings);
            infer_type_var_bindings(pv, av, bindings);
        }
        (Type::Tuple(p_elems), Type::Tuple(a_elems)) if p_elems.len() == a_elems.len() => {
            for (p, a) in p_elems.iter().zip(a_elems.iter()) {
                infer_type_var_bindings(p, a, bindings);
            }
        }
        _ => {}
    }
}

/// Check if a concrete type satisfies a protocol bound.
fn type_satisfies_bound(ty: &Type, bound: &str) -> bool {
    // TypeVars are not concrete — bounds are checked when they're instantiated
    if matches!(ty, Type::TypeVar(_)) {
        return true;
    }
    match bound {
        "Comparable" => matches!(
            ty,
            Type::Int | Type::Float | Type::Str | Type::Bool | Type::BigInt
        ),
        "Addable" => matches!(
            ty,
            Type::Int | Type::Float | Type::Str | Type::BigInt
        ),
        "Hashable" => matches!(
            ty,
            Type::Int | Type::Str | Type::Bool | Type::BigInt | Type::None
                | Type::Enum { .. } | Type::LiteralStr(_) | Type::LiteralInt(_) | Type::LiteralBool(_)
        ),
        _ => true,
    }
}

/// Result of lowering, including the HIR module and any diagnostics.
pub struct LoweringResult {
    pub module: HirModule,
    /// reveal_type() diagnostics (informational, printed to stderr)
    pub reveal_types: Vec<String>,
    /// Compiler warnings (non-fatal, printed to stderr)
    pub warnings: Vec<String>,
}

/// External module definitions that can be imported.
#[derive(Debug, Clone, Default)]
pub struct ExternalDefs {
    /// Map of module_name -> (function_name -> FunctionType)
    pub functions: std::collections::HashMap<String, std::collections::HashMap<String, FunctionType>>,
    /// Map of module_name -> (class_name -> Type)
    pub classes: std::collections::HashMap<String, std::collections::HashMap<String, Type>>,
    /// Map of module_name -> (constant_name -> Type)
    pub constants: std::collections::HashMap<String, std::collections::HashMap<String, Type>>,
    /// Set of class names that are error types (class Foo(Error)) across all modules
    pub error_types: std::collections::HashSet<String>,
    /// Map of module_name -> (owner_name -> (type_var_name -> bounds))
    pub type_param_bounds: std::collections::HashMap<String, std::collections::HashMap<String, std::collections::HashMap<String, Vec<String>>>>,
    /// Map of module_name -> (function_name -> type_var_names)
    pub generic_functions: std::collections::HashMap<String, std::collections::HashMap<String, Vec<String>>>,
}

/// Lower a parsed module AST into a typed HIR module.
pub fn lower_module(stmts: &[Stmt]) -> Result<LoweringResult, Vec<LoweringError>> {
    lower_module_with_externals(stmts, &ExternalDefs::default())
}

/// Lower a stdlib .sifr module. Allows _sifr.* intrinsic imports.
pub fn lower_module_stdlib(stmts: &[Stmt]) -> Result<LoweringResult, Vec<LoweringError>> {
    let mut ctx = LowerCtx::new();
    ctx.allow_intrinsic_imports = true;
    lower_module_impl(stmts, &ExternalDefs::default(), ctx)
}

/// Lower a stdlib .sifr module with external definitions (for inter-stdlib deps).
pub fn lower_module_stdlib_with_externals(stmts: &[Stmt], externals: &ExternalDefs) -> Result<LoweringResult, Vec<LoweringError>> {
    let mut ctx = LowerCtx::new();
    ctx.allow_intrinsic_imports = true;
    lower_module_impl(stmts, externals, ctx)
}

/// Lower a parsed module AST into a typed HIR module, with external module definitions.
pub fn lower_module_with_externals(stmts: &[Stmt], externals: &ExternalDefs) -> Result<LoweringResult, Vec<LoweringError>> {
    let ctx = LowerCtx::new();
    lower_module_impl(stmts, externals, ctx)
}

/// Internal implementation of module lowering.
fn lower_module_impl(stmts: &[Stmt], externals: &ExternalDefs, mut ctx: LowerCtx) -> Result<LoweringResult, Vec<LoweringError>> {

    // Register built-in functions
    register_builtins(&mut ctx);

    // Pass 0: Pre-register all class names as forward-reference placeholders.
    // This allows function signatures and other classes to reference classes
    // defined later in the file (e.g., ListNode, TreeNode, Node).
    for stmt in stmts {
        if let Stmt::ClassDef(class_def) = stmt {
            let class_name = class_def.name.to_string();
            if !ctx.class_types.contains_key(&class_name) {
                ctx.class_types.insert(class_name.clone(), Type::Class {
                    name: class_name,
                    fields: Vec::new(),
                    methods: Vec::new(),
                    parent_class: None,
                });
            }
        }
    }

    // Pass 0.5: Recognize TypeVar declarations: T = TypeVar("T")
    // These must be processed before type aliases and function signatures.
    for stmt in stmts {
        if let Stmt::Assign(assign) = stmt {
            if assign.targets.len() == 1 {
                if let Expr::Name(name) = &assign.targets[0] {
                    if let Expr::Call(call) = assign.value.as_ref() {
                        if let Expr::Name(func_name) = call.func.as_ref() {
                            if func_name.id.as_str() == "TypeVar" {
                                // Register this name as a type variable
                                ctx.type_vars.insert(name.id.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    // First pass: collect class definitions first (so function signatures can reference them),
    // then type aliases, then function signatures.
    for stmt in stmts {
        if let Stmt::ClassDef(class_def) = stmt {
            collect_class_type(class_def, &mut ctx);
        }
    }

    // Early import pass: resolve imported types so they're available for function signatures.
    // This must happen before function signature extraction so that imported error classes
    // (e.g., StatisticsError from sifr.statistics) can be used in Result[T, E] annotations.
    resolve_imports_early(stmts, externals, &mut ctx);

    for stmt in stmts {
        match stmt {
            Stmt::TypeAlias(type_alias) => {
                let name = match type_alias.name.as_ref() {
                    Expr::Name(n) => n.id.to_string(),
                    _ => {
                        ctx.error("type alias name must be a simple name".to_string());
                        continue;
                    }
                };
                let mut alias_type_params = Vec::new();
                if let Some(ref tps) = type_alias.type_params {
                    for tp in tps.iter() {
                        if let sifr_python_ast::TypeParam::TypeVar(tv) = tp {
                            let tp_name = tv.name.to_string();
                            ctx.type_vars.insert(tp_name.clone());
                            alias_type_params.push(tp_name);
                        }
                    }
                }
                let ty = resolve_annotation_expr(&type_alias.value, &mut ctx);
                if alias_type_params.is_empty() {
                    ctx.scope.define_type_alias(name, ty);
                } else {
                    ctx.scope.define_generic_type_alias(name, alias_type_params.clone(), ty);
                }
                for tp_name in &alias_type_params {
                    ctx.type_vars.remove(tp_name.as_str());
                }
            }
            _ => {}
        }
    }
    for stmt in stmts {
        if let Stmt::FunctionDef(func) = stmt {
            // PEP 695: register inline type params (def f[T](...)) as type variables
            let mut pep695_type_vars = Vec::new();
            if let Some(ref type_params) = func.type_params {
                for tp in type_params.iter() {
                    if let sifr_python_ast::TypeParam::TypeVar(tv) = tp {
                        let name = tv.name.to_string();
                        ctx.type_vars.insert(name.clone());
                        pep695_type_vars.push(name.clone());
                        if let Some(ref bound) = tv.bound {
                            let bound_name = match bound.as_ref() {
                                Expr::Name(n) => n.id.to_string(),
                                _ => continue,
                            };
                            ctx.type_param_bounds
                                .entry(func.name.to_string())
                                .or_insert_with(HashMap::new)
                                .entry(name)
                                .or_insert_with(Vec::new)
                                .push(bound_name);
                        }
                    }
                }
            }

            if let Some(ft) = extract_function_type(func, &mut ctx) {
                // Track which type variables this function uses (makes it generic)
                let mut func_type_vars = Vec::new();
                for (_, ty, _) in &ft.params {
                    collect_type_vars(ty, &mut func_type_vars);
                }
                collect_type_vars(&ft.return_type, &mut func_type_vars);
                // Also include PEP 695 type params
                for tv in &pep695_type_vars {
                    if !func_type_vars.contains(tv) {
                        func_type_vars.push(tv.clone());
                    }
                }
                func_type_vars.sort();
                func_type_vars.dedup();
                if !func_type_vars.is_empty() {
                    ctx.generic_functions.insert(func.name.to_string(), func_type_vars);
                }

                // Collect default values for parameters
                let mut defaults = Vec::new();
                for (i, param) in func.parameters.args.iter().enumerate() {
                    if let Some(ref default_expr) = param.default {
                        if let Some(hir_default) = lower_expr_simple(default_expr) {
                            defaults.push((i, hir_default));
                        }
                    }
                }
                // Also collect defaults for keyword-only args
                let regular_count = func.parameters.args.len();
                for (i, param) in func.parameters.kwonlyargs.iter().enumerate() {
                    if let Some(ref default_expr) = param.default {
                        if let Some(hir_default) = lower_expr_simple(default_expr) {
                            defaults.push((regular_count + i, hir_default));
                        }
                    }
                }
                if !defaults.is_empty() {
                    ctx.function_defaults.insert(func.name.to_string(), defaults);
                }
                ctx.functions.insert(func.name.to_string(), ft);
                // Track vararg functions
                if func.parameters.vararg.is_some() {
                    ctx.vararg_functions.insert(func.name.to_string());
                }
            }
        }
    }

    // Collect import statements and resolve imported names
    let mut imports = Vec::new();
    for stmt in stmts {
        if let Stmt::ImportFrom(import_from) = stmt {
            if let Some(ref module) = import_from.module {
                let module_name = module.to_string();
                let names: Vec<String> = import_from.names.iter()
                    .map(|alias| alias.name.to_string())
                    .collect();
                // Collect aliases: (original_name, local_alias)
                let aliases: Vec<(String, String)> = import_from.names.iter()
                    .filter_map(|alias| {
                        alias.asname.as_ref().map(|asname| {
                            (alias.name.to_string(), asname.to_string())
                        })
                    })
                    .collect();

                // Build a mapping from original name -> local name (alias or original)
                let local_name_for = |original: &str| -> String {
                    aliases.iter()
                        .find(|(orig, _)| orig == original)
                        .map(|(_, alias)| alias.clone())
                        .unwrap_or_else(|| original.to_string())
                };

                // Skip typing imports (TypeVar, Callable, etc.) - they are handled at the type level
                if module_name == "typing" {
                    continue;
                }

                // Skip enum imports (Enum is a built-in base class in Sifr)
                if module_name == "enum" {
                    continue;
                }

                // Block user imports of _sifr.* (internal intrinsics)
                // Stdlib .sifr files are allowed to import from _sifr.*
                if module_name.starts_with("_sifr.") {
                    if !ctx.allow_intrinsic_imports {
                        ctx.error(format!("cannot import from '{}' — _sifr.* modules are internal compiler intrinsics", module_name));
                        continue;
                    }
                    // Resolve intrinsic imports for stdlib .sifr files
                    if let Some(intrinsic_module) = crate::stdlib::get_intrinsic_module(&module_name) {
                        for name in &names {
                            let local = local_name_for(name);
                            if let Some(ft) = intrinsic_module.functions.get(name) {
                                ctx.functions.insert(local, ft.clone());
                            } else if let Some(const_ty) = intrinsic_module.constants.get(name) {
                                ctx.scope.define(local, const_ty.clone());
                            } else {
                                ctx.error(format!("intrinsic module '{}' has no member '{}'", module_name, name));
                            }
                        }
                        imports.push(HirImport {
                            module: module_name,
                            names,
                            aliases,
                        });
                        continue;
                    } else {
                        ctx.error(format!("unknown intrinsic module '{}'", module_name));
                        continue;
                    }
                }

                // Check if this is a stdlib import (sifr.*)
                // All sifr.* modules are now .sifr files compiled in the stdlib phase.
                // Resolve from pre-compiled stdlib modules (via externals).
                if module_name.starts_with("sifr.") {
                    // Check if there's a pre-compiled stdlib .sifr module in externals
                    let stdlib_module_key = module_name.clone();
                    let has_module = externals.functions.contains_key(&stdlib_module_key)
                        || externals.classes.contains_key(&stdlib_module_key)
                        || externals.constants.contains_key(&stdlib_module_key);
                    if has_module {
                        // Resolve each imported name from the stdlib module
                        for name in &names {
                            let local = local_name_for(name);
                            let mut found = false;
                            // Check functions
                            if let Some(module_fns) = externals.functions.get(&stdlib_module_key) {
                                if let Some(ft) = module_fns.get(name) {
                                    ctx.functions.insert(local.clone(), ft.clone());
                                    found = true;
                                    // Import generic function info and bounds
                                    if let Some(module_gf) = externals.generic_functions.get(&stdlib_module_key) {
                                        if let Some(type_vars) = module_gf.get(name) {
                                            ctx.generic_functions.insert(local.clone(), type_vars.clone());
                                        }
                                    }
                                    if let Some(module_bounds) = externals.type_param_bounds.get(&stdlib_module_key) {
                                        if let Some(owner_bounds) = module_bounds.get(name) {
                                            ctx.type_param_bounds.insert(local.clone(), owner_bounds.clone());
                                        }
                                    }
                                }
                            }
                            // Check classes
                            if !found {
                                if let Some(module_classes) = externals.classes.get(&stdlib_module_key) {
                                    if let Some(class_ty) = module_classes.get(name) {
                                        ctx.class_types.insert(local.clone(), class_ty.clone());
                                        // Register as error type if flagged in external defs
                                        if externals.error_types.contains(name) {
                                            ctx.error_types.insert(local.clone());
                                        }
                                        // Register constructor: prefer `new` method params if available
                                        if let Type::Class { fields, methods, .. } = class_ty {
                                            let ft = if let Some((_, new_ft)) = methods.iter().find(|(n, _)| n == "new") {
                                                let params: Vec<(String, Type)> = new_ft.params.iter()
                                                    .map(|(n, t, _)| (n.clone(), t.clone()))
                                                    .collect();
                                                FunctionType::new(params, class_ty.clone())
                                            } else {
                                                let params: Vec<(String, Type)> = fields.clone();
                                                FunctionType::new(params, class_ty.clone())
                                            };
                                            ctx.functions.insert(local.clone(), ft);
                                        }
                                        // Import class type parameter bounds
                                        if let Some(module_bounds) = externals.type_param_bounds.get(&stdlib_module_key) {
                                            if let Some(owner_bounds) = module_bounds.get(name) {
                                                ctx.type_param_bounds.insert(local.clone(), owner_bounds.clone());
                                            }
                                        }
                                        found = true;
                                    }
                                }
                            }
                            // Check constants
                            if !found {
                                if let Some(module_consts) = externals.constants.get(&stdlib_module_key) {
                                    if let Some(const_ty) = module_consts.get(name) {
                                        ctx.scope.define(local, const_ty.clone());
                                        found = true;
                                    }
                                }
                            }
                            if !found {
                                ctx.error(format!("module '{}' has no member '{}'", module_name, name));
                            }
                        }
                        imports.push(HirImport {
                            module: module_name,
                            names,
                            aliases,
                        });
                        continue;
                    } else {
                        // Module doesn't exist in stdlib — emit clear error at the import site
                        ctx.error(format!("unknown stdlib module '{}'", module_name));
                        continue;
                    }
                }

                // Check if the local module exists in externals before resolving
                let has_local_module = externals.functions.contains_key(&module_name)
                    || externals.classes.contains_key(&module_name);
                if !has_local_module {
                    ctx.error(format!("unknown module '{}'", module_name));
                    continue;
                }

                // Resolve imported names from external definitions (local modules)
                for name in &names {
                    let local = local_name_for(name);
                    // Check if it's a private name
                    if name.starts_with('_') {
                        ctx.error(format!("cannot import private name '{}' from module '{}'", name, module_name));
                        continue;
                    }

                    let mut found = false;
                    // Look up in external functions
                    if let Some(module_fns) = externals.functions.get(&module_name) {
                        if let Some(ft) = module_fns.get(name) {
                            ctx.functions.insert(local.clone(), ft.clone());
                            found = true;
                        }
                    }
                    // Look up in external classes
                    if !found {
                        if let Some(module_classes) = externals.classes.get(&module_name) {
                            if let Some(class_ty) = module_classes.get(name) {
                                ctx.class_types.insert(local.clone(), class_ty.clone());
                                // Register as error type if flagged in external defs
                                if externals.error_types.contains(name) {
                                    ctx.error_types.insert(local.clone());
                                }
                                // Register the constructor: prefer `new` method params if available,
                                // otherwise fall back to field-based constructor
                                if let Type::Class { fields, methods, .. } = class_ty {
                                    let ft = if let Some((_, new_ft)) = methods.iter().find(|(n, _)| n == "new") {
                                        // Use the actual __init__ parameters
                                        let params: Vec<(String, Type)> = new_ft.params.iter()
                                            .map(|(n, t, _)| (n.clone(), t.clone()))
                                            .collect();
                                        FunctionType::new(params, class_ty.clone())
                                    } else {
                                        // No __init__ — default constructor from fields
                                        let params: Vec<(String, Type)> = fields.clone();
                                        FunctionType::new(params, class_ty.clone())
                                    };
                                    ctx.functions.insert(local, ft);
                                }
                                found = true;
                            }
                        }
                    }
                    if !found {
                        ctx.error(format!("module '{}' has no member '{}'", module_name, name));
                    }
                }

                imports.push(HirImport {
                    module: module_name,
                    names,
                    aliases,
                });
            }
        }
    }

    // Collect module-level constants (annotated assignments at top level)
    let mut constants = Vec::new();
    for stmt in stmts {
        if let Stmt::AnnAssign(ann) = stmt {
            if let Expr::Name(name) = ann.target.as_ref() {
                let var_name = name.id.to_string();
                let ty = resolve_annotation_expr(&ann.annotation, &mut ctx);
                if let Some(ref value_expr) = ann.value {
                    if let Some(hir_value) = lower_expr_simple(value_expr) {
                        ctx.scope.define(var_name.clone(), ty.clone());
                        constants.push((var_name, ty, hir_value));
                    }
                }
            }
        }
        // Also handle bare assignments: PI = 3.14 (without annotation)
        if let Stmt::Assign(assign) = stmt {
            if assign.targets.len() == 1 {
                if let Expr::Name(name) = &assign.targets[0] {
                    let var_name = name.id.to_string();
                    // Skip TypeVar declarations (already handled)
                    if ctx.type_vars.contains(&var_name) {
                        continue;
                    }
                    if let Some(hir_value) = lower_expr_simple(&assign.value) {
                        let ty = hir_value.ty().clone();
                        ctx.scope.define(var_name.clone(), ty.clone());
                        constants.push((var_name, ty, hir_value));
                    }
                }
            }
        }
    }

    // Second pass: lower function bodies and class method bodies
    let mut functions = Vec::new();
    let mut classes = Vec::new();
    for stmt in stmts {
        match stmt {
            Stmt::FunctionDef(func) => {
                if let Some(hir_func) = lower_function(func, &mut ctx) {
                    functions.push(hir_func);
                }
            }
            Stmt::ClassDef(class_def) => {
                if let Some(hir_class) = lower_class(class_def, &mut ctx) {
                    classes.push(hir_class);
                }
            }
            _ => {}
        }
    }

    if ctx.errors.is_empty() {
        Ok(LoweringResult {
            module: HirModule {
                functions,
                classes,
                imports,
                constants,
                generic_functions: ctx.generic_functions.clone(),
                type_param_bounds: ctx.type_param_bounds.clone(),
            },
            reveal_types: ctx.reveal_types,
            warnings: ctx.warnings,
        })
    } else {
        Err(ctx.errors)
    }
}

/// Early import resolution: register imported types/functions/constants in the context
/// so they're available during function signature extraction and type annotation resolution.
/// This is a subset of the full import processing — it doesn't produce HirImport nodes.
fn resolve_imports_early(stmts: &[Stmt], externals: &ExternalDefs, ctx: &mut LowerCtx) {
    for stmt in stmts {
        if let Stmt::ImportFrom(import_from) = stmt {
            if let Some(ref module) = import_from.module {
                let module_name = module.to_string();
                let names: Vec<String> = import_from.names.iter()
                    .map(|alias| alias.name.to_string())
                    .collect();
                let aliases: Vec<(String, String)> = import_from.names.iter()
                    .filter_map(|alias| {
                        alias.asname.as_ref().map(|asname| {
                            (alias.name.to_string(), asname.to_string())
                        })
                    })
                    .collect();
                let local_name_for = |original: &str| -> String {
                    aliases.iter()
                        .find(|(orig, _)| orig == original)
                        .map(|(_, alias)| alias.clone())
                        .unwrap_or_else(|| original.to_string())
                };

                // Only resolve from externals (stdlib and local modules)
                let module_key = module_name.clone();
                if let Some(module_classes) = externals.classes.get(&module_key) {
                    for name in &names {
                        let local = local_name_for(name);
                        if let Some(class_ty) = module_classes.get(name) {
                            if !ctx.class_types.contains_key(&local) {
                                ctx.class_types.insert(local.clone(), class_ty.clone());
                                // Register as error type if flagged
                                if externals.error_types.contains(name) {
                                    ctx.error_types.insert(local.clone());
                                }
                                // Register constructor
                                if let Type::Class { fields, methods, .. } = class_ty {
                                    let ft = if let Some((_, new_ft)) = methods.iter().find(|(n, _)| n == "new") {
                                        let params: Vec<(String, Type)> = new_ft.params.iter()
                                            .map(|(n, t, _)| (n.clone(), t.clone()))
                                            .collect();
                                        FunctionType::new(params, class_ty.clone())
                                    } else {
                                        let params: Vec<(String, Type)> = fields.clone();
                                        FunctionType::new(params, class_ty.clone())
                                    };
                                    ctx.functions.insert(local, ft);
                                }
                            }
                        }
                    }
                }
                if let Some(module_fns) = externals.functions.get(&module_key) {
                    for name in &names {
                        let local = local_name_for(name);
                        if let Some(ft) = module_fns.get(name) {
                            if !ctx.functions.contains_key(&local) {
                                ctx.functions.insert(local, ft.clone());
                            }
                        }
                    }
                }
                if let Some(module_consts) = externals.constants.get(&module_key) {
                    for name in &names {
                        let local = local_name_for(name);
                        if let Some(const_ty) = module_consts.get(name) {
                            ctx.scope.define(local, const_ty.clone());
                        }
                    }
                }
            }
        }
    }
}

/// Check if a class definition extends an error class (Error or any registered error type).
fn is_error_class_with_ctx(class_def: &StmtClassDef, error_types: &std::collections::HashSet<String>) -> bool {
    for base in class_def.bases() {
        if let Expr::Name(n) = base {
            let base_name = n.id.as_str();
            if base_name == "Error" || error_types.contains(base_name) {
                return true;
            }
        }
    }
    false
}

/// Check if a class definition has `(Error)` as its base class (legacy, for contexts without error_types).
fn is_error_class(class_def: &StmtClassDef) -> bool {
    for base in class_def.bases() {
        if let Expr::Name(n) = base {
            if n.id.as_str() == "Error" {
                return true;
            }
        }
    }
    false
}

/// Check if a type is a valid error type (a class registered in error_types).
fn is_valid_error_type(ty: &Type, ctx: &LowerCtx) -> bool {
    match ty {
        Type::Class { name, .. } => ctx.error_types.contains(name),
        _ => false,
    }
}

/// Format a type name for use in error messages.
fn format_type_name(ty: &Type) -> String {
    match ty {
        Type::Int => "int".to_string(),
        Type::Float => "float".to_string(),
        Type::Str => "str".to_string(),
        Type::Bool => "bool".to_string(),
        Type::None => "None".to_string(),
        Type::Class { name, .. } => name.clone(),
        Type::List(inner) => format!("list[{}]", format_type_name(inner)),
        Type::Dict(k, v) => format!("dict[{}, {}]", format_type_name(k), format_type_name(v)),
        _ => format!("{:?}", ty),
    }
}


/// Collect error types from raise statements in a list of HIR statements.
fn collect_raise_error_types(stmts: &[HirStmt], errors: &mut std::collections::HashSet<String>) {
    for stmt in stmts {
        match stmt {
            HirStmt::Raise { value } => {
                if let Type::Class { name, .. } = value.ty() {
                    errors.insert(name.clone());
                }
            }
            HirStmt::If { then_body, elif_clauses, else_body, .. } => {
                collect_raise_error_types(then_body, errors);
                for (_, body) in elif_clauses {
                    collect_raise_error_types(body, errors);
                }
                if let Some(eb) = else_body {
                    collect_raise_error_types(eb, errors);
                }
            }
            HirStmt::While { body, .. } | HirStmt::For { body, .. } => {
                collect_raise_error_types(body, errors);
            }
            _ => {}
        }
    }
}

/// Check if a class definition has `(Protocol)` as its base class.
fn is_protocol_class(class_def: &StmtClassDef) -> bool {
    for base in class_def.bases() {
        if let Expr::Name(n) = base {
            if n.id.as_str() == "Protocol" {
                return true;
            }
        }
    }
    false
}

/// Check if a class definition is a newtype wrapper around a primitive.
/// Returns the wrapped primitive type if so.
fn get_newtype_inner(class_def: &StmtClassDef) -> Option<Type> {
    for base in class_def.bases() {
        if let Expr::Name(n) = base {
            match n.id.as_str() {
                "int" => return Some(Type::Int),
                "float" => return Some(Type::Float),
                "str" => return Some(Type::Str),
                "bool" => return Some(Type::Bool),
                _ => {}
            }
        }
    }
    None
}

/// Dunder method names that map to Rust operator trait impls.
const OPERATOR_DUNDERS: &[&str] = &[
    "__add__", "__sub__", "__mul__", "__truediv__", "__floordiv__", "__mod__",
    "__eq__", "__ne__", "__lt__", "__le__", "__gt__", "__ge__",
    "__str__", "__repr__",
    "__neg__", "__pos__",
    "__contains__",
];

/// Check if a method name is an operator dunder.
fn is_operator_dunder(name: &str) -> bool {
    OPERATOR_DUNDERS.contains(&name)
}

/// Get the parent class name for single inheritance.
/// Returns None for Error, Protocol, and primitive base classes.
fn get_parent_class(class_def: &StmtClassDef) -> Option<String> {
    for base in class_def.bases() {
        if let Expr::Name(n) = base {
            let name = n.id.as_str();
            // Skip special base classes
            if matches!(name, "Error" | "Protocol" | "int" | "float" | "str" | "bool" | "Enum") {
                return None;
            }
            return Some(name.to_string());
        }
    }
    None
}

/// Check if a class is an enum (inherits from Enum)
fn is_enum_class(class_def: &StmtClassDef) -> bool {
    for base in class_def.bases() {
        if let Expr::Name(n) = base {
            if n.id.as_str() == "Enum" {
                return true;
            }
        }
    }
    false
}

/// Collect enum variants from a class body
/// Returns (name, optional_int_value) for each variant
fn collect_enum_variants(class_def: &StmtClassDef) -> Vec<(String, Option<i64>)> {
    let mut variants = Vec::new();
    let mut auto_value = 1i64;
    for stmt in &class_def.body {
        match stmt {
            Stmt::Assign(assign) => {
                if assign.targets.len() == 1 {
                    if let Expr::Name(name) = &assign.targets[0] {
                        let variant_name = name.id.to_string();
                        // Check if it has an integer value
                        let value = if let Expr::NumberLiteral(num) = assign.value.as_ref() {
                            if let sifr_python_ast::Number::Int(i) = &num.value {
                                i.as_i64()
                            } else {
                                None
                            }
                        } else {
                            None
                        };
                        let v = value.unwrap_or(auto_value);
                        auto_value = v + 1;
                        variants.push((variant_name, value));
                    }
                }
            }
            Stmt::AnnAssign(ann) => {
                // `RED: int = 1` style
                if let Expr::Name(name) = ann.target.as_ref() {
                    let variant_name = name.id.to_string();
                    let value = if let Some(val_expr) = &ann.value {
                        if let Expr::NumberLiteral(num) = val_expr.as_ref() {
                            if let sifr_python_ast::Number::Int(i) = &num.value {
                                i.as_i64()
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    let v = value.unwrap_or(auto_value);
                    auto_value = v + 1;
                    variants.push((variant_name, value));
                }
            }
            _ => {}
        }
    }
    variants
}

/// Check if a function definition has a specific decorator.
fn has_decorator(func: &StmtFunctionDef, decorator_name: &str) -> bool {
    for decorator in func.decorator_list.iter() {
        if let Expr::Name(n) = &decorator.expression {
            if n.id.as_str() == decorator_name {
                return true;
            }
        }
    }
    false
}

/// First pass: collect class fields and method signatures, register the class type.
fn collect_class_type(class_def: &StmtClassDef, ctx: &mut LowerCtx) {
    let class_name = class_def.name.to_string();
    let mut fields: Vec<(String, Type)> = Vec::new();
    let mut methods: Vec<(String, FunctionType)> = Vec::new();
    let is_error = is_error_class(class_def);
    let is_protocol = is_protocol_class(class_def);
    let newtype_inner = get_newtype_inner(class_def);

    // PEP 695: register inline type params (class C[T]) as type variables
    if let Some(ref type_params) = class_def.type_params {
        let mut declared_params = Vec::new();
        for tp in type_params.iter() {
            if let sifr_python_ast::TypeParam::TypeVar(tv) = tp {
                let tp_name = tv.name.to_string();
                ctx.type_vars.insert(tp_name.clone());
                declared_params.push(tp_name.clone());
                if let Some(ref bound) = tv.bound {
                    if let Expr::Name(n) = bound.as_ref() {
                        ctx.type_param_bounds
                            .entry(class_name.clone())
                            .or_insert_with(HashMap::new)
                            .entry(tp_name)
                            .or_insert_with(Vec::new)
                            .push(n.id.to_string());
                    }
                }
            }
        }
        if !declared_params.is_empty() {
            ctx.class_declared_type_params.insert(class_name.clone(), declared_params);
        }
    }

    // For newtype declarations, register as a Newtype type
    if let Some(ref inner) = newtype_inner {
        let newtype_ty = Type::Newtype {
            name: class_name.clone(),
            inner: Box::new(inner.clone()),
        };
        ctx.class_types.insert(class_name.clone(), newtype_ty.clone());

        // Register constructor: ClassName(value) -> ClassName
        let ft = FunctionType::new(
            vec![("value".to_string(), inner.clone())],
            newtype_ty,
        );
        ctx.functions.insert(class_name.clone(), ft);
        return;
    }

    // For enum declarations, register as an Enum type
    if is_enum_class(class_def) {
        let variants = collect_enum_variants(class_def);
        // Check for duplicate variant values
        {
            let mut seen_values: std::collections::HashMap<i64, String> = std::collections::HashMap::new();
            for (vname, vval) in &variants {
                let val = vval.unwrap_or(0);
                if let Some(existing) = seen_values.get(&val) {
                    if vval.is_some() {
                        ctx.error(format!(
                            "enum '{}' has duplicate value {}: variants '{}' and '{}'",
                            class_name, val, existing, vname
                        ));
                    }
                } else if vval.is_some() {
                    seen_values.insert(val, vname.clone());
                }
            }
        }
        let enum_ty = Type::Enum {
            name: class_name.clone(),
            variants: variants.iter().map(|(n, v)| (n.clone(), *v)).collect(),
        };
        ctx.class_types.insert(class_name.clone(), enum_ty.clone());
        // Register each variant as a constant of the enum type
        for (variant_name, _) in &variants {
            ctx.functions.insert(
                format!("{}.{}", class_name, variant_name),
                FunctionType::new(vec![], enum_ty.clone()),
            );
        }
        // Collect method signatures from enum body and register them
        for stmt in &class_def.body {
            if let Stmt::FunctionDef(func) = stmt {
                let method_name = func.name.to_string();
                if method_name == "__init__" { continue; }
                let mut params = Vec::new();
                for param in func.parameters.args.iter().skip(1) {
                    let param_name = param.parameter.name.to_string();
                    let param_ty = if let Some(ref ann) = param.parameter.annotation {
                        resolve_annotation_expr(ann, ctx)
                    } else {
                        Type::Any
                    };
                    params.push((param_name, param_ty));
                }
                let return_ty = if let Some(ref ret_ann) = func.returns {
                    resolve_annotation_expr(ret_ann, ctx)
                } else {
                    Type::None
                };
                let ft = FunctionType::new(params, return_ty);
                // Register method as ClassName.method_name for lookup
                ctx.functions.insert(format!("{}.{}", class_name, method_name), ft.clone());
                methods.push((method_name, ft));
            }
        }
        return;
    }

    // For protocol definitions, register as a Protocol type
    if is_protocol {
        // Collect method signatures for the protocol
        for stmt in &class_def.body {
            if let Stmt::FunctionDef(func) = stmt {
                let method_name = func.name.to_string();
                if method_name == "__init__" { continue; }
                let mut params = Vec::new();
                for param in func.parameters.args.iter().skip(1) {
                    let param_name = param.parameter.name.to_string();
                    let param_ty = if let Some(ref ann) = param.parameter.annotation {
                        resolve_annotation_expr(ann, ctx)
                    } else {
                        Type::Any
                    };
                    params.push((param_name, param_ty));
                }
                let return_ty = if let Some(ref ret_ann) = func.returns {
                    resolve_annotation_expr(ret_ann, ctx)
                } else {
                    Type::None
                };
                methods.push((method_name, FunctionType::new(params, return_ty)));
            }
        }
        let proto_ty = Type::Protocol {
            name: class_name.clone(),
            methods: methods.clone(),
        };
        ctx.class_types.insert(class_name, proto_ty);
        return;
    }

    // For error types, ensure a 'message' field exists (add if not explicitly declared)
    // This will be checked after collecting all fields

    // Inherit parent fields and methods for single inheritance
    let parent_class_name = get_parent_class(class_def);
    if let Some(ref parent_name) = parent_class_name {
        if let Some(parent_ty) = ctx.class_types.get(parent_name).cloned() {
            if let Type::Class { fields: parent_fields, methods: parent_methods, .. } = parent_ty {
                // Inherit parent fields
                for (fname, fty) in &parent_fields {
                    fields.push((fname.clone(), fty.clone()));
                }
                // Inherit parent methods
                for (mname, mft) in &parent_methods {
                    methods.push((mname.clone(), mft.clone()));
                }
            }
        } else {
            ctx.error(format!("parent class '{}' not defined", parent_name));
        }
    }

    // Register a preliminary class type so self-referential annotations work
    // (e.g., `def distance(self, other: Point)` inside class Point)
    ctx.class_types.insert(class_name.clone(), Type::Class {
        name: class_name.clone(),
        fields: vec![],
        methods: vec![],
        parent_class: None,
    });

    let mut field_defaults: Vec<(usize, HirExpr)> = Vec::new();

    for stmt in &class_def.body {
        match stmt {
            // Field annotations: `x: float` or `x: float = 0.0`
            Stmt::AnnAssign(ann) => {
                if let Expr::Name(name) = ann.target.as_ref() {
                    let ty = resolve_annotation_expr(&ann.annotation, ctx);
                    let field_idx = fields.len();
                    fields.push((name.id.to_string(), ty));
                    // Collect default value if present (for auto-init default params)
                    if let Some(ref default_expr) = ann.value {
                        if let Some(hir_default) = lower_expr_simple(default_expr) {
                            field_defaults.push((field_idx, hir_default));
                        }
                    }
                }
            }
            // Method definitions
            Stmt::FunctionDef(func) => {
                let method_name = func.name.to_string();
                if method_name == "__init__" {
                    // Constructor: extract params (skip `self`)
                    let mut params = Vec::new();
                    for param in func.parameters.args.iter().skip(1) {
                        let param_name = param.parameter.name.to_string();
                        let param_ty = if let Some(ref ann) = param.parameter.annotation {
                            resolve_annotation_expr(ann, ctx)
                        } else {
                            ctx.error(format!(
                                "parameter '{}' in {}.__init__ is missing a type annotation",
                                param_name, class_name
                            ));
                            Type::Any
                        };
                        params.push((param_name, param_ty));
                    }
                    // Constructor returns the class type (registered below)
                    // We store it as a function for call resolution
                    let constructor_ft = FunctionType::new(params.clone(), Type::None); // placeholder, updated below
                    ctx.functions.insert(class_name.clone(), constructor_ft);

                    // Collect defaults for constructor
                    let mut defaults = Vec::new();
                    for (i, param) in func.parameters.args.iter().skip(1).enumerate() {
                        if let Some(ref default_expr) = param.default {
                            if let Some(hir_default) = lower_expr_simple(default_expr) {
                                defaults.push((i, hir_default));
                            }
                        }
                    }
                    if !defaults.is_empty() {
                        ctx.function_defaults.insert(class_name.clone(), defaults);
                    }
                } else {
                    // Regular/class/static method: extract params
                    // For @staticmethod, don't skip any params
                    // For @classmethod and regular methods, skip `self`/`cls`
                    let is_static = has_decorator(func, "staticmethod");
                    let skip_count = if is_static { 0 } else { 1 };
                    let mut params = Vec::new();
                    for param in func.parameters.args.iter().skip(skip_count) {
                        let param_name = param.parameter.name.to_string();
                        let param_ty = if let Some(ref ann) = param.parameter.annotation {
                            resolve_annotation_expr(ann, ctx)
                        } else {
                            ctx.error(format!(
                                "parameter '{}' in {}.{} is missing a type annotation",
                                param_name, class_name, method_name
                            ));
                            Type::Any
                        };
                        params.push((param_name, param_ty));
                    }
                    let return_ty = if let Some(ref ret_ann) = func.returns {
                        resolve_annotation_expr(ret_ann, ctx)
                    } else {
                        Type::None
                    };
                    methods.push((method_name, FunctionType::new(params, return_ty)));
                }
            }
            Stmt::Pass(_) => {} // Allow pass in class body
            _ => {
                ctx.error(format!("unsupported statement in class '{}' body", class_name));
            }
        }
    }

    let class_ty = Type::Class {
        name: class_name.clone(),
        fields: fields.clone(),
        methods: methods.clone(),
        parent_class: None,
    };

    // Update the constructor function to return the class type
    if let Some(ft) = ctx.functions.get_mut(&class_name) {
        ft.return_type = Box::new(class_ty.clone());
    } else {
        // No __init__ defined -- create a default constructor from fields

        // Validate field ordering: required fields must come before defaulted fields
        let default_indices: std::collections::HashSet<usize> = field_defaults.iter().map(|(i, _)| *i).collect();
        let mut seen_default = false;
        for (i, (fname, _)) in fields.iter().enumerate() {
            if default_indices.contains(&i) {
                seen_default = true;
            } else if seen_default {
                ctx.error(format!(
                    "class '{}': required field '{}' declared after field with default value",
                    class_name, fname
                ));
            }
        }

        // Inheritance diagnostic: warn when child has own fields but no __init__ and extends a parent
        if parent_class_name.is_some() {
            let parent_field_count = if let Some(ref pname) = parent_class_name {
                ctx.class_types.get(pname).map_or(0, |ty| {
                    if let Type::Class { fields: pf, .. } = ty { pf.len() } else { 0 }
                })
            } else { 0 };
            let has_own_fields = fields.len() > parent_field_count;
            if has_own_fields {
                ctx.error(format!(
                    "class '{}' has fields but no __init__; parent fields will not be initialized. \
                     Define an explicit __init__ with super().__init__(...)",
                    class_name
                ));
            }
        }

        let params: Vec<(String, Type)> = fields.clone();
        let ft = FunctionType::new(params, class_ty.clone());
        ctx.functions.insert(class_name.clone(), ft);
        // Store field defaults for the auto-generated constructor
        if !field_defaults.is_empty() {
            ctx.function_defaults.insert(class_name.clone(), field_defaults);
        }
    }

    if is_error {
        ctx.error_types.insert(class_name.clone());
    }

    ctx.class_types.insert(class_name, class_ty);
}

/// Second pass: lower class method bodies into HirClass.
fn lower_class(class_def: &StmtClassDef, ctx: &mut LowerCtx) -> Option<HirClass> {
    let class_name = class_def.name.to_string();
    let class_ty = ctx.class_types.get(&class_name)?.clone();
    let is_protocol = is_protocol_class(class_def);
    let newtype_inner = get_newtype_inner(class_def);

    // For protocol definitions, emit a HirClass with is_protocol=true
    if is_protocol {
        let methods_sigs = match &class_ty {
            Type::Protocol { methods, .. } => methods.clone(),
            _ => return None,
        };
        // Protocols have no fields, no body to lower -- just method signatures
        let hir_methods: Vec<HirFunction> = methods_sigs.iter().map(|(name, ft)| {
            HirFunction {
                name: name.clone(),
                params: ft.params.iter().map(|(pn, pt, _)| HirParam {
                    name: pn.clone(),
                    ty: pt.clone(),
                    default: None,
                    keyword_only: false,
                    convention: ParamConvention::default(),
                }).collect(),
                return_type: *ft.return_type.clone(),
                body: vec![], // Protocol methods have no body
                method_kind: MethodKind::Regular,
                decorators: vec![],
                type_params: Vec::new(),
            }
        }).collect();

        return Some(HirClass {
            name: class_name,
            fields: vec![],
            methods: hir_methods,
            is_hashable: false,
            is_error_type: false,
            is_protocol: true,
            operator_impls: Vec::new(),
            newtype_inner: None,
            implements_protocols: Vec::new(),
            parent_class: None,
            type_params: Vec::new(),
            is_enum: false,
            enum_variants: Vec::new(),
        });
    }

    // For enum declarations, emit a HirClass with is_enum=true
    if is_enum_class(class_def) {
        let variants = collect_enum_variants(class_def);
        // Lower any methods defined in the enum body
        let mut hir_methods = Vec::new();
        ctx.current_class = Some(class_name.clone());
        for stmt in &class_def.body {
            if let Stmt::FunctionDef(func) = stmt {
                let method_name = func.name.to_string();
                ctx.scope.push();
                ctx.scope.define("self".to_string(), class_ty.clone());

                // Define method parameters (skip `self`)
                let mut params = Vec::new();
                for param in func.parameters.args.iter().skip(1) {
                    let param_name = param.parameter.name.to_string();
                    let param_ty = if let Some(ref ann) = param.parameter.annotation {
                        resolve_annotation_expr(ann, ctx)
                    } else {
                        Type::Any
                    };
                    ctx.scope.define(param_name.clone(), param_ty.clone());
                    params.push(HirParam {
                        name: param_name,
                        ty: param_ty,
                        default: None,
                        keyword_only: false,
                        convention: ParamConvention::default(),
                    });
                }

                let return_ty = if let Some(ref ret_ann) = func.returns {
                    resolve_annotation_expr(ret_ann, ctx)
                } else {
                    Type::None
                };

                let method_ft = FunctionType::new(
                    params.iter().map(|p| (p.name.clone(), p.ty.clone())).collect(),
                    return_ty.clone(),
                );

                let body = lower_stmts(&func.body, &method_ft, ctx);
                ctx.scope.pop();

                hir_methods.push(HirFunction {
                    name: method_name,
                    params,
                    return_type: return_ty,
                    body,
                    method_kind: MethodKind::Regular,
                    decorators: vec![],
                    type_params: Vec::new(),
                });
            }
        }
        ctx.current_class = None;
        return Some(HirClass {
            name: class_name,
            fields: vec![],
            methods: hir_methods,
            is_hashable: true,
            is_error_type: false,
            is_protocol: false,
            operator_impls: Vec::new(),
            newtype_inner: None,
            implements_protocols: Vec::new(),
            parent_class: None,
            type_params: Vec::new(),
            is_enum: true,
            enum_variants: variants,
        });
    }

    // For newtype declarations, emit a minimal HirClass
    if let Some(ref inner) = newtype_inner {
        // Lower any methods defined in the newtype body
        let mut hir_methods = Vec::new();
        for stmt in &class_def.body {
            if let Stmt::FunctionDef(func) = stmt {
                let method_name = func.name.to_string();
                if method_name == "__init__" { continue; } // Skip __init__ for newtypes
                ctx.current_class = Some(class_name.clone());
                ctx.scope.push();
                ctx.scope.define("self".to_string(), class_ty.clone());
                let mut params = Vec::new();
                for param in func.parameters.args.iter().skip(1) {
                    let param_name = param.parameter.name.to_string();
                    let param_ty = if let Some(ref ann) = param.parameter.annotation {
                        resolve_annotation_expr(ann, ctx)
                    } else { Type::Any };
                    ctx.scope.define(param_name.clone(), param_ty.clone());
                    params.push(HirParam { name: param_name, ty: param_ty, default: None, keyword_only: false, convention: ParamConvention::default() });
                }
                let return_ty = if let Some(ref ret_ann) = func.returns {
                    resolve_annotation_expr(ret_ann, ctx)
                } else { Type::None };
                let method_ft = FunctionType::new(
                    params.iter().map(|p| (p.name.clone(), p.ty.clone())).collect(),
                    return_ty.clone(),
                );
                let body = lower_stmts(&func.body, &method_ft, ctx);
                ctx.scope.pop();
                ctx.current_class = None;
                hir_methods.push(HirFunction { name: method_name, params, return_type: return_ty, body, method_kind: MethodKind::Regular, decorators: vec![], type_params: Vec::new() });
            }
        }

        return Some(HirClass {
            name: class_name,
            fields: vec![("0".to_string(), inner.clone())], // Single wrapped field
            methods: hir_methods,
            is_hashable: is_hashable_type(inner),
            is_error_type: false,
            is_protocol: false,
            operator_impls: Vec::new(),
            newtype_inner: Some(inner.clone()),
            parent_class: None,
            implements_protocols: Vec::new(),
            type_params: Vec::new(),
            is_enum: false,
            enum_variants: Vec::new(),
        });
    }

    let (all_fields, _method_types) = match &class_ty {
        Type::Class { fields, methods, .. } => (fields.clone(), methods.clone()),
        _ => return None,
    };

    let parent_class_name = get_parent_class(class_def);

    // Separate own fields from inherited fields
    // For struct codegen, we only want the child's own fields (parent is embedded)
    let parent_field_names: Vec<String> = if let Some(ref parent_name) = parent_class_name {
        if let Some(parent_ty) = ctx.class_types.get(parent_name) {
            if let Type::Class { fields: pf, .. } = parent_ty {
                pf.iter().map(|(n, _)| n.clone()).collect()
            } else { vec![] }
        } else { vec![] }
    } else { vec![] };

    let own_fields: Vec<(String, Type)> = all_fields.iter()
        .filter(|(name, _)| !parent_field_names.contains(name))
        .cloned()
        .collect();

    // Determine if all fields are hashable (primitives: int, float, bool, str)
    let is_hashable = all_fields.iter().all(|(_, ty)| is_hashable_type(ty));

    let mut hir_methods = Vec::new();
    let mut operator_impls = Vec::new();

    for stmt in &class_def.body {
        if let Stmt::FunctionDef(func) = stmt {
            let method_name = func.name.to_string();

            // Detect @classmethod and @staticmethod decorators
            let is_classmethod = has_decorator(func, "classmethod");
            let is_staticmethod = has_decorator(func, "staticmethod");
            let method_kind = if is_classmethod {
                MethodKind::ClassMethod
            } else if is_staticmethod {
                MethodKind::StaticMethod
            } else {
                MethodKind::Regular
            };

            // Set current class context for `self` resolution
            ctx.current_class = Some(class_name.clone());
            ctx.current_parent_class = parent_class_name.clone();

            // Push a new scope for the method
            ctx.scope.push();

            // For static methods, don't skip any parameter (no self/cls)
            // For class methods, skip `cls` parameter
            // For regular methods, skip `self` parameter
            let skip_count = if is_staticmethod { 0 } else { 1 }; // classmethod has cls, regular has self

            // Define `self` in scope (for regular methods)
            if !is_staticmethod && !is_classmethod {
                ctx.scope.define("self".to_string(), class_ty.clone());
            }

            // Define method parameters (skip `self`/`cls`)
            let mut params = Vec::new();
            for param in func.parameters.args.iter().skip(skip_count) {
                let param_name = param.parameter.name.to_string();
                let param_ty = if let Some(ref ann) = param.parameter.annotation {
                    resolve_annotation_expr(ann, ctx)
                } else {
                    Type::Any
                };
                ctx.scope.define(param_name.clone(), param_ty.clone());
                params.push(HirParam {
                    name: param_name,
                    ty: param_ty,
                    default: None,
                    keyword_only: false,
                    convention: ParamConvention::default(),
                });
            }

            let return_ty = if method_name == "__init__" {
                Type::None
            } else if let Some(ref ret_ann) = func.returns {
                resolve_annotation_expr(ret_ann, ctx)
            } else {
                Type::None
            };

            // Create a dummy function type for lower_stmts
            let method_ft = FunctionType::new(
                params.iter().map(|p| (p.name.clone(), p.ty.clone())).collect(),
                return_ty.clone(),
            );

            // Lower method body
            let body = lower_stmts(&func.body, &method_ft, ctx);

            // Determine receiver mutability: if any statement assigns to self.field, it's &mut self
            let _is_mutating = method_name == "__init__" || body_contains_field_assign(&body);

            ctx.scope.pop();
            ctx.current_class = None;
            ctx.current_parent_class = None;

            // Collect user-defined decorators (excluding classmethod/staticmethod)
            let method_decorators: Vec<String> = func.decorator_list.iter().filter_map(|d| {
                if let Expr::Name(n) = &d.expression {
                    let name = n.id.to_string();
                    if name != "classmethod" && name != "staticmethod" {
                        Some(name)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }).collect();

            let hir_func = HirFunction {
                name: if method_name == "__init__" { "new".to_string() } else { method_name.clone() },
                params,
                return_type: return_ty,
                body,
                method_kind,
                decorators: method_decorators,
                type_params: Vec::new(),
            };

            // Separate operator dunders from regular methods
            if is_operator_dunder(&method_name) {
                operator_impls.push((method_name, hir_func));
            } else {
                hir_methods.push(hir_func);
            }
        }
    }

    let is_error = ctx.error_types.contains(&class_name);

    // Check which protocols this class satisfies
    let mut implements_protocols = Vec::new();
    for (proto_name, proto_ty) in &ctx.class_types.clone() {
        if let Type::Protocol { methods: proto_methods, .. } = proto_ty {
            // Check if class has all required methods
            let satisfies = proto_methods.iter().all(|(pname, _pft)| {
                _method_types.iter().any(|(mname, _)| mname == pname)
            });
            if satisfies {
                implements_protocols.push(proto_name.clone());
            }
        }
    }

    // Collect PEP 695 type params for the class
    let class_type_params: Vec<String> = if let Some(ref type_params) = class_def.type_params {
        type_params.iter().filter_map(|tp| {
            if let sifr_python_ast::TypeParam::TypeVar(tv) = tp {
                Some(tv.name.to_string())
            } else {
                None
            }
        }).collect()
    } else {
        Vec::new()
    };

    Some(HirClass {
        name: class_name,
        fields: own_fields,
        methods: hir_methods,
        is_hashable,
        is_error_type: is_error,
        is_protocol: false,
        operator_impls,
        newtype_inner: None,
        implements_protocols,
        parent_class: parent_class_name,
        type_params: class_type_params,
        is_enum: false,
        enum_variants: Vec::new(),
    })
}

/// Check if a type is hashable (can derive Hash + Eq).
fn is_hashable_type(ty: &Type) -> bool {
    match ty {
        Type::Int | Type::Bool | Type::Str | Type::None | Type::BigInt => true,
        Type::Float => false, // f64 doesn't implement Hash
        Type::LiteralInt(_) | Type::LiteralBool(_) | Type::LiteralStr(_) => true,
        Type::Tuple(elems) => elems.iter().all(is_hashable_type),
        Type::Class { fields, .. } => fields.iter().all(|(_, t)| is_hashable_type(t)),
        _ => false,
    }
}

/// Check if a method body contains any field assignments (self.field = ...).
fn body_contains_field_assign(stmts: &[HirStmt]) -> bool {
    stmts.iter().any(|s| matches!(s, HirStmt::FieldAssign { .. }))
}

/// Lower a simple expression (literal values only) without requiring a full LowerCtx.
/// Used for collecting default parameter values in the first pass.
fn lower_expr_simple(expr: &Expr) -> Option<HirExpr> {
    match expr {
        Expr::NumberLiteral(num) => {
            match &num.value {
                Number::Int(i) => Some(HirExpr::IntLiteral(i.as_i64()?)),
                Number::Float(f) => Some(HirExpr::FloatLiteral(*f)),
                _ => None,
            }
        }
        Expr::StringLiteral(s) => Some(HirExpr::StringLiteral(s.value.to_str().to_string())),
        Expr::BooleanLiteral(b) => Some(HirExpr::BoolLiteral(b.value)),
        Expr::NoneLiteral(_) => Some(HirExpr::NoneLiteral),
        Expr::UnaryOp(unary) if matches!(unary.op, UnaryOp::USub) => {
            // Handle negative literals like -1
            if let Some(inner) = lower_expr_simple(&unary.operand) {
                match inner {
                    HirExpr::IntLiteral(v) => Some(HirExpr::IntLiteral(-v)),
                    HirExpr::FloatLiteral(v) => Some(HirExpr::FloatLiteral(-v)),
                    _ => None,
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

fn register_builtins(ctx: &mut LowerCtx) {
    // print() accepts any single argument and returns None
    ctx.functions.insert(
        "print".to_string(),
        FunctionType::all_borrow(vec![("value".to_string(), Type::Any)], Type::None),
    );

    // Register built-in error classes.
    // These are compiler built-ins (like int, str, bool) — available without imports.
    // Error hierarchy: Error -> {IOError, ParseError, ValueError, ...}
    //                  IOError -> {FileNotFoundError, PermissionError, ...}

    // --- Base error class ---
    {
        let msg_fields = vec![("message".to_string(), Type::Str)];
        let class_ty = Type::Class {
            name: "Error".to_string(),
            fields: msg_fields.clone(),
            methods: vec![],
            parent_class: None,
        };
        ctx.class_types.insert("Error".to_string(), class_ty.clone());
        ctx.error_types.insert("Error".to_string());
        ctx.functions.insert("Error".to_string(), FunctionType::new(
            vec![("message".to_string(), Type::Str)],
            class_ty,
        ));
    }

    // --- Mid-level error classes (parent: Error) ---
    // IOError has an extra `kind` field for subclass dispatch; constructor accepts only message
    {
        let fields = vec![
            ("message".to_string(), Type::Str),
            ("kind".to_string(), Type::Str),
        ];
        let class_ty = Type::Class {
            name: "IOError".to_string(),
            fields: fields.clone(),
            methods: vec![],
            parent_class: Some("Error".to_string()),
        };
        ctx.class_types.insert("IOError".to_string(), class_ty.clone());
        ctx.error_types.insert("IOError".to_string());
        ctx.functions.insert("IOError".to_string(), FunctionType::new(
            vec![("message".to_string(), Type::Str)],
            class_ty,
        ));
    }
    let other_mid_level_errors = ["ParseError", "ValueError", "DivisionError", "KeyError", "OverflowError"];
    for &error_name in &other_mid_level_errors {
        let fields = vec![("message".to_string(), Type::Str)];
        let class_ty = Type::Class {
            name: error_name.to_string(),
            fields: fields.clone(),
            methods: vec![],
            parent_class: Some("Error".to_string()),
        };
        ctx.class_types.insert(error_name.to_string(), class_ty.clone());
        ctx.error_types.insert(error_name.to_string());
        ctx.functions.insert(error_name.to_string(), FunctionType::new(
            vec![("message".to_string(), Type::Str)],
            class_ty,
        ));
    }

    // --- IOError subclasses (parent: IOError) ---
    let io_subclasses = [
        "FileNotFoundError", "PermissionError", "FileExistsError",
        "IsADirectoryError", "NotADirectoryError", "DirectoryNotEmptyError",
    ];
    for &error_name in &io_subclasses {
        let fields = vec![("message".to_string(), Type::Str)];
        let class_ty = Type::Class {
            name: error_name.to_string(),
            fields: fields.clone(),
            methods: vec![],
            parent_class: Some("IOError".to_string()),
        };
        ctx.class_types.insert(error_name.to_string(), class_ty.clone());
        ctx.error_types.insert(error_name.to_string());
        ctx.functions.insert(error_name.to_string(), FunctionType::new(
            vec![("message".to_string(), Type::Str)],
            class_ty,
        ));
    }

    // --- JSONDecodeError (parent: Error, extra fields: line, column) ---
    // Constructor accepts only message; line/column are populated by intrinsics
    {
        let fields = vec![
            ("message".to_string(), Type::Str),
            ("line".to_string(), Type::Int),
            ("column".to_string(), Type::Int),
        ];
        let class_ty = Type::Class {
            name: "JSONDecodeError".to_string(),
            fields: fields.clone(),
            methods: vec![],
            parent_class: Some("Error".to_string()),
        };
        ctx.class_types.insert("JSONDecodeError".to_string(), class_ty.clone());
        ctx.error_types.insert("JSONDecodeError".to_string());
        ctx.functions.insert("JSONDecodeError".to_string(), FunctionType::new(
            vec![("message".to_string(), Type::Str)],
            class_ty,
        ));
    }

    // --- TOMLDecodeError (parent: Error, extra fields: line, column) ---
    // Constructor accepts only message; line/column are populated by intrinsics
    {
        let fields = vec![
            ("message".to_string(), Type::Str),
            ("line".to_string(), Type::Int),
            ("column".to_string(), Type::Int),
        ];
        let class_ty = Type::Class {
            name: "TOMLDecodeError".to_string(),
            fields: fields.clone(),
            methods: vec![],
            parent_class: Some("Error".to_string()),
        };
        ctx.class_types.insert("TOMLDecodeError".to_string(), class_ty.clone());
        ctx.error_types.insert("TOMLDecodeError".to_string());
        ctx.functions.insert("TOMLDecodeError".to_string(), FunctionType::new(
            vec![("message".to_string(), Type::Str)],
            class_ty,
        ));
    }

    // --- RegexError (parent: Error, extra field: detail) ---
    {
        let fields = vec![
            ("message".to_string(), Type::Str),
            ("detail".to_string(), Type::Str),
        ];
        let class_ty = Type::Class {
            name: "RegexError".to_string(),
            fields: fields.clone(),
            methods: vec![],
            parent_class: Some("Error".to_string()),
        };
        ctx.class_types.insert("RegexError".to_string(), class_ty.clone());
        ctx.error_types.insert("RegexError".to_string());
        // Constructor accepts only message; detail is populated by intrinsics
        ctx.functions.insert("RegexError".to_string(), FunctionType::new(
            vec![("message".to_string(), Type::Str)],
            class_ty,
        ));
    }

    // Build error hierarchy for exhaustiveness checking
    ctx.error_hierarchy.insert("IOError".to_string(), vec![
        "FileNotFoundError".to_string(),
        "PermissionError".to_string(),
        "FileExistsError".to_string(),
        "IsADirectoryError".to_string(),
        "NotADirectoryError".to_string(),
        "DirectoryNotEmptyError".to_string(),
    ]);
}

fn ast_convention_to_param(conv: AstParamConvention, ty: &Type) -> ParamConvention {
    match conv {
        AstParamConvention::Mut => ParamConvention::MutBorrow,
        AstParamConvention::Own => ParamConvention::Own,
        AstParamConvention::Default => {
            // TypeVars (generics) default to Borrow since the concrete type is unknown
            if matches!(ty, Type::TypeVar(_)) {
                ParamConvention::Borrow
            } else if ty.ownership() == OwnershipKind::Copy {
                ParamConvention::Own
            } else {
                ParamConvention::Borrow
            }
        }
    }
}

fn extract_function_type(func: &StmtFunctionDef, ctx: &mut LowerCtx) -> Option<FunctionType> {
    let mut params: Vec<(String, Type, ParamConvention)> = Vec::new();

    for param in &func.parameters.args {
        let name = param.parameter.name.to_string();
        let ty = if let Some(annotation) = &param.parameter.annotation {
            resolve_annotation_expr(annotation, ctx)
        } else {
            ctx.error(format!(
                "parameter '{}' in function '{}' is missing a type annotation",
                name, func.name
            ));
            Type::Any
        };
        let conv = ast_convention_to_param(param.parameter.convention, &ty);
        params.push((name, ty, conv));
    }

    // Vararg parameter (*args) -- becomes Vec<T>
    if let Some(ref vararg) = func.parameters.vararg {
        let name = vararg.name.to_string();
        let elem_ty = if let Some(ref annotation) = vararg.annotation {
            resolve_annotation_expr(annotation, ctx)
        } else {
            ctx.error(format!(
                "vararg parameter '{}' in function '{}' is missing a type annotation",
                name, func.name
            ));
            Type::Any
        };
        let list_ty = Type::List(Box::new(elem_ty));
        let conv = ast_convention_to_param(vararg.convention, &list_ty);
        params.push((name, list_ty, conv));
    }

    // Also include keyword-only parameters
    for param in &func.parameters.kwonlyargs {
        let name = param.parameter.name.to_string();
        let ty = if let Some(annotation) = &param.parameter.annotation {
            resolve_annotation_expr(annotation, ctx)
        } else {
            ctx.error(format!(
                "parameter '{}' in function '{}' is missing a type annotation",
                name, func.name
            ));
            Type::Any
        };
        let conv = ast_convention_to_param(param.parameter.convention, &ty);
        params.push((name, ty, conv));
    }

    let return_type = if let Some(returns) = &func.returns {
        resolve_annotation_expr(returns, ctx)
    } else {
        Type::Any // marker for "needs inference" -- will be inferred from body
    };

    Some(FunctionType {
        params,
        return_type: Box::new(return_type),
    })
}

fn resolve_annotation_expr(expr: &Expr, ctx: &mut LowerCtx) -> Type {
    match expr {
        Expr::Name(name) => {
            // Check type variables first (e.g., T from TypeVar)
            if ctx.type_vars.contains(name.id.as_str()) {
                return Type::TypeVar(name.id.to_string());
            }
            // Check type aliases first
            if let Some(alias_ty) = ctx.scope.lookup_type_alias(&name.id) {
                return alias_ty.clone();
            }
            // Check class types
            if let Some(class_ty) = ctx.class_types.get(name.id.as_str()) {
                return class_ty.clone();
            }
            resolve_type_annotation(&name.id).unwrap_or_else(|| {
                ctx.error(format!("unknown type: '{}'", name.id));
                Type::Any
            })
        }
        Expr::NoneLiteral(_) => Type::None,
        // Union type syntax: int | str (parsed as BinOp with BitOr)
        Expr::BinOp(binop) if matches!(binop.op, Operator::BitOr) => {
            let left = resolve_annotation_expr(&binop.left, ctx);
            let right = resolve_annotation_expr(&binop.right, ctx);
            make_union(vec![left, right])
        }
        // Literal string in type position: "GET" | "POST"
        Expr::StringLiteral(s) => {
            Type::LiteralStr(s.value.to_str().to_string())
        }
        // Literal int in type position: 200 | 404
        Expr::NumberLiteral(num) => {
            match &num.value {
                Number::Int(i) => {
                    if let Some(val) = i.as_i64() {
                        Type::LiteralInt(val)
                    } else {
                        ctx.error("integer literal too large for type annotation".to_string());
                        Type::Any
                    }
                }
                _ => {
                    ctx.error("only integer literals are supported in type annotations".to_string());
                    Type::Any
                }
            }
        }
        // Literal bool in type position: True | False
        Expr::BooleanLiteral(b) => {
            Type::LiteralBool(b.value)
        }
        Expr::Subscript(sub) => {
            // Handle generic type annotations: list[int], dict[str, int], tuple[int, str]
            let base_name = match sub.value.as_ref() {
                Expr::Name(n) => n.id.to_string(),
                _ => {
                    ctx.error("unsupported type annotation base".to_string());
                    return Type::Any;
                }
            };
            match base_name.as_str() {
                "list" => {
                    let elem_ty = resolve_annotation_expr(&sub.slice, ctx);
                    Type::List(Box::new(elem_ty))
                }
                "set" => {
                    let elem_ty = resolve_annotation_expr(&sub.slice, ctx);
                    Type::Set(Box::new(elem_ty))
                }
                "dict" => {
                    // dict[K, V] -- the slice is a Tuple expression
                    match sub.slice.as_ref() {
                        Expr::Tuple(tuple) => {
                            if tuple.elts.len() != 2 {
                                ctx.error("dict type annotation requires exactly 2 type parameters".to_string());
                                return Type::Any;
                            }
                            let key_ty = resolve_annotation_expr(&tuple.elts[0], ctx);
                            let val_ty = resolve_annotation_expr(&tuple.elts[1], ctx);
                            Type::Dict(Box::new(key_ty), Box::new(val_ty))
                        }
                        _ => {
                            ctx.error("dict type annotation requires [K, V] syntax".to_string());
                            Type::Any
                        }
                    }
                }
                "tuple" => {
                    // tuple[A, B, ...] -- the slice is a Tuple expression
                    match sub.slice.as_ref() {
                        Expr::Tuple(tuple) => {
                            let elem_types: Vec<Type> = tuple.elts.iter()
                                .map(|e| resolve_annotation_expr(e, ctx))
                                .collect();
                            Type::Tuple(elem_types)
                        }
                        _ => {
                            // Single-element tuple: tuple[int]
                            let elem_ty = resolve_annotation_expr(&sub.slice, ctx);
                            Type::Tuple(vec![elem_ty])
                        }
                    }
                }
                "Result" => {
                    // Result[T, E] -- the slice is a Tuple expression
                    match sub.slice.as_ref() {
                        Expr::Tuple(tuple) => {
                            if tuple.elts.len() != 2 {
                                ctx.error("Result type annotation requires exactly 2 type parameters".to_string());
                                return Type::Any;
                            }
                            let ok_ty = resolve_annotation_expr(&tuple.elts[0], ctx);
                            let err_ty = resolve_annotation_expr(&tuple.elts[1], ctx);
                            // Enforce: E must be a class extending Error
                            if !is_valid_error_type(&err_ty, ctx) {
                                let err_name = format_type_name(&err_ty);
                                ctx.error(format!(
                                    "`{}` is not a valid error type in Result — use a class extending Error, e.g. `Result[{}, ValueError]`",
                                    err_name,
                                    format_type_name(&ok_ty),
                                ));
                                return Type::Any;
                            }
                            Type::Result(Box::new(ok_ty), Box::new(err_ty))
                        }
                        _ => {
                            ctx.error("Result type annotation requires [T, E] syntax".to_string());
                            Type::Any
                        }
                    }
                }
                "Option" => {
                    // Option[T] -> T | None (sugar)
                    let inner_ty = resolve_annotation_expr(&sub.slice, ctx);
                    make_union(vec![inner_ty, Type::None])
                }
                "TypeGuard" => {
                    // TypeGuard[T] -- type predicate return type
                    let inner_ty = resolve_annotation_expr(&sub.slice, ctx);
                    // Store as the inner type; the function signature handler
                    // will recognize TypeGuard and mark it as a type predicate
                    inner_ty
                }
                "Callable" => {
                    // Callable[[param_types], return_type]
                    // The slice is a Tuple of [List[param_types], return_type]
                    match sub.slice.as_ref() {
                        Expr::Tuple(tuple) => {
                            if tuple.elts.len() != 2 {
                                ctx.error("Callable type requires exactly 2 type parameters: [[param_types], return_type]".to_string());
                                return Type::Any;
                            }
                            // First element should be a list of parameter types
                            let param_types = match &tuple.elts[0] {
                                Expr::List(list) => {
                                    list.elts.iter()
                                        .map(|e| resolve_annotation_expr(e, ctx))
                                        .collect::<Vec<_>>()
                                }
                                _ => {
                                    ctx.error("Callable parameter types must be a list: Callable[[int, str], bool]".to_string());
                                    return Type::Any;
                                }
                            };
                            let return_type = resolve_annotation_expr(&tuple.elts[1], ctx);
                            let conventions = param_types.iter().map(|ty| {
                                if ty.ownership() == OwnershipKind::Copy {
                                    ParamConvention::Own
                                } else {
                                    ParamConvention::Borrow
                                }
                            }).collect();
                            Type::Callable(param_types, conventions, Box::new(return_type))
                        }
                        _ => {
                            ctx.error("Callable type requires [[param_types], return_type] syntax".to_string());
                            Type::Any
                        }
                    }
                }
                _ => {
                    // Check if it's a generic type alias (e.g., Pair[int])
                    if let Some((alias_params, alias_body)) = ctx.scope.lookup_generic_type_alias(&base_name).cloned() {
                        let type_args: Vec<Type> = match sub.slice.as_ref() {
                            Expr::Tuple(tup) => tup.elts.iter().map(|e| resolve_annotation_expr(e, ctx)).collect(),
                            single => vec![resolve_annotation_expr(single, ctx)],
                        };
                        let mut bindings = HashMap::new();
                        for (i, tp) in alias_params.iter().enumerate() {
                            if let Some(arg) = type_args.get(i) {
                                bindings.insert(tp.clone(), arg.clone());
                            }
                        }
                        return substitute_type_vars(&alias_body, &bindings);
                    }
                    // Check if it's a generic class instantiation (e.g., Stack[int])
                    if let Some(class_ty) = ctx.class_types.get(&base_name).cloned() {
                        // Resolve type arguments and substitute into the class type
                        let type_args: Vec<Type> = match sub.slice.as_ref() {
                            Expr::Tuple(tup) => tup.elts.iter().map(|e| resolve_annotation_expr(e, ctx)).collect(),
                            single => vec![resolve_annotation_expr(single, ctx)],
                        };
                        // Build substitution map from class type params to concrete args
                        if let Type::Class { ref fields, ref methods, .. } = class_ty {
                            // Use declared type parameters (from class C[T]) when available,
                            // falling back to scanning fields/methods for backward compatibility.
                            let class_type_params: Vec<String> = ctx.class_declared_type_params
                                .get(&base_name)
                                .cloned()
                                .unwrap_or_else(|| {
                                    fields.iter()
                                        .flat_map(|(_, ty)| { let mut vars = Vec::new(); collect_type_vars(ty, &mut vars); vars })
                                        .chain(methods.iter().flat_map(|(_, ft)| {
                                            let mut vars = Vec::new();
                                            for (_, pt, _) in &ft.params { collect_type_vars(pt, &mut vars); }
                                            collect_type_vars(&ft.return_type, &mut vars);
                                            vars
                                        }))
                                        .collect::<std::collections::HashSet<_>>()
                                        .into_iter().collect::<Vec<_>>()
                                });
                            if !class_type_params.is_empty() && !type_args.is_empty() {
                                let mut bindings = HashMap::new();
                                for (i, tp) in class_type_params.iter().enumerate() {
                                    if let Some(arg) = type_args.get(i) {
                                        bindings.insert(tp.clone(), arg.clone());
                                    }
                                }
                                if !bindings.is_empty() {
                                    let subst_fields: Vec<(String, Type)> = fields.iter()
                                        .map(|(n, t)| (n.clone(), substitute_type_vars(t, &bindings)))
                                        .collect();
                                    let subst_methods: Vec<(String, FunctionType)> = methods.iter()
                                        .map(|(n, ft)| {
                                            let subst_params: Vec<(String, Type, ParamConvention)> = ft.params.iter()
                                                .map(|(pn, pt, pc)| (pn.clone(), substitute_type_vars(pt, &bindings), *pc))
                                                .collect();
                                            let subst_ret = substitute_type_vars(&ft.return_type, &bindings);
                                            (n.clone(), FunctionType { params: subst_params, return_type: Box::new(subst_ret) })
                                        })
                                        .collect();
                                    return Type::Class {
                                        name: base_name.clone(),
                                        fields: subst_fields,
                                        methods: subst_methods,
                                        parent_class: None,
                                    };
                                }
                            }
                        }
                        class_ty
                    } else {
                        ctx.error(format!("unknown generic type: '{}'", base_name));
                        Type::Any
                    }
                }
            }
        }
        _ => {
            ctx.error("unsupported type annotation expression".to_string());
            Type::Any
        }
    }
}

fn lower_function(func: &StmtFunctionDef, ctx: &mut LowerCtx) -> Option<HirFunction> {
    let ft = ctx.functions.get(&func.name.to_string())?.clone();

    ctx.scope.push();

    // Define parameters in scope, handling defaults
    let mut params = Vec::new();

    // Regular args
    for (i, param_def) in func.parameters.args.iter().enumerate() {
        let name = param_def.parameter.name.to_string();
        let ty = ft.params.get(i).map(|(_, t, _)| t.clone()).unwrap_or(Type::Any);
        ctx.scope.define(name.clone(), ty.clone());

        let default = param_def.default.as_ref().and_then(|d| lower_expr(d, ctx));

        let convention = ast_convention_to_param(param_def.parameter.convention, &ty);

        params.push(HirParam {
            name,
            ty,
            default,
            keyword_only: false,
            convention,
        });
    }

    // Vararg parameter (*args) -- becomes Vec<T>
    if let Some(ref vararg) = func.parameters.vararg {
        let name = vararg.name.to_string();
        let regular_count = func.parameters.args.len();
        let ty = ft.params.get(regular_count).map(|(_, t, _)| t.clone()).unwrap_or(Type::Any);
        ctx.scope.define(name.clone(), ty.clone());

        let convention = ast_convention_to_param(vararg.convention, &ty);
        params.push(HirParam {
            name,
            ty,
            default: None,
            keyword_only: false,
            convention,
        });
    }

    // Keyword-only args (after * separator)
    let regular_count = func.parameters.args.len() + if func.parameters.vararg.is_some() { 1 } else { 0 };
    for (i, param_def) in func.parameters.kwonlyargs.iter().enumerate() {
        let name = param_def.parameter.name.to_string();
        let ty = ft.params.get(regular_count + i).map(|(_, t, _)| t.clone()).unwrap_or(Type::Any);
        ctx.scope.define(name.clone(), ty.clone());

        let default = param_def.default.as_ref().and_then(|d| lower_expr(d, ctx));

        let convention = ast_convention_to_param(param_def.parameter.convention, &ty);

        params.push(HirParam {
            name,
            ty,
            default,
            keyword_only: true,
            convention,
        });
    }

    // Populate borrowed_params for escape analysis in lower_return / lower_let
    // A param is "borrowed" (escape-unsafe) if its convention is Borrow and its type is Move.
    // Exclude TypeVar parameters: generics are monomorphized by Rust and ownership is handled
    // by the Rust compiler, not by Sifr's escape analysis.
    ctx.borrowed_params.clear();
    for param in &params {
        if param.convention == ParamConvention::Borrow
            && param.ty.ownership() == OwnershipKind::Move
            && !matches!(param.ty, Type::TypeVar(_))
        {
            ctx.borrowed_params.insert(param.name.clone());
        }
    }

    // Lower body
    let body = lower_stmts(&func.body, &ft, ctx);

    ctx.borrowed_params.clear();

    ctx.scope.pop();

    // Infer return type if not explicitly annotated (marked as Type::Any)
    let inferred_return_type = if *ft.return_type == Type::Any && func.returns.is_none() {
        let return_types = collect_return_types(&body);
        if return_types.is_empty() {
            Type::None // no return statements -> None
        } else if return_types.len() == 1 {
            return_types.into_iter().next().unwrap()
        } else {
            // Multiple return types -> union
            let mut members: Vec<Type> = return_types.into_iter().collect();
            members.sort_by(|a, b| a.display_name().cmp(&b.display_name()));
            members.dedup();
            if members.len() == 1 {
                members.into_iter().next().unwrap()
            } else {
                Type::Union(members)
            }
        }
    } else {
        *ft.return_type
    };

    // Collect user-defined decorators (excluding classmethod/staticmethod)
    let decorators: Vec<String> = func.decorator_list.iter().filter_map(|d| {
        if let Expr::Name(n) = &d.expression {
            let name = n.id.to_string();
            if name != "classmethod" && name != "staticmethod" {
                Some(name)
            } else {
                None
            }
        } else {
            None
        }
    }).collect();

    // Collect type parameters for generic functions
    let type_params = ctx.generic_functions.get(&func.name.to_string()).cloned().unwrap_or_default();

    Some(HirFunction {
        name: func.name.to_string(),
        params,
        return_type: inferred_return_type,
        body,
        method_kind: MethodKind::Regular,
        decorators,
        type_params,
    })
}

fn lower_stmts(stmts: &[Stmt], func_type: &FunctionType, ctx: &mut LowerCtx) -> Vec<HirStmt> {
    let mut result = Vec::new();
    for stmt in stmts {
        // Handle chained assignment (x = y = z = 0) by expanding into multiple statements
        if let Stmt::Assign(assign) = stmt {
            if assign.targets.len() > 1 {
                let expanded = lower_chained_assign(assign, ctx);
                result.extend(expanded);
                continue;
            }
        }
        if let Some(hir_stmt) = lower_stmt(stmt, func_type, ctx) {
            result.push(hir_stmt);
        }
    }
    result
}

fn lower_stmt(stmt: &Stmt, func_type: &FunctionType, ctx: &mut LowerCtx) -> Option<HirStmt> {
    match stmt {
        Stmt::AnnAssign(ann) => lower_ann_assign(ann, ctx),
        Stmt::Assign(assign) => lower_assign(assign, ctx),
        Stmt::AugAssign(aug) => lower_aug_assign(aug, ctx),
        Stmt::Return(ret) => lower_return(ret, func_type, ctx),
        Stmt::Expr(expr_stmt) => {
            // Check if this is a yield expression used as a statement
            if let Expr::Yield(yield_expr) = expr_stmt.value.as_ref() {
                if let Some(ref val) = yield_expr.value {
                    let value = lower_expr(val, ctx)?;
                    return Some(HirStmt::Yield { value });
                } else {
                    ctx.error("yield without a value is not supported".to_string());
                    return None;
                }
            }
            let expr = lower_expr(&expr_stmt.value, ctx)?;
            // #[must_use] enforcement: Result values must not be silently discarded
            let expr_ty = expr.ty();
            if matches!(expr_ty, Type::Result(_, _)) {
                ctx.error(format!(
                    "unused Result value of type '{}' must be used. Use 'let _ = expr' to explicitly discard",
                    expr_ty.display_name()
                ));
            }
            Some(HirStmt::Expr { expr })
        }
        Stmt::If(if_stmt) => lower_if(if_stmt, func_type, ctx),
        Stmt::While(while_stmt) => lower_while(while_stmt, func_type, ctx),
        Stmt::For(for_stmt) => lower_for(for_stmt, func_type, ctx),
        Stmt::Break(_) => {
            if !ctx.in_loop() {
                ctx.error("'break' outside of loop".to_string());
                return None;
            }
            Some(HirStmt::Break)
        }
        Stmt::Continue(_) => {
            if !ctx.in_loop() {
                ctx.error("'continue' outside of loop".to_string());
                return None;
            }
            Some(HirStmt::Continue)
        }
        Stmt::Pass(_) => Some(HirStmt::Pass),
        Stmt::Delete(del_stmt) => {
            if del_stmt.targets.len() != 1 {
                ctx.error("del with multiple targets not supported".to_string());
                return None;
            }
            match &del_stmt.targets[0] {
                Expr::Subscript(sub) => {
                    let object = lower_expr(&sub.value, ctx)?;
                    let index = lower_expr(&sub.slice, ctx)?;
                    Some(HirStmt::Delete { object, index })
                }
                _ => {
                    ctx.error("del is only supported for collection items (del d[key], del a[i])".to_string());
                    None
                }
            }
        }
        Stmt::Assert(assert_stmt) => {
            let test = lower_expr(&assert_stmt.test, ctx)?;
            let msg = if let Some(ref msg_expr) = assert_stmt.msg {
                Some(lower_expr(msg_expr, ctx)?)
            } else {
                None
            };
            Some(HirStmt::Assert { test, msg })
        }
        Stmt::Raise(raise_stmt) => {
            if let Some(ref exc) = raise_stmt.exc {
                // Check if the raise expression is a string literal — disallow raise "message"
                if matches!(exc.as_ref(), Expr::StringLiteral(_) | Expr::FString(_)) {
                    ctx.error("raise requires an Error class instance — `raise \"message\"` is not allowed, use e.g. `raise ValueError(\"message\")`".to_string());
                    return None;
                }
                let value = lower_expr(exc, ctx)?;
                // Verify the raised value is an error type
                let raised_ty = value.ty();
                if !is_valid_error_type(raised_ty, ctx) {
                    let ty_name = format_type_name(raised_ty);
                    ctx.error(format!(
                        "raise requires an Error class instance — `{}` is not an Error class",
                        ty_name
                    ));
                    return None;
                }
                Some(HirStmt::Raise { value })
            } else {
                ctx.error("bare 'raise' without an expression is not supported".to_string());
                None
            }
        }
        Stmt::With(with_stmt) => {
            if with_stmt.items.is_empty() {
                ctx.error("with statement must have at least one item".to_string());
                return None;
            }
            let mut items = Vec::new();
            ctx.scope.push();
            for item in &with_stmt.items {
                let value = lower_expr(&item.context_expr, ctx)?;
                let var_name = if let Some(ref vars) = item.optional_vars {
                    match vars.as_ref() {
                        Expr::Name(n) => n.id.to_string(),
                        _ => {
                            ctx.error("with target must be a simple name".to_string());
                            return None;
                        }
                    }
                } else {
                    format!("_with_val_{}", items.len())
                };
                let val_ty = value.ty().clone();
                // Check if the type implements the ContextManager protocol (__enter__/__exit__)
                let has_context_manager = match &val_ty {
                    Type::Class { methods, .. } => {
                        let has_enter = methods.iter().any(|(name, _)| name == "__enter__");
                        let has_exit = methods.iter().any(|(name, _)| name == "__exit__");
                        if has_enter && has_exit {
                            true
                        } else if has_enter || has_exit {
                            ctx.error(format!(
                                "type used in 'with' statement must implement both __enter__ and __exit__ methods"
                            ));
                            false
                        } else {
                            ctx.error(format!(
                                "type '{}' does not implement the ContextManager protocol (missing __enter__ and __exit__ methods)",
                                match &val_ty { Type::Class { name, .. } => name.clone(), _ => "unknown".to_string() }
                            ));
                            false
                        }
                    }
                    _ => {
                        // Non-class types don't have methods — can't be context managers
                        ctx.error(format!(
                            "type used in 'with' statement must implement the ContextManager protocol (__enter__/__exit__)"
                        ));
                        false
                    }
                };
                // If the type has __enter__, the variable is bound to __enter__()'s return type
                // We resolve the actual class type from ctx.class_types to get full fields/methods
                let bound_ty = if has_context_manager {
                    if let Type::Class { methods, .. } = &val_ty {
                        let ret_ty = methods.iter()
                            .find(|(name, _)| name == "__enter__")
                            .map(|(_, ft)| (*ft.return_type).clone())
                            .unwrap_or(val_ty.clone());
                        // If the return type is a class, look up the fully-defined version
                        if let Type::Class { name: ret_name, .. } = &ret_ty {
                            ctx.class_types.get(ret_name).cloned().unwrap_or(ret_ty)
                        } else {
                            ret_ty
                        }
                    } else {
                        val_ty.clone()
                    }
                } else {
                    val_ty.clone()
                };
                ctx.scope.define(var_name.clone(), bound_ty);
                items.push((var_name, value, has_context_manager));
            }
            let body = lower_stmts(&with_stmt.body, func_type, ctx);
            ctx.scope.pop();
            Some(HirStmt::With { items, body })
        }
        Stmt::Try(try_stmt) => {
            let prev_in_try = ctx.in_try_block;
            let prev_try_errors = std::mem::take(&mut ctx.try_block_error_types);
            ctx.in_try_block = true;
            let body = lower_stmts(&try_stmt.body, func_type, ctx);
            ctx.in_try_block = prev_in_try;
            let mut try_error_types = std::mem::replace(&mut ctx.try_block_error_types, prev_try_errors);

            // Also collect error types from raise statements in the body
            collect_raise_error_types(&body, &mut try_error_types);

            let mut handlers = Vec::new();
            let mut has_catch_all = false;
            let mut covered_types = std::collections::HashSet::new();

            for handler in &try_stmt.handlers {
                let ExceptHandler::ExceptHandler(h) = handler;
                let error_type = if let Some(ref type_expr) = h.type_ {
                    if let Expr::Name(n) = type_expr.as_ref() {
                        Some(n.id.to_string())
                    } else {
                        None
                    }
                } else {
                    None
                };
                let name = h.name.as_ref().map(|n| n.to_string());

                // Check if this is a catch-all (except Error) or a specific handler
                if let Some(ref et) = error_type {
                    if et == "Error" {
                        has_catch_all = true;
                    } else {
                        // Validate the except type is a known error class
                        if !ctx.error_types.contains(et) {
                            ctx.error(format!(
                                "`{}` in except arm is not a known error class — use a class extending Error",
                                et
                            ));
                        }
                        covered_types.insert(et.clone());
                    }
                } else {
                    // Bare except (no type) — acts as catch-all
                    has_catch_all = true;
                }

                // Define the error variable in scope if named
                ctx.scope.push();
                if let Some(ref var_name) = name {
                    let error_var_ty = if let Some(ref et) = error_type {
                        if et == "Error" {
                            // catch-all: bind as the base Error type
                            ctx.class_types.get("Error").cloned().unwrap_or_else(|| Type::Class {
                                name: "Error".to_string(),
                                fields: vec![("message".to_string(), Type::Str)],
                                methods: vec![],
                                parent_class: None,
                            })
                        } else if let Some(class_ty) = ctx.class_types.get(et) {
                            class_ty.clone()
                        } else {
                            // Unknown error type — already reported above
                            Type::Class {
                                name: et.clone(),
                                fields: vec![("message".to_string(), Type::Str)],
                                methods: vec![],
                                parent_class: None,
                            }
                        }
                    } else {
                        // Bare except — error variable is base Error type
                        ctx.class_types.get("Error").cloned().unwrap_or_else(|| Type::Class {
                            name: "Error".to_string(),
                            fields: vec![("message".to_string(), Type::Str)],
                            methods: vec![],
                            parent_class: None,
                        })
                    };
                    ctx.scope.define(var_name.clone(), error_var_ty);
                }
                let handler_body = lower_stmts(&h.body, func_type, ctx);
                ctx.scope.pop();

                // Resolve the error type for codegen
                let error_resolved_type = error_type.as_ref().and_then(|et| {
                    ctx.class_types.get(et).cloned()
                });
                handlers.push(HirExceptHandler {
                    error_type,
                    error_resolved_type,
                    name,
                    body: handler_body,
                });
            }

            // Exhaustiveness checking: if no catch-all, all error types must be covered
            // A parent type covers all its children (e.g., except IOError covers FileNotFoundError)
            // Subclasses partially cover their parent (e.g., except FileNotFoundError covers IOError::FileNotFound)
            if !has_catch_all && !try_error_types.is_empty() {
                // Expand covered_types: if a parent is covered, all its children are covered
                let mut expanded_covered = covered_types.clone();
                for covered in &covered_types {
                    if let Some(children) = ctx.error_hierarchy.get(covered) {
                        for child in children {
                            expanded_covered.insert(child.clone());
                        }
                    }
                }
                // Check if subclasses fully cover their parent
                // If all children of a parent are covered, the parent is covered
                for (parent, children) in &ctx.error_hierarchy {
                    if try_error_types.contains(parent) && !expanded_covered.contains(parent) {
                        let all_children_covered = children.iter().all(|c| expanded_covered.contains(c));
                        if all_children_covered {
                            expanded_covered.insert(parent.clone());
                        }
                    }
                }
                let uncovered: Vec<String> = try_error_types.iter()
                    .filter(|et| !expanded_covered.contains(*et))
                    .cloned()
                    .collect();
                if !uncovered.is_empty() {
                    let mut sorted = uncovered;
                    sorted.sort();
                    ctx.error(format!(
                        "except arms do not cover all error types from try body — uncovered: {}. Add `except Error as e` as a catch-all or add specific except arms",
                        sorted.join(", ")
                    ));
                }
            }

            let body_error_types: Vec<String> = try_error_types.into_iter().collect();
            Some(HirStmt::TryExcept { body, handlers, body_error_types })
        }
        Stmt::FunctionDef(func) => {
            // Nested function definition (def inside def)
            // Extract the function type (params + return type)
            let ft = extract_function_type(func, ctx)?;

            // Register the nested function in the current scope so it can be called
            ctx.functions.insert(func.name.to_string(), ft.clone());

            // Lower the nested function body
            ctx.scope.push();

            // Define parameters in scope
            let mut params = Vec::new();
            for (i, param_def) in func.parameters.args.iter().enumerate() {
                let name = param_def.parameter.name.to_string();
                let ty = ft.params.get(i).map(|(_, t, _)| t.clone()).unwrap_or(Type::Any);
                ctx.scope.define(name.clone(), ty.clone());
                let default = param_def.default.as_ref().and_then(|d| lower_expr(d, ctx));
                params.push(HirParam {
                    name,
                    ty,
                    default,
                    keyword_only: false,
                    convention: ParamConvention::default(),
                });
            }

            // Vararg parameter (*args)
            if let Some(ref vararg) = func.parameters.vararg {
                let name = vararg.name.to_string();
                let regular_count = func.parameters.args.len();
                let ty = ft.params.get(regular_count).map(|(_, t, _)| t.clone()).unwrap_or(Type::Any);
                ctx.scope.define(name.clone(), ty.clone());
                params.push(HirParam {
                    name,
                    ty,
                    default: None,
                    keyword_only: false,
                    convention: ParamConvention::default(),
                });
            }

            // Keyword-only args
            let regular_count = func.parameters.args.len() + if func.parameters.vararg.is_some() { 1 } else { 0 };
            for (i, param_def) in func.parameters.kwonlyargs.iter().enumerate() {
                let name = param_def.parameter.name.to_string();
                let ty = ft.params.get(regular_count + i).map(|(_, t, _)| t.clone()).unwrap_or(Type::Any);
                ctx.scope.define(name.clone(), ty.clone());
                let default = param_def.default.as_ref().and_then(|d| lower_expr(d, ctx));
                params.push(HirParam {
                    name,
                    ty,
                    default,
                    keyword_only: true,
                    convention: ParamConvention::default(),
                });
            }

            let body = lower_stmts(&func.body, &ft, ctx);
            ctx.scope.pop();

            // Infer return type if not explicitly annotated
            let inferred_return_type = if *ft.return_type == Type::Any && func.returns.is_none() {
                let return_types = collect_return_types(&body);
                if return_types.is_empty() {
                    Type::None
                } else if return_types.len() == 1 {
                    return_types.into_iter().next().unwrap()
                } else {
                    let mut members: Vec<Type> = return_types.into_iter().collect();
                    members.sort_by(|a, b| a.display_name().cmp(&b.display_name()));
                    members.dedup();
                    if members.len() == 1 {
                        members.into_iter().next().unwrap()
                    } else {
                        Type::Union(members)
                    }
                }
            } else {
                *ft.return_type
            };

            // Collect user-defined decorators
            let decorators: Vec<String> = func.decorator_list.iter().filter_map(|d| {
                if let Expr::Name(n) = &d.expression {
                    let name = n.id.to_string();
                    if name != "classmethod" && name != "staticmethod" {
                        Some(name)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }).collect();

            Some(HirStmt::NestedFunction {
                func: HirFunction {
                    name: func.name.to_string(),
                    params,
                    return_type: inferred_return_type,
                    body,
                    method_kind: MethodKind::Regular,
                    decorators,
                    type_params: Vec::new(),
                },
            })
        }
        Stmt::Match(match_stmt) => lower_match(match_stmt, func_type, ctx),
        _ => {
            ctx.error("unsupported statement type".to_string());
            None
        }
    }
}

fn lower_match(
    match_stmt: &StmtMatch,
    func_type: &FunctionType,
    ctx: &mut LowerCtx,
) -> Option<HirStmt> {
    let subject = lower_expr(&match_stmt.subject, ctx)?;
    let subject_ty = subject.ty().clone();

    let mut arms = Vec::new();
    for case in &match_stmt.cases {
        ctx.scope.push();

        let pattern = lower_pattern(&case.pattern, &subject_ty, ctx)?;

        // Bind captured variables into scope
        bind_pattern_vars(&pattern, ctx);

        let guard = if let Some(ref g) = case.guard {
            let guard_expr = lower_expr(g, ctx)?;
            let guard_ty = guard_expr.ty();
            if *guard_ty != Type::Bool && *guard_ty != Type::Any {
                ctx.error(format!(
                    "match guard must be a bool expression, got '{}'",
                    guard_ty.display_name()
                ));
            }
            Some(guard_expr)
        } else {
            None
        };

        let body = lower_stmts(&case.body, func_type, ctx);

        ctx.scope.pop();

        arms.push(HirMatchArm { pattern, guard, body });
    }

    // Exhaustiveness check: verify all variants of the subject type are covered
    let has_wildcard = arms.iter().any(|arm| matches!(arm.pattern, HirPattern::Wildcard));
    let has_capture_without_guard = arms.iter().any(|arm| {
        matches!(arm.pattern, HirPattern::Capture { .. }) && arm.guard.is_none()
    });

    if !has_wildcard && !has_capture_without_guard {
        if let Type::Union(members) = &subject_ty {
            // Collect covered types from arms
            let mut covered_none = false;
            let mut covered_classes: std::collections::HashSet<String> = std::collections::HashSet::new();
            let mut covered_types: std::collections::HashSet<String> = std::collections::HashSet::new();
            let mut covered_literal_strs: std::collections::HashSet<String> = std::collections::HashSet::new();
            let mut covered_literal_ints: std::collections::HashSet<i64> = std::collections::HashSet::new();
            let mut covered_literal_bools: std::collections::HashSet<bool> = std::collections::HashSet::new();

            fn collect_literal_coverage(
                pattern: &HirPattern,
                covered_literal_strs: &mut std::collections::HashSet<String>,
                covered_literal_ints: &mut std::collections::HashSet<i64>,
                covered_literal_bools: &mut std::collections::HashSet<bool>,
            ) {
                if let HirPattern::Literal { value } = pattern {
                    match value {
                        HirExpr::StringLiteral(s) => { covered_literal_strs.insert(s.clone()); }
                        HirExpr::IntLiteral(n) => { covered_literal_ints.insert(*n); }
                        HirExpr::BoolLiteral(b) => { covered_literal_bools.insert(*b); }
                        _ => {}
                    }
                }
            }

            for arm in &arms {
                match &arm.pattern {
                    HirPattern::None => { covered_none = true; }
                    HirPattern::Class { class_name, .. } => { covered_classes.insert(class_name.clone()); }
                    HirPattern::Capture { ty, .. } if arm.guard.is_none() => {
                        covered_types.insert(ty.display_name());
                    }
                    HirPattern::Literal { .. } => {
                        collect_literal_coverage(&arm.pattern, &mut covered_literal_strs, &mut covered_literal_ints, &mut covered_literal_bools);
                    }
                    HirPattern::Or { patterns } => {
                        for p in patterns {
                            match p {
                                HirPattern::None => { covered_none = true; }
                                HirPattern::Class { class_name, .. } => { covered_classes.insert(class_name.clone()); }
                                HirPattern::Literal { .. } => {
                                    collect_literal_coverage(p, &mut covered_literal_strs, &mut covered_literal_ints, &mut covered_literal_bools);
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }

            // Check each union member is covered
            let mut uncovered: Vec<String> = Vec::new();
            for member in members {
                match member {
                    Type::None => {
                        if !covered_none { uncovered.push("None".to_string()); }
                    }
                    Type::Class { name, .. } => {
                        if !covered_classes.contains(name) && !covered_types.contains(name) {
                            uncovered.push(name.clone());
                        }
                    }
                    Type::Int => {
                        if !covered_types.contains("int") && !covered_classes.contains("int") {
                            uncovered.push("int".to_string());
                        }
                    }
                    Type::Str => {
                        if !covered_types.contains("str") && !covered_classes.contains("str") {
                            uncovered.push("str".to_string());
                        }
                    }
                    Type::Float => {
                        if !covered_types.contains("float") && !covered_classes.contains("float") {
                            uncovered.push("float".to_string());
                        }
                    }
                    Type::Bool => {
                        if !covered_types.contains("bool") && !covered_classes.contains("bool") {
                            uncovered.push("bool".to_string());
                        }
                    }
                    Type::LiteralStr(s) => {
                        if !covered_literal_strs.contains(s) {
                            uncovered.push(format!("\"{}\"", s));
                        }
                    }
                    Type::LiteralInt(n) => {
                        if !covered_literal_ints.contains(n) {
                            uncovered.push(n.to_string());
                        }
                    }
                    Type::LiteralBool(b) => {
                        if !covered_literal_bools.contains(b) {
                            uncovered.push(b.to_string());
                        }
                    }
                    _ => {}
                }
            }

            if !uncovered.is_empty() {
                ctx.error(format!(
                    "non-exhaustive match: type '{}' has uncovered variants: {} — add matching case(s) or `case _:`",
                    subject_ty.display_name(),
                    uncovered.join(", ")
                ));
            }
        }

        // Check enum exhaustiveness
        if let Type::Enum { ref name, ref variants } = subject_ty {
            let mut covered_variants: std::collections::HashSet<String> = std::collections::HashSet::new();
            for arm in &arms {
                if let HirPattern::Value { path } = &arm.pattern {
                    if path.len() == 2 {
                        covered_variants.insert(path[1].clone());
                    }
                }
                if let HirPattern::Or { patterns } = &arm.pattern {
                    for p in patterns {
                        if let HirPattern::Value { path } = p {
                            if path.len() == 2 {
                                covered_variants.insert(path[1].clone());
                            }
                        }
                    }
                }
            }
            let uncovered: Vec<&String> = variants.iter()
                .map(|(v, _)| v)
                .filter(|v| !covered_variants.contains(*v))
                .collect();
            if !uncovered.is_empty() {
                ctx.error(format!(
                    "non-exhaustive match: enum '{}' has uncovered variants: {} — add matching case(s) or `case _:`",
                    name,
                    uncovered.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
                ));
            }
        }

        // For non-union, non-enum types with only literal/guarded patterns, require a wildcard
        if !matches!(subject_ty, Type::Union(_)) && !matches!(subject_ty, Type::Enum { .. }) {
            let all_literal_or_guarded = arms.iter().all(|arm| {
                matches!(arm.pattern, HirPattern::Literal { .. })
                    || matches!(arm.pattern, HirPattern::Or { .. })
                    || arm.guard.is_some()
            });
            if all_literal_or_guarded {
                ctx.error(format!(
                    "non-exhaustive match: type '{}' cannot be fully covered by literal patterns — add `case _:` to handle remaining values",
                    subject_ty.display_name()
                ));
            }
        }
    }

    Some(HirStmt::Match { subject, subject_ty, arms })
}

fn lower_pattern(
    pattern: &Pattern,
    subject_ty: &Type,
    ctx: &mut LowerCtx,
) -> Option<HirPattern> {
    match pattern {
        Pattern::MatchAs(pat_as) => {
            if pat_as.pattern.is_none() && pat_as.name.is_none() {
                // `case _:` — wildcard
                return Some(HirPattern::Wildcard);
            }
            if let Some(name) = &pat_as.name {
                let var_name = name.to_string();
                if let Some(inner_pat) = &pat_as.pattern {
                    // `case SomePattern as x:` — match inner pattern, bind to x
                    let inner = lower_pattern(inner_pat, subject_ty, ctx)?;
                    // For now, treat as capture with narrowed type
                    let narrowed_ty = pattern_narrowed_type(&inner, subject_ty, ctx);
                    let _ = inner; // inner pattern info embedded in capture
                    return Some(HirPattern::Capture { name: var_name, ty: narrowed_ty });
                } else {
                    // `case x:` — capture pattern
                    return Some(HirPattern::Capture { name: var_name, ty: subject_ty.clone() });
                }
            }
            if let Some(inner_pat) = &pat_as.pattern {
                return lower_pattern(inner_pat, subject_ty, ctx);
            }
            Some(HirPattern::Wildcard)
        }
        Pattern::MatchSingleton(singleton) => {
            match &singleton.value {
                Singleton::None => Some(HirPattern::None),
                Singleton::True => Some(HirPattern::Literal {
                    value: HirExpr::BoolLiteral(true),
                }),
                Singleton::False => Some(HirPattern::Literal {
                    value: HirExpr::BoolLiteral(false),
                }),
            }
        }
        Pattern::MatchValue(val_pat) => {
            // Could be a literal or an attribute access like Color.RED
            match val_pat.value.as_ref() {
                Expr::Attribute(attr) => {
                    let obj_name = match attr.value.as_ref() {
                        Expr::Name(n) => n.id.to_string(),
                        _ => {
                            ctx.error("complex attribute pattern not supported".to_string());
                            return None;
                        }
                    };
                    let attr_name = attr.attr.to_string();
                    Some(HirPattern::Value { path: vec![obj_name, attr_name] })
                }
                _ => {
                    // Try to lower as a literal expression
                    let expr = lower_expr(val_pat.value.as_ref(), ctx)?;
                    Some(HirPattern::Literal { value: expr })
                }
            }
        }
        Pattern::MatchOr(or_pat) => {
            let mut patterns = Vec::new();
            for p in &or_pat.patterns {
                patterns.push(lower_pattern(p, subject_ty, ctx)?);
            }
            Some(HirPattern::Or { patterns })
        }
        Pattern::MatchClass(class_pat) => {
            let class_name = match class_pat.cls.as_ref() {
                Expr::Name(n) => n.id.to_string(),
                _ => {
                    ctx.error("class pattern class name must be a simple name".to_string());
                    return None;
                }
            };

            // Resolve the class type to get field types
            let class_ty = ctx.class_types.get(&class_name).cloned();

            let mut fields = Vec::new();
            for kw in &class_pat.arguments.keywords {
                let field_name = kw.attr.to_string();
                let field_ty = if let Some(Type::Class { fields: class_fields, .. }) = &class_ty {
                    let found = class_fields.iter()
                        .find(|(n, _)| n == &field_name)
                        .map(|(_, t)| t.clone());
                    if found.is_none() {
                        ctx.error(format!(
                            "class '{}' has no field '{}' — available fields: {}",
                            class_name,
                            field_name,
                            class_fields.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>().join(", ")
                        ));
                        return None;
                    }
                    found.unwrap()
                } else {
                    Type::Any
                };
                let field_pattern = lower_pattern(&kw.pattern, &field_ty, ctx)?;
                fields.push((field_name, field_pattern));
            }

            Some(HirPattern::Class { class_name, fields })
        }
        Pattern::MatchSequence(seq_pat) => {
            if seq_pat.patterns.is_empty() {
                return Some(HirPattern::Tuple { elements: vec![] });
            }
            let elem_types: Vec<Type> = if let Type::Tuple(ref elems) = *subject_ty {
                elems.clone()
            } else {
                vec![Type::Any; seq_pat.patterns.len()]
            };
            let mut elements = Vec::new();
            for (i, pat) in seq_pat.patterns.iter().enumerate() {
                let elem_ty = elem_types.get(i).cloned().unwrap_or(Type::Any);
                if let Some(lowered) = lower_pattern(pat, &elem_ty, ctx) {
                    elements.push(lowered);
                } else {
                    return None;
                }
            }
            Some(HirPattern::Tuple { elements })
        }
        Pattern::MatchMapping(_) => {
            ctx.error("mapping patterns in match are not yet supported".to_string());
            None
        }
        Pattern::MatchStar(_) => {
            ctx.error("star patterns in match are not yet supported".to_string());
            None
        }
    }
}

fn pattern_narrowed_type(pattern: &HirPattern, subject_ty: &Type, ctx: &LowerCtx) -> Type {
    match pattern {
        HirPattern::None => Type::None,
        HirPattern::Class { class_name, .. } => {
            // Look up the class type
            if let Some(class_ty) = ctx.class_types.get(class_name) {
                class_ty.clone()
            } else {
                subject_ty.clone()
            }
        }
        _ => subject_ty.clone(),
    }
}

fn bind_pattern_vars(pattern: &HirPattern, ctx: &mut LowerCtx) {
    match pattern {
        HirPattern::Capture { name, ty } => {
            ctx.scope.define(name.clone(), ty.clone());
        }
        HirPattern::Class { fields, .. } => {
            for (_, field_pat) in fields {
                bind_pattern_vars(field_pat, ctx);
            }
        }
        HirPattern::Or { patterns } => {
            // Bind from first pattern (all OR alternatives should bind same names)
            if let Some(first) = patterns.first() {
                bind_pattern_vars(first, ctx);
            }
        }
        HirPattern::Tuple { elements } => {
            for elem in elements {
                bind_pattern_vars(elem, ctx);
            }
        }
        _ => {}
    }
}

fn lower_ann_assign(ann: &StmtAnnAssign, ctx: &mut LowerCtx) -> Option<HirStmt> {
    let name = match ann.target.as_ref() {
        Expr::Name(n) => n.id.to_string(),
        _ => {
            ctx.error("annotated assignment target must be a simple name".to_string());
            return None;
        }
    };

    let declared_type = resolve_annotation_expr(&ann.annotation, ctx);

    let value = if let Some(val) = &ann.value {
        let mut expr = lower_expr(val, ctx)?;
        let expr_ty = expr.ty().clone();
        // Inside try blocks, auto-unwrap Result[T, E] when declared type is T
        if ctx.in_try_block {
            if let Type::Result(ref ok_ty, ref err_ty) = expr_ty {
                if ok_ty.as_ref().is_assignable_to(&declared_type) {
                    // Track the error type for exhaustiveness checking
                    if let Type::Class { name, .. } = err_ty.as_ref() {
                        ctx.try_block_error_types.insert(name.clone());
                    }
                    expr = HirExpr::QuestionMark {
                        expr: Box::new(expr),
                        ty: declared_type.clone(),
                    };
                }
            }
        }
        // Type check: value must be assignable to declared type
        let final_ty = expr.ty().clone();
        // int literals are assignable to bigint (coercion: 42 -> BigInt::from(42))
        let is_int_to_bigint = final_ty == Type::Int && declared_type == Type::BigInt;
        if !is_int_to_bigint && !final_ty.is_assignable_to(&declared_type) {
            ctx.error(format!(
                "type mismatch: expected '{}', got '{}'",
                declared_type.display_name(),
                final_ty.display_name()
            ));
        }
        expr
    } else {
        ctx.error(format!("variable '{}' must be initialized", name));
        return None;
    };

    // Track move: if RHS is a variable name with Move ownership, mark it as moved.
    // Also check escape analysis: storing a borrowed parameter into a local variable
    // would allow it to outlive the borrow, which is not allowed.
    if let HirExpr::Name { name: ref src_name, ref ty } = value {
        if ty.ownership() == sifr_type_system::OwnershipKind::Move {
            // Escape analysis: cannot store a borrowed parameter into a new binding
            if ctx.borrowed_params.contains(src_name.as_str()) {
                ctx.error(format!(
                    "cannot store borrowed parameter `{}`: it is borrowed by default -- use `own {}` to take ownership, or store `{}.clone()`",
                    src_name, src_name, src_name
                ));
            } else {
                ctx.scope.mark_moved(src_name);
            }
        }
    }

    ctx.scope.define(name.clone(), declared_type.clone());

    Some(HirStmt::Let {
        name,
        ty: declared_type,
        value,
        is_mutable: true,
    })
}

/// Handle chained assignment: x = y = z = 0
/// Expands into: z = 0; y = z; x = y (right-to-left, last target gets the value first)
fn lower_chained_assign(assign: &StmtAssign, ctx: &mut LowerCtx) -> Vec<HirStmt> {
    let mut result = Vec::new();

    // Lower the value expression once
    let value = match lower_expr(&assign.value, ctx) {
        Some(v) => v,
        None => return result,
    };
    let val_ty = value.ty().clone();

    // Process targets in reverse order (rightmost gets the value first)
    let targets: Vec<_> = assign.targets.iter().collect();
    for (i, target) in targets.iter().rev().enumerate() {
        if let Expr::Name(n) = target {
            let name = n.id.to_string();
            if i == 0 {
                // First (rightmost) target gets the actual value
                let existing = ctx.scope.lookup(&name);
                if existing.is_some() {
                    // Reassignment
                    result.push(HirStmt::Assign {
                        name: name.clone(),
                        value: value.clone(),
                    });
                } else {
                    // New variable
                    ctx.scope.define(name.clone(), val_ty.clone());
                    result.push(HirStmt::Let {
                        name: name.clone(),
                        ty: val_ty.clone(),
                        value: value.clone(),
                        is_mutable: true,
                    });
                }
            } else {
                // Subsequent targets get a reference to the previous target
                let prev_target = match targets.get(targets.len() - i) {
                    Some(Expr::Name(prev_n)) => prev_n.id.to_string(),
                    _ => continue,
                };
                let name_expr = HirExpr::Name {
                    name: prev_target,
                    ty: val_ty.clone(),
                };
                let existing = ctx.scope.lookup(&name);
                if existing.is_some() {
                    result.push(HirStmt::Assign {
                        name: name.clone(),
                        value: name_expr,
                    });
                } else {
                    ctx.scope.define(name.clone(), val_ty.clone());
                    result.push(HirStmt::Let {
                        name: name.clone(),
                        ty: val_ty.clone(),
                        value: name_expr,
                        is_mutable: true,
                    });
                }
            }
        } else {
            ctx.error("chained assignment targets must be simple names".to_string());
        }
    }

    result
}

fn lower_assign(assign: &StmtAssign, ctx: &mut LowerCtx) -> Option<HirStmt> {
    if assign.targets.len() != 1 {
        ctx.error("multiple assignment targets not supported yet".to_string());
        return None;
    }

    // Handle tuple unpacking: a, b = expr or a, *b = expr
    if let Expr::Tuple(tuple) = &assign.targets[0] {
        // Check if any element is a Starred expression (star unpacking)
        let has_star = tuple.elts.iter().any(|e| matches!(e, Expr::Starred(_)));
        if has_star {
            return lower_star_unpack_assign(tuple, &assign.value, ctx);
        }
        return lower_tuple_unpack_assign(tuple, &assign.value, ctx);
    }

    // Handle attribute assignment: self.field = value or obj.field = value
    if let Expr::Attribute(attr) = &assign.targets[0] {
        let obj_name = match attr.value.as_ref() {
            Expr::Name(n) => n.id.to_string(),
            _ => {
                ctx.error("attribute assignment target must be a simple name".to_string());
                return None;
            }
        };
        let field_name = attr.attr.to_string();
        let value = lower_expr(&assign.value, ctx)?;
        return Some(HirStmt::FieldAssign {
            object: obj_name,
            field: field_name,
            value,
        });
    }

    // Handle subscript assignment: list[i] = val or dict[key] = val
    if let Expr::Subscript(sub) = &assign.targets[0] {
        // Handle nested subscript: matrix[i][j] = val
        if let Expr::Subscript(inner_sub) = sub.value.as_ref() {
            let obj_name = match inner_sub.value.as_ref() {
                Expr::Name(n) => n.id.to_string(),
                _ => {
                    ctx.error("nested subscript assignment target must be a simple name".to_string());
                    return None;
                }
            };
            let obj_ty = ctx.scope.lookup(&obj_name)
                .map(|info| info.effective_type().clone())
                .unwrap_or(Type::Unknown);
            let outer_index = lower_expr(&inner_sub.slice, ctx)?;
            let inner_index = lower_expr(&sub.slice, ctx)?;
            let value = lower_expr(&assign.value, ctx)?;
            return Some(HirStmt::NestedSubscriptAssign {
                object: obj_name,
                outer_index,
                inner_index,
                value,
                object_ty: obj_ty,
            });
        }
        // Handle attribute subscript assignment: self.field[key] = val
        if let Expr::Attribute(attr) = sub.value.as_ref() {
            let obj_name = match attr.value.as_ref() {
                Expr::Name(n) => n.id.to_string(),
                _ => {
                    ctx.error("subscript assignment target must be a simple name".to_string());
                    return None;
                }
            };
            let field_name = attr.attr.to_string();
            // Look up field type from the object's class definition
            let field_ty = ctx.scope.lookup(&obj_name)
                .and_then(|info| {
                    let obj_ty = info.effective_type();
                    // The object may be typed as Type::Class directly (e.g. `self`)
                    // or as Type::Unknown for unresolved types.
                    if let Type::Class { fields, .. } = obj_ty {
                        fields.iter().find(|(n, _)| n == &field_name).map(|(_, t)| t.clone())
                    } else if let Type::Class { name: class_name, .. } = obj_ty {
                        // Class by name reference
                        ctx.class_types.get(class_name).and_then(|class_ty| {
                            if let Type::Class { fields, .. } = class_ty {
                                fields.iter().find(|(n, _)| n == &field_name).map(|(_, t)| t.clone())
                            } else {
                                None
                            }
                        })
                    } else {
                        None
                    }
                })
                .unwrap_or(Type::Unknown);
            let index = lower_expr(&sub.slice, ctx)?;
            let value = lower_expr(&assign.value, ctx)?;
            return Some(HirStmt::AttributeSubscriptAssign {
                object: obj_name,
                field: field_name,
                index,
                value,
                field_ty,
            });
        }
        let obj_name = match sub.value.as_ref() {
            Expr::Name(n) => n.id.to_string(),
            _ => {
                ctx.error("subscript assignment target must be a simple name".to_string());
                return None;
            }
        };
        let obj_ty = ctx.scope.lookup(&obj_name)
            .map(|info| info.effective_type().clone())
            .unwrap_or(Type::Unknown);
        let index = lower_expr(&sub.slice, ctx)?;
        let value = lower_expr(&assign.value, ctx)?;
        return Some(HirStmt::SubscriptAssign {
            object: obj_name,
            index,
            value,
            object_ty: obj_ty,
        });
    }

    let name = match &assign.targets[0] {
        Expr::Name(n) => n.id.to_string(),
        _ => {
            ctx.error("assignment target must be a simple name".to_string());
            return None;
        }
    };

    // Handle `_ = expr` as explicit discard (suppresses #[must_use] warnings)
    if name == "_" {
        let value = lower_expr(&assign.value, ctx)?;
        let value_ty = value.ty().clone();
        return Some(HirStmt::Let {
            name: "_".to_string(),
            ty: value_ty,
            value,
            is_mutable: false,
        });
    }

    let value = lower_expr(&assign.value, ctx)?;
    let value_ty = value.ty().clone();

    // Track move: if RHS is a variable name with Move ownership, mark it as moved
    if let HirExpr::Name { name: ref src_name, ref ty } = value {
        if ty.ownership() == sifr_type_system::OwnershipKind::Move {
            ctx.scope.mark_moved(src_name);
        }
    }

    // Check if variable already exists
    if let Some(info) = ctx.scope.lookup(&name) {
        // Reassignment: check type compatibility
        if !value_ty.is_assignable_to(&info.ty) {
            ctx.error(format!(
                "type mismatch: cannot assign '{}' to variable '{}' of type '{}'",
                value_ty.display_name(),
                name,
                info.ty.display_name()
            ));
        }
        // Reset moved state on reassignment
        ctx.scope.reset_moved(&name);
        Some(HirStmt::Assign { name, value })
    } else {
        // New variable (type inferred)
        ctx.scope.define(name.clone(), value_ty.clone());
        Some(HirStmt::Let {
            name,
            ty: value_ty,
            value,
            is_mutable: true,
        })
    }
}

fn lower_aug_assign(aug: &StmtAugAssign, ctx: &mut LowerCtx) -> Option<HirStmt> {
    // Handle augmented assignment on attributes: self.field += val
    if let Expr::Attribute(attr) = aug.target.as_ref() {
        let obj_name = match attr.value.as_ref() {
            Expr::Name(n) => n.id.to_string(),
            _ => {
                ctx.error("augmented attribute assignment target must be a simple name".to_string());
                return None;
            }
        };
        let field_name = attr.attr.to_string();
        let value = lower_expr(&aug.value, ctx)?;
        let op_str = match aug.op {
            Operator::Add => "+=",
            Operator::Sub => "-=",
            Operator::Mult => "*=",
            Operator::Div => "/=",
            Operator::Mod => "%=",
            Operator::Pow => "**=",
            Operator::BitAnd => "&=",
            Operator::BitOr => "|=",
            Operator::BitXor => "^=",
            Operator::LShift => "<<=",
            Operator::RShift => ">>=",
            Operator::FloorDiv => "//=",
            Operator::MatMult => {
                ctx.error("matrix multiplication operator (@) is not supported".to_string());
                return None;
            }
        };
        return Some(HirStmt::AttributeAugAssign {
            object: obj_name,
            field: field_name,
            op: op_str.to_string(),
            value,
        });
    }

    // Handle augmented assignment on subscript: list[i] += val
    if let Expr::Subscript(sub) = aug.target.as_ref() {
        let obj_name = match sub.value.as_ref() {
            Expr::Name(n) => n.id.to_string(),
            _ => {
                ctx.error("augmented subscript assignment target must be a simple name".to_string());
                return None;
            }
        };
        let obj_ty = ctx.scope.lookup(&obj_name)
            .map(|info| info.effective_type().clone())
            .unwrap_or(Type::Unknown);
        let index = lower_expr(&sub.slice, ctx)?;
        let value = lower_expr(&aug.value, ctx)?;
        let op_str = match aug.op {
            Operator::Add => "+=",
            Operator::Sub => "-=",
            Operator::Mult => "*=",
            Operator::Div => "/=",
            Operator::Mod => "%=",
            Operator::Pow => "**=",
            Operator::BitAnd => "&=",
            Operator::BitOr => "|=",
            Operator::BitXor => "^=",
            Operator::LShift => "<<=",
            Operator::RShift => ">>=",
            Operator::FloorDiv => "//=",
            Operator::MatMult => {
                ctx.error("matrix multiplication operator (@) is not supported".to_string());
                return None;
            }
        };
        return Some(HirStmt::SubscriptAugAssign {
            object: obj_name,
            index,
            op: op_str.to_string(),
            value,
            object_ty: obj_ty,
        });
    }

    let name = match aug.target.as_ref() {
        Expr::Name(n) => n.id.to_string(),
        _ => {
            ctx.error("augmented assignment target must be a simple name".to_string());
            return None;
        }
    };

    let value = lower_expr(&aug.value, ctx)?;

    let op_str = match aug.op {
        Operator::Add => "+=",
        Operator::Sub => "-=",
        Operator::Mult => "*=",
        Operator::Div => "/=",
        Operator::FloorDiv => "//=",
        Operator::Mod => "%=",
        Operator::Pow => "**=",
        Operator::BitAnd => "&=",
        Operator::BitOr => "|=",
        Operator::BitXor => "^=",
        Operator::LShift => "<<=",
        Operator::RShift => ">>=",
        Operator::MatMult => {
            ctx.error("matrix multiplication operator (@) is not supported".to_string());
            return None;
        }
    };

    // Check that the variable exists
    let var_info = ctx.scope.lookup(&name);
    if var_info.is_none() {
        ctx.error(format!("undefined variable: '{}'", name));
        return None;
    }
    let var_ty = var_info.unwrap().ty.clone();

    // Type check the operation
    let base_op = &op_str[..op_str.len() - 1]; // Remove '='
    // For += on strings, allow str += str
    // For += on lists, allow list += list
    if base_op == "+" {
        match (&var_ty, value.ty()) {
            (Type::Str, Type::Str) => {}
            (Type::List(_), Type::List(_)) => {}
            _ => {
                if let Err(e) = type_check_binary_op(&var_ty, base_op, value.ty()) {
                    ctx.error(e.message);
                    return None;
                }
            }
        }
    } else if let Err(e) = type_check_binary_op(&var_ty, base_op, value.ty()) {
        ctx.error(e.message);
        return None;
    }

    Some(HirStmt::AugAssign {
        name,
        op: op_str.to_string(),
        value,
    })
}

fn lower_return(ret: &StmtReturn, func_type: &FunctionType, ctx: &mut LowerCtx) -> Option<HirStmt> {
    let value = if let Some(val) = &ret.value {
        let expr = lower_expr(val, ctx)?;
        let expr_ty = expr.ty().clone();

        // Escape analysis: returning a borrowed parameter is a compile error.
        // The programmer must use `own` to transfer ownership, or call `.clone()` explicitly.
        if let HirExpr::Name { name, ty } = &expr {
            if ctx.borrowed_params.contains(name.as_str())
                && ty.ownership() == OwnershipKind::Move
            {
                ctx.error(format!(
                    "cannot return borrowed parameter `{}`: it is borrowed by default -- use `own {}` to take ownership, or return `{}.clone()`",
                    name, name, name
                ));
            }
        }

        // If the function returns Result[T, E] and the value is T (not Result), wrap in Ok()
        if let Type::Result(ref ok_ty, _) = *func_type.return_type {
            if expr_ty.is_assignable_to(ok_ty) && !matches!(expr_ty, Type::Result(_, _)) {
                // Wrap in Ok()
                return Some(HirStmt::Return {
                    value: Some(HirExpr::OkWrap {
                        ty: func_type.return_type.as_ref().clone(),
                        value: Box::new(expr),
                    }),
                });
            }
        }

        if !expr_ty.is_assignable_to(&func_type.return_type) {
            ctx.error(format!(
                "return type mismatch: expected '{}', got '{}'",
                func_type.return_type.display_name(),
                expr_ty.display_name()
            ));
        }
        Some(expr)
    } else {
        if *func_type.return_type != Type::None {
            // If function returns Result[(), E], wrap in Ok(())
            if let Type::Result(ref ok_ty, _) = *func_type.return_type {
                if **ok_ty == Type::None {
                    return Some(HirStmt::Return {
                        value: Some(HirExpr::OkWrap {
                            ty: func_type.return_type.as_ref().clone(),
                            value: Box::new(HirExpr::NoneLiteral),
                        }),
                    });
                }
            }
            ctx.error(format!(
                "function expects return type '{}', but returns nothing",
                func_type.return_type.display_name()
            ));
        }
        None
    };

    Some(HirStmt::Return { value })
}

fn lower_if(if_stmt: &StmtIf, func_type: &FunctionType, ctx: &mut LowerCtx) -> Option<HirStmt> {
    // Try to detect a narrowing condition from the test expression
    let narrowing_cond = detect_narrowing_condition(&if_stmt.test, ctx);

    let condition = lower_expr(&if_stmt.test, ctx)?;

    // Save narrowing state before branches
    let saved_state = ctx.scope.save_narrowing_state();
    // Save moved state before branches
    let saved_moved = ctx.scope.save_moved_state();

    // Apply narrowing for then-branch (condition is true)
    if let Some(ref cond) = narrowing_cond {
        apply_narrowing(ctx, cond, true);
    }

    ctx.scope.push();
    let then_body = lower_stmts(&if_stmt.body, func_type, ctx);
    ctx.scope.pop();

    // Record which vars were moved in then-branch
    let then_moved = ctx.scope.save_moved_state();

    // Restore state before processing elif/else
    ctx.scope.restore_narrowing_state(&saved_state);
    ctx.scope.restore_moved_state(&saved_moved);

    // Collect all narrowing conditions (if + elifs) for cumulative negation
    let mut all_conditions: Vec<NarrowingCondition> = Vec::new();
    if let Some(ref cond) = narrowing_cond {
        all_conditions.push(cond.clone());
    }

    // Track moved state from each branch for merging
    let mut branch_moved_states: Vec<_> = vec![then_moved];

    let mut elif_clauses = Vec::new();
    for clause in &if_stmt.elif_else_clauses {
        if let Some(test) = &clause.test {
            // For elif, apply the negation of ALL previous conditions
            // This ensures cumulative narrowing: if A was Dog, elif B was Cat,
            // then in elif C the type is narrowed by removing both Dog and Cat
            ctx.scope.restore_narrowing_state(&saved_state);
            ctx.scope.restore_moved_state(&saved_moved);
            for prev_cond in &all_conditions {
                apply_narrowing(ctx, prev_cond, false);
            }

            let elif_narrowing = detect_narrowing_condition(test, ctx);
            let cond = lower_expr(test, ctx)?;

            let elif_saved = ctx.scope.save_narrowing_state();
            if let Some(ref elif_cond) = elif_narrowing {
                apply_narrowing(ctx, elif_cond, true);
            }

            ctx.scope.push();
            let body = lower_stmts(&clause.body, func_type, ctx);
            ctx.scope.pop();
            elif_clauses.push((cond, body));

            // Record moved state from this elif branch
            branch_moved_states.push(ctx.scope.save_moved_state());

            ctx.scope.restore_narrowing_state(&elif_saved);
            ctx.scope.restore_moved_state(&saved_moved);

            // Track this elif's condition for subsequent branches
            if let Some(elif_cond) = elif_narrowing {
                all_conditions.push(elif_cond);
            }
        }
    }

    // For else-branch, apply negation of ALL conditions (if + all elifs)
    let else_body = if_stmt.elif_else_clauses.iter().find(|c| c.test.is_none()).map(|clause| {
        ctx.scope.restore_narrowing_state(&saved_state);
        ctx.scope.restore_moved_state(&saved_moved);
        for prev_cond in &all_conditions {
            apply_narrowing(ctx, prev_cond, false);
        }
        ctx.scope.push();
        let body = lower_stmts(&clause.body, func_type, ctx);
        ctx.scope.pop();
        // Record moved state from else branch
        branch_moved_states.push(ctx.scope.save_moved_state());
        body
    });

    // Restore original narrowing state after all branches
    ctx.scope.restore_narrowing_state(&saved_state);
    ctx.scope.restore_moved_state(&saved_moved);

    // Merge moved state: if a variable was moved in ANY branch, mark it as moved
    // after the if/else (conservative, matches Rust behavior for partial moves)
    for branch_state in &branch_moved_states {
        for (name, was_moved) in branch_state {
            if *was_moved {
                ctx.scope.mark_moved(name);
            }
        }
    }

    // Early-return narrowing: if the then-body always exits (return/break/continue/raise),
    // apply the inverse narrowing after the if block.
    // e.g., `if x is None: return` -> after the if, x is not None
    if let Some(ref cond) = narrowing_cond {
        if then_body_always_exits(&then_body) && elif_clauses.is_empty() && else_body.is_none() {
            apply_narrowing(ctx, cond, false);
        }
    }

    Some(HirStmt::If {
        condition,
        then_body,
        elif_clauses,
        else_body,
    })
}

/// Check if a block of statements always exits (return, break, continue, raise).
/// Used for early-return narrowing: `if x is None: return` narrows x after the if.
fn then_body_always_exits(stmts: &[HirStmt]) -> bool {
    if let Some(last) = stmts.last() {
        matches!(last, HirStmt::Return { .. })
            || matches!(last, HirStmt::Expr { expr: HirExpr::Call { func, .. } } if func == "raise")
    } else {
        false
    }
}

/// Collect all return types from a list of HIR statements (recursively).
fn collect_return_types(stmts: &[HirStmt]) -> Vec<Type> {
    let mut types = Vec::new();
    for stmt in stmts {
        match stmt {
            HirStmt::Return { value: Some(expr) } => {
                types.push(expr.ty().clone());
            }
            HirStmt::Return { value: None } => {
                types.push(Type::None);
            }
            HirStmt::If { then_body, elif_clauses, else_body, .. } => {
                types.extend(collect_return_types(then_body));
                for (_, body) in elif_clauses {
                    types.extend(collect_return_types(body));
                }
                if let Some(body) = else_body {
                    types.extend(collect_return_types(body));
                }
            }
            HirStmt::While { body, else_body, .. } => {
                types.extend(collect_return_types(body));
                if let Some(eb) = else_body {
                    types.extend(collect_return_types(eb));
                }
            }
            HirStmt::For { body, .. } => {
                types.extend(collect_return_types(body));
            }
            HirStmt::TryExcept { body, handlers, .. } => {
                types.extend(collect_return_types(body));
                for handler in handlers {
                    types.extend(collect_return_types(&handler.body));
                }
            }
            _ => {}
        }
    }
    types
}

/// Detect a narrowing condition from an if-test expression.
fn detect_narrowing_condition(expr: &Expr, ctx: &LowerCtx) -> Option<NarrowingCondition> {
    match expr {
        // isinstance(x, Type) -> IsInstance narrowing
        Expr::Call(call) => {
            if let Expr::Name(func_name) = call.func.as_ref() {
                if func_name.id.as_str() == "isinstance" && call.arguments.args.len() == 2 {
                    if let Expr::Name(var) = &call.arguments.args[0] {
                        let var_name = var.id.to_string();
                        // Check that the variable exists and has a union/Unknown type
                        if ctx.scope.lookup(&var_name).is_some() {
                            if let Expr::Name(type_name) = &call.arguments.args[1] {
                                // Try built-in types first, then class types
                                let target_ty = resolve_type_annotation(&type_name.id)
                                    .or_else(|| ctx.class_types.get(type_name.id.as_str()).cloned());
                                if let Some(target_ty) = target_ty {
                                    return Some(NarrowingCondition::IsInstance(var_name, target_ty));
                                }
                            }
                        }
                    }
                }
            }
            None
        }
        // x is None / x is not None
        Expr::Compare(cmp) => {
            if cmp.ops.len() == 1 && cmp.comparators.len() == 1 {
                match &cmp.ops[0] {
                    CmpOp::Is => {
                        if let (Expr::Name(var), Expr::NoneLiteral(_)) = (cmp.left.as_ref(), &cmp.comparators[0]) {
                            let var_name = var.id.to_string();
                            if ctx.scope.lookup(&var_name).is_some() {
                                return Some(NarrowingCondition::IsNone(var_name));
                            }
                        }
                    }
                    CmpOp::IsNot => {
                        if let (Expr::Name(var), Expr::NoneLiteral(_)) = (cmp.left.as_ref(), &cmp.comparators[0]) {
                            let var_name = var.id.to_string();
                            if ctx.scope.lookup(&var_name).is_some() {
                                return Some(NarrowingCondition::IsNotNone(var_name));
                            }
                        }
                    }
                    // x == "value" -> Equality narrowing
                    CmpOp::Eq => {
                        if let Expr::Name(var) = cmp.left.as_ref() {
                            let var_name = var.id.to_string();
                            if ctx.scope.lookup(&var_name).is_some() {
                                if let Some(lit_val) = expr_to_literal_value(&cmp.comparators[0]) {
                                    return Some(NarrowingCondition::Equality(var_name, lit_val));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            None
        }
        // Simple variable name -> Truthiness narrowing
        Expr::Name(name) => {
            let var_name = name.id.to_string();
            if ctx.scope.lookup(&var_name).is_some() {
                Some(NarrowingCondition::Truthiness(var_name))
            } else {
                None
            }
        }
        // not expr -> negate the inner condition
        Expr::UnaryOp(unary) if matches!(unary.op, UnaryOp::Not) => {
            let inner = detect_narrowing_condition(&unary.operand, ctx)?;
            Some(NarrowingCondition::Not(Box::new(inner)))
        }
        // a and b -> And narrowing (both conditions must be true)
        Expr::BoolOp(boolop) if matches!(boolop.op, BoolOp::And) => {
            let conditions: Vec<NarrowingCondition> = boolop.values.iter()
                .filter_map(|v| detect_narrowing_condition(v, ctx))
                .collect();
            if conditions.is_empty() {
                None
            } else if conditions.len() == 1 {
                Some(conditions.into_iter().next().unwrap())
            } else {
                Some(NarrowingCondition::And(conditions))
            }
        }
        _ => None,
    }
}

/// Convert an AST expression to a LiteralValue (for equality narrowing).
fn expr_to_literal_value(expr: &Expr) -> Option<sifr_type_system::LiteralValue> {
    match expr {
        Expr::StringLiteral(s) => Some(sifr_type_system::LiteralValue::Str(s.value.to_str().to_string())),
        Expr::NumberLiteral(num) => {
            match &num.value {
                Number::Int(i) => i.as_i64().map(sifr_type_system::LiteralValue::Int),
                _ => None,
            }
        }
        Expr::BooleanLiteral(b) => Some(sifr_type_system::LiteralValue::Bool(b.value)),
        _ => None,
    }
}

/// Apply narrowing to the scope based on a condition.
fn apply_narrowing(ctx: &mut LowerCtx, condition: &NarrowingCondition, is_true: bool) {
    match condition {
        NarrowingCondition::And(conditions) => {
            if is_true {
                // All conditions are true: apply each narrowing
                for cond in conditions {
                    apply_narrowing(ctx, cond, true);
                }
            } else {
                // At least one is false: can't narrow precisely, skip
            }
        }
        NarrowingCondition::Or(conditions) => {
            if !is_true {
                // All conditions are false: apply each false-narrowing
                for cond in conditions {
                    apply_narrowing(ctx, cond, false);
                }
            }
        }
        _ => {
            if let Some(var_name) = condition.var_name() {
                if let Some(info) = ctx.scope.lookup(var_name) {
                    let current_ty = info.effective_type().clone();
                    let narrowed = narrow_type(&current_ty, condition, is_true);
                    ctx.scope.narrow_var(var_name, narrowed);
                }
            }
        }
    }
}

fn lower_while(while_stmt: &StmtWhile, func_type: &FunctionType, ctx: &mut LowerCtx) -> Option<HirStmt> {
    let condition = lower_expr(&while_stmt.test, ctx)?;

    // Snapshot moved state before loop to detect moves inside the body
    let moved_before_loop = ctx.scope.save_moved_state();

    ctx.scope.push();
    ctx.loop_depth += 1;
    let body = lower_stmts(&while_stmt.body, func_type, ctx);
    ctx.loop_depth -= 1;
    ctx.scope.pop();

    // Check for outer-scope variables moved inside the loop body
    let newly_moved = ctx.scope.moved_since(&moved_before_loop);
    for var_name in &newly_moved {
        ctx.error(format!(
            "value '{}' is moved inside loop body; it would be unavailable on subsequent iterations",
            var_name
        ));
    }

    let else_body = if !while_stmt.orelse.is_empty() {
        ctx.scope.push();
        let else_stmts = lower_stmts(&while_stmt.orelse, func_type, ctx);
        ctx.scope.pop();
        Some(else_stmts)
    } else {
        None
    };

    Some(HirStmt::While { condition, body, else_body })
}

fn lower_for(for_stmt: &StmtFor, func_type: &FunctionType, ctx: &mut LowerCtx) -> Option<HirStmt> {
    // Lower the iterable expression
    let iter_expr = lower_expr(&for_stmt.iter, ctx)?;
    let iter_ty = iter_expr.ty().clone();

    // Determine the element type from the iterable
    let elem_ty = iter_ty.iterable_element_type().unwrap_or_else(|| {
        ctx.error(format!(
            "cannot iterate over type '{}'",
            iter_ty.display_name()
        ));
        Type::Any
    });

    // Extract the target variable name(s)
    let target_name = match for_stmt.target.as_ref() {
        Expr::Name(n) => n.id.to_string(),
        Expr::Tuple(tup) => {
            // Tuple unpacking: for i, v in enumerate(lst)
            let names: Vec<String> = tup.elts.iter().filter_map(|e| {
                if let Expr::Name(n) = e {
                    Some(n.id.to_string())
                } else {
                    None
                }
            }).collect();
            if names.len() != tup.elts.len() {
                ctx.error("for loop tuple target must contain only simple names".to_string());
                return None;
            }
            names.join(",")
        }
        _ => {
            ctx.error("for loop target must be a simple name or tuple".to_string());
            return None;
        }
    };

    // Snapshot moved state before loop to detect moves inside the body
    let moved_before_loop = ctx.scope.save_moved_state();

    // Create a new scope for the loop body, define the loop variable(s)
    ctx.scope.push();
    if target_name.contains(',') {
        // Tuple unpacking: define each variable with its type from the tuple
        let names: Vec<&str> = target_name.split(',').collect();
        if let Type::Tuple(elem_types) = &elem_ty {
            for (i, name) in names.iter().enumerate() {
                let ty = elem_types.get(i).cloned().unwrap_or(Type::Any);
                ctx.scope.define(name.to_string(), ty);
            }
        } else {
            // Fallback: define all as Any
            for name in &names {
                ctx.scope.define(name.to_string(), Type::Any);
            }
        }
    } else {
        ctx.scope.define(target_name.clone(), elem_ty.clone());
    }
    ctx.loop_depth += 1;
    let body = lower_stmts(&for_stmt.body, func_type, ctx);
    ctx.loop_depth -= 1;
    ctx.scope.pop();

    // Check for outer-scope variables moved inside the loop body
    let newly_moved = ctx.scope.moved_since(&moved_before_loop);
    for var_name in &newly_moved {
        ctx.error(format!(
            "value '{}' is moved inside loop body; it would be unavailable on subsequent iterations",
            var_name
        ));
    }

    let else_body = if !for_stmt.orelse.is_empty() {
        ctx.scope.push();
        let else_stmts = lower_stmts(&for_stmt.orelse, func_type, ctx);
        ctx.scope.pop();
        Some(else_stmts)
    } else {
        None
    };

    Some(HirStmt::For {
        target: target_name,
        target_ty: elem_ty,
        iter: iter_expr,
        body,
        else_body,
    })
}

fn lower_expr(expr: &Expr, ctx: &mut LowerCtx) -> Option<HirExpr> {
    match expr {
        Expr::NumberLiteral(num) => lower_number_literal(num),
        Expr::StringLiteral(s) => {
            let value = s.value.to_str().to_string();
            Some(HirExpr::StringLiteral(value))
        }
        Expr::BooleanLiteral(b) => Some(HirExpr::BoolLiteral(b.value)),
        Expr::NoneLiteral(_) => Some(HirExpr::NoneLiteral),
        Expr::Name(name) => lower_name(name, ctx),
        Expr::BinOp(binop) => lower_binop(binop, ctx),
        Expr::UnaryOp(unary) => lower_unaryop(unary, ctx),
        Expr::Compare(cmp) => lower_compare(cmp, ctx),
        Expr::BoolOp(boolop) => lower_boolop(boolop, ctx),
        Expr::Call(call) => lower_call(call, ctx),
        Expr::If(if_expr) => lower_if_expr(if_expr, ctx),
        Expr::List(list) => lower_list_literal(list, ctx),
        Expr::Set(set) => lower_set_literal(set, ctx),
        Expr::Dict(dict) => lower_dict_literal(dict, ctx),
        Expr::Tuple(tuple) => lower_tuple_literal(tuple, ctx),
        Expr::Subscript(sub) => lower_subscript(sub, ctx),
        Expr::Attribute(attr) => lower_attribute(attr, ctx),
        Expr::FString(fstring) => lower_fstring(fstring, ctx),
        Expr::Named(named) => lower_named_expr(named, ctx),
        Expr::Lambda(lambda) => lower_lambda(lambda, ctx),
        Expr::ListComp(comp) => lower_list_comp(comp, ctx),
        Expr::SetComp(comp) => lower_set_comp(comp, ctx),
        Expr::DictComp(comp) => lower_dict_comp(comp, ctx),
        Expr::Generator(gen) => lower_generator_expr(gen, ctx),
        _ => {
            ctx.error("unsupported expression type".to_string());
            None
        }
    }
}

fn lower_number_literal(num: &ExprNumberLiteral) -> Option<HirExpr> {
    match &num.value {
        Number::Int(i) => {
            let val = i.as_i64()?;
            Some(HirExpr::IntLiteral(val))
        }
        Number::Float(f) => Some(HirExpr::FloatLiteral(*f)),
        Number::Complex { .. } => None, // Not supported in M1
    }
}

fn lower_name(name: &ExprName, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let var_name = name.id.to_string();

    // Check if it's a known variable
    if let Some(info) = ctx.scope.lookup(&var_name) {
        let is_moved = info.is_moved;
        // Use effective type (narrowed if available)
        let ty = info.effective_type().clone();
        if is_moved {
            ctx.error(format!(
                "use of moved value: '{}'",
                var_name
            ));
        }
        return Some(HirExpr::Name {
            name: var_name,
            ty,
        });
    }

    // Check if it's a known function
    if let Some(ft) = ctx.functions.get(&var_name) {
        let ft = ft.clone();
        return Some(HirExpr::Name {
            name: var_name,
            ty: Type::Function(ft),
        });
    }

    // Check built-in constants
    match var_name.as_str() {
        "True" => return Some(HirExpr::BoolLiteral(true)),
        "False" => return Some(HirExpr::BoolLiteral(false)),
        _ => {}
    }

    ctx.error(format!("undefined variable: '{}'", var_name));
    None
}

/// Map a binary operator to its corresponding dunder method name.
fn op_to_dunder(op: &str) -> Option<&'static str> {
    match op {
        "+" => Some("__add__"),
        "-" => Some("__sub__"),
        "*" => Some("__mul__"),
        "/" => Some("__truediv__"),
        "//" => Some("__floordiv__"),
        "%" => Some("__mod__"),
        "**" => Some("__pow__"),
        _ => None,
    }
}

fn lower_binop(binop: &ExprBinOp, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let left = lower_expr(&binop.left, ctx)?;
    let right = lower_expr(&binop.right, ctx)?;

    let op_str = match binop.op {
        Operator::Add => "+",
        Operator::Sub => "-",
        Operator::Mult => "*",
        Operator::Div => "/",
        Operator::FloorDiv => "//",
        Operator::Mod => "%",
        Operator::Pow => "**",
        Operator::BitAnd => "&",
        Operator::BitOr => "|",
        Operator::BitXor => "^",
        Operator::LShift => "<<",
        Operator::RShift => ">>",
        Operator::MatMult => {
            ctx.error("matrix multiplication operator (@) is not supported".to_string());
            return None;
        }
    };

    match type_check_binary_op(left.ty(), op_str, right.ty()) {
        Ok(result_ty) => {
            if result_ty == Type::Int {
                check_int_overflow_risk(op_str, &left, &right, ctx);
            }
            Some(HirExpr::BinOp {
                left: Box::new(left),
                op: op_str.to_string(),
                right: Box::new(right),
                ty: result_ty,
            })
        }
        Err(e) => {
            // Check for operator overloading on class types
            if let Type::Class { methods, .. } = left.ty() {
                if let Some(dunder) = op_to_dunder(op_str) {
                    if let Some((_, ft)) = methods.iter().find(|(n, _)| n == dunder) {
                        let result_ty = *ft.return_type.clone();
                        return Some(HirExpr::BinOp {
                            left: Box::new(left),
                            op: op_str.to_string(),
                            right: Box::new(right),
                            ty: result_ty,
                        });
                    }
                }
            }
            ctx.error(e.message);
            None
        }
    }
}

fn check_int_overflow_risk(op: &str, left: &HirExpr, right: &HirExpr, ctx: &mut LowerCtx) {
    let is_left_const = matches!(left, HirExpr::IntLiteral(_));
    let is_right_const = matches!(right, HirExpr::IntLiteral(_));

    match op {
        "**" => {
            if let HirExpr::IntLiteral(exp) = right {
                if *exp > 40 {
                    ctx.warn(format!(
                        "warning: int exponentiation with large exponent ({}) may overflow i64; consider using bigint",
                        exp
                    ));
                }
            } else {
                ctx.warn(
                    "warning: int exponentiation (**) with non-constant exponent may overflow i64 at runtime; consider using bigint".to_string()
                );
            }
        }
        "*" => {
            if !is_left_const && !is_right_const {
                ctx.warn(
                    "warning: int multiplication with non-constant operands may overflow i64 at runtime; consider using bigint for large values".to_string()
                );
            }
        }
        "<<" => {
            if !is_right_const {
                ctx.warn(
                    "warning: int left shift (<<) with non-constant shift amount may overflow i64 at runtime; consider using bigint".to_string()
                );
            } else if let HirExpr::IntLiteral(shift) = right {
                if *shift >= 63 {
                    ctx.warn(format!(
                        "warning: int left shift by {} exceeds i64 range; consider using bigint",
                        shift
                    ));
                }
            }
        }
        _ => {}
    }
}

fn lower_unaryop(unary: &ExprUnaryOp, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let operand = lower_expr(&unary.operand, ctx)?;

    let op_str = match unary.op {
        UnaryOp::USub => "-",
        UnaryOp::UAdd => "+",
        UnaryOp::Not => "not",
        UnaryOp::Invert => "~",
    };

    match type_check_unary_op(op_str, operand.ty()) {
        Ok(result_ty) => Some(HirExpr::UnaryOp {
            op: op_str.to_string(),
            operand: Box::new(operand),
            ty: result_ty,
        }),
        Err(e) => {
            ctx.error(e.message);
            None
        }
    }
}

fn lower_compare(cmp: &ExprCompare, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let left = lower_expr(&cmp.left, ctx)?;

    // Handle `in` and `not in` operators specially
    if cmp.ops.len() == 1 {
        match &cmp.ops[0] {
            CmpOp::In => {
                let collection = lower_expr(&cmp.comparators[0], ctx)?;
                let collection_ty = collection.ty().clone();
                if let Some(elem_ty) = collection_ty.contains_element_type() {
                    if !left.ty().is_assignable_to(&elem_ty) {
                        ctx.error(format!(
                            "'in' operator: element type '{}' is not compatible with collection element type '{}'",
                            left.ty().display_name(),
                            elem_ty.display_name()
                        ));
                    }
                } else {
                    ctx.error(format!(
                        "'in' operator not supported for type '{}'",
                        collection_ty.display_name()
                    ));
                }
                return Some(HirExpr::ContainsOp {
                    element: Box::new(left),
                    collection: Box::new(collection),
                    ty: Type::Bool,
                });
            }
            CmpOp::NotIn => {
                let collection = lower_expr(&cmp.comparators[0], ctx)?;
                let collection_ty = collection.ty().clone();
                if let Some(elem_ty) = collection_ty.contains_element_type() {
                    if !left.ty().is_assignable_to(&elem_ty) {
                        ctx.error(format!(
                            "'not in' operator: element type '{}' is not compatible with collection element type '{}'",
                            left.ty().display_name(),
                            elem_ty.display_name()
                        ));
                    }
                } else {
                    ctx.error(format!(
                        "'not in' operator not supported for type '{}'",
                        collection_ty.display_name()
                    ));
                }
                // Wrap in a UnaryOp not
                let contains = HirExpr::ContainsOp {
                    element: Box::new(left),
                    collection: Box::new(collection),
                    ty: Type::Bool,
                };
                return Some(HirExpr::UnaryOp {
                    op: "not".to_string(),
                    operand: Box::new(contains),
                    ty: Type::Bool,
                });
            }
            _ => {}
        }
    }

    let mut ops = Vec::new();
    let mut comparators = Vec::new();

    for (op, comparator) in cmp.ops.iter().zip(cmp.comparators.iter()) {
        let op_str = match op {
            CmpOp::Eq => "==",
            CmpOp::NotEq => "!=",
            CmpOp::Lt => "<",
            CmpOp::Gt => ">",
            CmpOp::LtE => "<=",
            CmpOp::GtE => ">=",
            CmpOp::Is => "is",
            CmpOp::IsNot => "is not",
            _ => {
                ctx.error("unsupported comparison operator".to_string());
                return None;
            }
        };

        let right = lower_expr(comparator, ctx)?;

        // `is` and `is not` are identity checks (used for None comparison)
        // They don't need type_check_comparison
        if op_str != "is" && op_str != "is not" {
            if let Err(e) = type_check_comparison(left.ty(), op_str, right.ty()) {
                // Check for operator overloading on class types
                let has_overload = match left.ty() {
                    Type::Class { methods, .. } => {
                        let dunder = match op_str {
                            "==" | "!=" => "__eq__",
                            "<" | ">" | "<=" | ">=" => "__lt__",
                            _ => "",
                        };
                        !dunder.is_empty() && methods.iter().any(|(n, _)| n == dunder)
                    }
                    _ => false,
                };
                if !has_overload {
                    ctx.error(e.message);
                    return None;
                }
            }
        }

        ops.push(op_str.to_string());
        comparators.push(right);
    }

    Some(HirExpr::Compare {
        left: Box::new(left),
        ops,
        comparators,
        ty: Type::Bool,
    })
}

fn lower_boolop(boolop: &ExprBoolOp, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let op_str = match boolop.op {
        BoolOp::And => "and",
        BoolOp::Or => "or",
    };

    let mut values = Vec::new();
    for val in &boolop.values {
        let expr = lower_expr(val, ctx)?;
        values.push(expr);
    }

    // Check all values are Bool
    for val in &values {
        if let Err(e) = type_check_bool_op(val.ty(), op_str, &Type::Bool) {
            ctx.error(e.message);
            return None;
        }
    }

    Some(HirExpr::BoolOp {
        op: op_str.to_string(),
        values,
        ty: Type::Bool,
    })
}

fn lower_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    // Handle method calls: obj.method(args)
    if let Expr::Attribute(attr) = call.func.as_ref() {
        return lower_method_call(attr, call, ctx);
    }

    let func_name = match call.func.as_ref() {
        Expr::Name(n) => n.id.to_string(),
        _ => {
            ctx.error("only simple function calls are supported".to_string());
            return None;
        }
    };

    // Handle `cls(...)` in @classmethod as constructor call for the current class
    if func_name == "cls" {
        if let Some(ref class_name) = ctx.current_class {
            let class_name = class_name.clone();
            if let Some(class_ty) = ctx.class_types.get(&class_name).cloned() {
                // Lower arguments
                let mut args = Vec::new();
                for arg in &call.arguments.args {
                    let expr = lower_expr(arg, ctx)?;
                    args.push(expr);
                }
                return Some(HirExpr::ConstructorCall {
                    class_name,
                    args,
                    ty: class_ty,
                });
            }
        }
    }

    // Special handling for range() built-in
    if func_name == "range" {
        return lower_range_call(call, ctx);
    }

    // Special handling for len() built-in
    if func_name == "len" {
        return lower_len_call(call, ctx);
    }

    // Special handling for isinstance() built-in
    if func_name == "isinstance" {
        return lower_isinstance_call(call, ctx);
    }

    // Special handling for reveal_type() built-in
    if func_name == "reveal_type" {
        return lower_reveal_type_call(call, ctx);
    }

    // Special handling for str() conversion
    if func_name == "str" {
        if call.arguments.args.len() == 1 {
            let arg = lower_expr(&call.arguments.args[0], ctx)?;
            return Some(HirExpr::Call {
                func: "str".to_string(),
                args: vec![arg],
                ty: Type::Str,
            });
        }
    }

    // pow(base, exp) -> base ** exp
    if func_name == "pow" {
        if call.arguments.args.len() != 2 {
            ctx.error("pow() takes exactly 2 arguments".to_string());
            return None;
        }
        let base = lower_expr(&call.arguments.args[0], ctx)?;
        let exp = lower_expr(&call.arguments.args[1], ctx)?;
        let result_ty = if base.ty() == &Type::Int && exp.ty() == &Type::Int {
            Type::Int
        } else {
            Type::Float
        };
        return Some(HirExpr::Call {
            func: "pow".to_string(),
            args: vec![base, exp],
            ty: result_ty,
        });
    }

    // Special handling for abs() built-in
    if func_name == "abs" {
        if call.arguments.args.len() != 1 {
            ctx.error(format!("abs() takes exactly 1 argument, got {}", call.arguments.args.len()));
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        let ty = arg.ty().clone();
        if !ty.is_numeric() {
            ctx.error(format!("abs() argument must be numeric, got '{}'", ty.display_name()));
            return None;
        }
        return Some(HirExpr::Call {
            func: "abs".to_string(),
            args: vec![arg],
            ty,
        });
    }

    // Special handling for hash() built-in
    if func_name == "hash" {
        if call.arguments.args.len() != 1 {
            ctx.error(format!("hash() takes exactly 1 argument, got {}", call.arguments.args.len()));
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        let ty = arg.ty().clone();
        // Check if the type is hashable
        if !is_hashable_type(&ty) {
            ctx.error(format!("hash() argument must be hashable, got '{}'", ty.display_name()));
            return None;
        }
        return Some(HirExpr::Call {
            func: "hash".to_string(),
            args: vec![arg],
            ty: Type::Int,
        });
    }

    // Special handling for round() built-in
    if func_name == "round" {
        if call.arguments.args.is_empty() || call.arguments.args.len() > 2 {
            ctx.error(format!("round() takes 1 or 2 arguments, got {}", call.arguments.args.len()));
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        if !arg.ty().is_numeric() {
            ctx.error(format!("round() argument must be numeric, got '{}'", arg.ty().display_name()));
            return None;
        }
        if call.arguments.args.len() == 2 {
            let ndigits = lower_expr(&call.arguments.args[1], ctx)?;
            return Some(HirExpr::Call {
                func: "round".to_string(),
                args: vec![arg, ndigits],
                ty: Type::Float,
            });
        }
        return Some(HirExpr::Call {
            func: "round".to_string(),
            args: vec![arg],
            ty: Type::Int,
        });
    }

    // Special handling for repr() built-in
    if func_name == "repr" {
        if call.arguments.args.len() != 1 {
            ctx.error(format!("repr() takes exactly 1 argument, got {}", call.arguments.args.len()));
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        return Some(HirExpr::Call {
            func: "repr".to_string(),
            args: vec![arg],
            ty: Type::Str,
        });
    }

    // Special handling for int() conversion
    if func_name == "int" {
        if call.arguments.args.len() != 1 {
            ctx.error(format!("int() takes exactly 1 argument, got {}", call.arguments.args.len()));
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        let arg_ty = arg.ty().clone();
        // int(str) -> Result[int, ParseError] (fallible)
        // int(float) -> int (infallible truncation)
        // int(int) -> int (identity)
        // int(bool) -> int (True=1, False=0)
        // int(bigint) -> Result[int, OverflowError] (may overflow i64)
        let result_ty = if arg_ty == Type::Str {
            let parse_error_ty = ctx.class_types.get("ParseError").cloned().unwrap_or(Type::Class {
                name: "ParseError".to_string(),
                fields: vec![("message".to_string(), Type::Str)],
                methods: vec![],
                parent_class: None,
            });
            Type::Result(Box::new(Type::Int), Box::new(parse_error_ty))
        } else if arg_ty == Type::BigInt {
            let overflow_error_ty = ctx.class_types.get("OverflowError").cloned().unwrap_or(Type::Class {
                name: "OverflowError".to_string(),
                fields: vec![("message".to_string(), Type::Str)],
                methods: vec![],
                parent_class: None,
            });
            Type::Result(Box::new(Type::Int), Box::new(overflow_error_ty))
        } else {
            Type::Int
        };
        return Some(HirExpr::Call {
            func: "int".to_string(),
            args: vec![arg],
            ty: result_ty,
        });
    }

    // bigint(n) — convert int to bigint (always succeeds)
    if func_name == "bigint" {
        if call.arguments.args.len() != 1 {
            ctx.error(format!("bigint() takes exactly 1 argument, got {}", call.arguments.args.len()));
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        let arg_ty = arg.ty().clone();
        if arg_ty != Type::Int && arg_ty != Type::BigInt {
            ctx.error(format!("bigint() requires an int argument, got '{}'", arg_ty.display_name()));
            return None;
        }
        return Some(HirExpr::Call {
            func: "bigint".to_string(),
            args: vec![arg],
            ty: Type::BigInt,
        });
    }

    // Special handling for float() conversion
    if func_name == "float" {
        if call.arguments.args.len() != 1 {
            ctx.error(format!("float() takes exactly 1 argument, got {}", call.arguments.args.len()));
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        let arg_ty = arg.ty().clone();
        // float(str) -> Result[float, ParseError] (fallible)
        // float(int) -> float (infallible widening)
        // float(float) -> float (identity)
        let result_ty = if arg_ty == Type::Str {
            let parse_error_ty = ctx.class_types.get("ParseError").cloned().unwrap_or(Type::Class {
                name: "ParseError".to_string(),
                fields: vec![("message".to_string(), Type::Str)],
                methods: vec![],
                parent_class: None,
            });
            Type::Result(Box::new(Type::Float), Box::new(parse_error_ty))
        } else {
            Type::Float
        };
        return Some(HirExpr::Call {
            func: "float".to_string(),
            args: vec![arg],
            ty: result_ty,
        });
    }

    // Special handling for bool() conversion
    if func_name == "bool" {
        if call.arguments.args.len() != 1 {
            ctx.error(format!("bool() takes exactly 1 argument, got {}", call.arguments.args.len()));
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        return Some(HirExpr::Call {
            func: "bool".to_string(),
            args: vec![arg],
            ty: Type::Bool,
        });
    }

    // --- Built-in generic functions ---

    // min(iterable) or min(a, b) -> element type
    if func_name == "min" {
        if call.arguments.args.len() == 2 {
            // min(a, b) -> std::cmp::min(a, b)
            let a = lower_expr(&call.arguments.args[0], ctx)?;
            let b = lower_expr(&call.arguments.args[1], ctx)?;
            let result_ty = a.ty().clone();
            return Some(HirExpr::Call {
                func: "min".to_string(),
                args: vec![a, b],
                ty: result_ty,
            });
        } else if call.arguments.args.len() == 1 {
            let arg = lower_expr(&call.arguments.args[0], ctx)?;
            let elem_ty = match arg.ty() {
                Type::List(elem) => *elem.clone(),
                _ => {
                    ctx.error(format!("min() argument must be a list, got '{}'", arg.ty().display_name()));
                    return None;
                }
            };
            // Returns Option[T] = T | None (safe: None on empty list)
            return Some(HirExpr::Call {
                func: "min".to_string(),
                args: vec![arg],
                ty: Type::Union(vec![elem_ty, Type::None]),
            });
        } else {
            ctx.error("min() takes 1 or 2 arguments".to_string());
            return None;
        }
    }

    // max(iterable) or max(a, b) -> element type
    if func_name == "max" {
        if call.arguments.args.len() == 2 {
            // max(a, b) -> std::cmp::max(a, b)
            let a = lower_expr(&call.arguments.args[0], ctx)?;
            let b = lower_expr(&call.arguments.args[1], ctx)?;
            let result_ty = a.ty().clone();
            return Some(HirExpr::Call {
                func: "max".to_string(),
                args: vec![a, b],
                ty: result_ty,
            });
        } else if call.arguments.args.len() == 1 {
            let arg = lower_expr(&call.arguments.args[0], ctx)?;
            let elem_ty = match arg.ty() {
                Type::List(elem) => *elem.clone(),
                _ => {
                    ctx.error(format!("max() argument must be a list, got '{}'", arg.ty().display_name()));
                    return None;
                }
            };
            // Returns Option[T] = T | None (safe: None on empty list)
            return Some(HirExpr::Call {
                func: "max".to_string(),
                args: vec![arg],
                ty: Type::Union(vec![elem_ty, Type::None]),
            });
        } else {
            ctx.error("max() takes 1 or 2 arguments".to_string());
            return None;
        }
    }

    // sum(iterable) -> element type (int or float)
    if func_name == "sum" {
        if call.arguments.args.len() != 1 {
            ctx.error("sum() takes exactly 1 argument".to_string());
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        let elem_ty = match arg.ty() {
            Type::List(elem) => *elem.clone(),
            _ => {
                ctx.error(format!("sum() argument must be a list, got '{}'", arg.ty().display_name()));
                return None;
            }
        };
        return Some(HirExpr::Call {
            func: "sum".to_string(),
            args: vec![arg],
            ty: elem_ty,
        });
    }

    // sorted(iterable) -> list of element type
    if func_name == "sorted" {
        if call.arguments.args.len() != 1 {
            ctx.error("sorted() takes exactly 1 argument".to_string());
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        let list_ty = match arg.ty() {
            Type::List(_) => arg.ty().clone(),
            _ => {
                ctx.error(format!("sorted() argument must be a list, got '{}'", arg.ty().display_name()));
                return None;
            }
        };
        return Some(HirExpr::Call {
            func: "sorted".to_string(),
            args: vec![arg],
            ty: list_ty,
        });
    }

    // reversed(iterable) -> list of element type
    if func_name == "reversed" {
        if call.arguments.args.len() != 1 {
            ctx.error("reversed() takes exactly 1 argument".to_string());
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        let list_ty = match arg.ty() {
            Type::List(_) => arg.ty().clone(),
            _ => {
                ctx.error(format!("reversed() argument must be a list, got '{}'", arg.ty().display_name()));
                return None;
            }
        };
        return Some(HirExpr::Call {
            func: "reversed".to_string(),
            args: vec![arg],
            ty: list_ty,
        });
    }

    // enumerate(iterable) -> list of (int, element) tuples
    if func_name == "enumerate" {
        if call.arguments.args.len() != 1 {
            ctx.error("enumerate() takes exactly 1 argument".to_string());
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        let elem_ty = match arg.ty() {
            Type::List(elem) => *elem.clone(),
            _ => {
                ctx.error(format!("enumerate() argument must be a list, got '{}'", arg.ty().display_name()));
                return None;
            }
        };
        let tuple_ty = Type::Tuple(vec![Type::Int, elem_ty]);
        let result_ty = Type::List(Box::new(tuple_ty));
        return Some(HirExpr::Call {
            func: "enumerate".to_string(),
            args: vec![arg],
            ty: result_ty,
        });
    }

    // zip(iter1, iter2) -> list of (elem1, elem2) tuples
    if func_name == "zip" {
        if call.arguments.args.len() != 2 {
            ctx.error("zip() takes exactly 2 arguments".to_string());
            return None;
        }
        let arg1 = lower_expr(&call.arguments.args[0], ctx)?;
        let arg2 = lower_expr(&call.arguments.args[1], ctx)?;
        let elem1 = match arg1.ty() {
            Type::List(elem) => *elem.clone(),
            _ => {
                ctx.error(format!("zip() argument 1 must be a list, got '{}'", arg1.ty().display_name()));
                return None;
            }
        };
        let elem2 = match arg2.ty() {
            Type::List(elem) => *elem.clone(),
            _ => {
                ctx.error(format!("zip() argument 2 must be a list, got '{}'", arg2.ty().display_name()));
                return None;
            }
        };
        let tuple_ty = Type::Tuple(vec![elem1, elem2]);
        let result_ty = Type::List(Box::new(tuple_ty));
        return Some(HirExpr::Call {
            func: "zip".to_string(),
            args: vec![arg1, arg2],
            ty: result_ty,
        });
    }

    // any(iterable) -> bool
    if func_name == "any" {
        if call.arguments.args.len() != 1 {
            ctx.error("any() takes exactly 1 argument".to_string());
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        return Some(HirExpr::Call {
            func: "any".to_string(),
            args: vec![arg],
            ty: Type::Bool,
        });
    }

    // all(iterable) -> bool
    if func_name == "all" {
        if call.arguments.args.len() != 1 {
            ctx.error("all() takes exactly 1 argument".to_string());
            return None;
        }
        let arg = lower_expr(&call.arguments.args[0], ctx)?;
        return Some(HirExpr::Call {
            func: "all".to_string(),
            args: vec![arg],
            ty: Type::Bool,
        });
    }

    // map(func, iterable) -> list
    if func_name == "map" {
        if call.arguments.args.len() != 2 {
            ctx.error("map() takes exactly 2 arguments (function, iterable)".to_string());
            return None;
        }
        // Lower iterable first to get element type for contextual lambda typing
        let iter_arg = lower_expr(&call.arguments.args[1], ctx)?;
        let elem_ty = match iter_arg.ty() {
            Type::List(elem) => *elem.clone(),
            _ => Type::Any,
        };
        // Lower lambda with contextual typing
        let func_arg = lower_lambda_with_context(&call.arguments.args[0], &[elem_ty], ctx)?;
        // Determine result element type from the function's return type
        let result_elem_ty = match func_arg.ty() {
            Type::Function(ft) => *ft.return_type.clone(),
            _ => Type::Any,
        };
        let result_ty = Type::List(Box::new(result_elem_ty));
        return Some(HirExpr::Call {
            func: "map".to_string(),
            args: vec![func_arg, iter_arg],
            ty: result_ty,
        });
    }

    // filter(func, iterable) -> list (same element type)
    if func_name == "filter" {
        if call.arguments.args.len() != 2 {
            ctx.error("filter() takes exactly 2 arguments (function, iterable)".to_string());
            return None;
        }
        // Lower iterable first to get element type for contextual lambda typing
        let iter_arg = lower_expr(&call.arguments.args[1], ctx)?;
        let elem_ty = match iter_arg.ty() {
            Type::List(elem) => *elem.clone(),
            _ => Type::Any,
        };
        // Lower lambda with contextual typing
        let func_arg = lower_lambda_with_context(&call.arguments.args[0], &[elem_ty], ctx)?;
        let result_ty = iter_arg.ty().clone();
        return Some(HirExpr::Call {
            func: "filter".to_string(),
            args: vec![func_arg, iter_arg],
            ty: result_ty,
        });
    }

    // open(path, mode="r") -> FileHandle  — built-in file open (raises IOError on failure)
    // Matches Python's open() behavior: raises on error, returns FileHandle directly.
    if func_name == "open" {
        let n_args = call.arguments.args.len();
        let n_kwargs = call.arguments.keywords.len();
        let path_arg = if n_args >= 1 {
            lower_expr(&call.arguments.args[0], ctx)?
        } else {
            ctx.error("open() requires at least 1 argument: open(path) or open(path, mode)".to_string());
            return None;
        };
        let mode_arg = if n_args >= 2 {
            lower_expr(&call.arguments.args[1], ctx)?
        } else if let Some(kw) = call.arguments.keywords.iter().find(|k| k.arg.as_deref() == Some("mode")) {
            lower_expr(&kw.value, ctx)?
        } else {
            HirExpr::StringLiteral("r".to_string())
        };
        // Return type: FileHandle (raises IOError on failure — used in try/except blocks)
        // FileHandle methods are defined in io.sifr; register them here for type checking.
        let io_err_ty = Type::Class {
            name: "IOError".to_string(),
            fields: vec![("message".to_string(), Type::Str)],
            methods: vec![],
            parent_class: None,
        };
        let file_handle_ty = Type::Class {
            name: "FileHandle".to_string(),
            fields: vec![
                ("_handle".to_string(), Type::Int),
                ("_mode".to_string(), Type::Str),
            ],
            methods: vec![
                ("read".to_string(), FunctionType::all_borrow(vec![], Type::Result(Box::new(Type::Str), Box::new(io_err_ty.clone())))),
                ("write".to_string(), FunctionType::all_borrow(vec![("data".to_string(), Type::Str)], Type::Result(Box::new(Type::None), Box::new(io_err_ty.clone())))),
                ("readline".to_string(), FunctionType::all_borrow(vec![], Type::Result(Box::new(Type::Union(vec![Type::Str, Type::None])), Box::new(io_err_ty.clone())))),
                ("readlines".to_string(), FunctionType::all_borrow(vec![], Type::Result(Box::new(Type::List(Box::new(Type::Str))), Box::new(io_err_ty.clone())))),
                ("close".to_string(), FunctionType::all_borrow(vec![], Type::None)),
                ("read_bytes".to_string(), FunctionType::all_borrow(vec![], Type::Result(Box::new(Type::List(Box::new(Type::Int))), Box::new(io_err_ty.clone())))),
                ("write_bytes".to_string(), FunctionType::all_borrow(vec![("data".to_string(), Type::List(Box::new(Type::Int)))], Type::Result(Box::new(Type::None), Box::new(io_err_ty.clone())))),
                ("__enter__".to_string(), FunctionType::all_borrow(vec![], Type::Class {
                    name: "FileHandle".to_string(),
                    fields: vec![
                        ("_handle".to_string(), Type::Int),
                        ("_mode".to_string(), Type::Str),
                    ],
                    methods: vec![],
                    parent_class: None,
                })),
                ("__exit__".to_string(), FunctionType::all_borrow(vec![], Type::None)),
            ],
            parent_class: None,
        };
        // Register FileHandle in the class types so method calls work
        ctx.class_types.insert("FileHandle".to_string(), file_handle_ty.clone());
        // Register IOError as a possible exception from this call
        ctx.try_block_error_types.insert("IOError".to_string());
        return Some(HirExpr::Call {
            func: "builtin_open".to_string(),
            args: vec![path_arg, mode_arg],
            ty: file_handle_ty,
        });
    }

    // Check if this is a Callable-typed variable being called
    let callable_info = ctx.scope.lookup(&func_name).and_then(|info| {
        if let Type::Callable(ref param_types, ref conventions, ref ret_type) = info.ty {
            Some((param_types.clone(), conventions.clone(), *ret_type.clone()))
        } else {
            None
        }
    });
    if let Some((param_types, conventions, ret_type)) = callable_info {
        // Lower arguments
        let mut args = Vec::new();
        for arg in &call.arguments.args {
            let expr = lower_expr(arg, ctx)?;
            args.push(expr);
        }
        if args.len() != param_types.len() {
            ctx.error(format!(
                "callable '{}' expects {} argument(s), got {}",
                func_name, param_types.len(), args.len()
            ));
            return None;
        }
        // Type check arguments and apply convention-aware move tracking
        for (i, (arg, param_ty)) in args.iter().zip(param_types.iter()).enumerate() {
            if !arg.ty().is_assignable_to(param_ty) {
                ctx.error(format!(
                    "argument {} of callable '{}': expected '{}', got '{}'",
                    i + 1, func_name, param_ty.display_name(), arg.ty().display_name()
                ));
            }
            // Apply move tracking based on convention
            let convention = conventions.get(i).copied().unwrap_or(ParamConvention::Borrow);
            if convention == ParamConvention::Own {
                // Own convention: transfer ownership, mark variable as moved
                if let HirExpr::Name { name, ty } = arg {
                    if ty.ownership() == OwnershipKind::Move {
                        ctx.scope.mark_moved(name);
                    }
                }
            }
            // Borrow/MutBorrow: no move, variable remains usable
        }
        return Some(HirExpr::Call {
            func: func_name,
            args,
            ty: ret_type,
        });
    }

    let ft = ctx.functions.get(&func_name).cloned().or_else(|| {
        ctx.error(format!("undefined function: '{}'", func_name));
        None
    })?;

    // Lower positional arguments
    let mut positional_args = Vec::new();
    for arg in &call.arguments.args {
        let expr = lower_expr(arg, ctx)?;
        positional_args.push(expr);
    }

    // Lower keyword arguments
    let mut keyword_args: Vec<(String, HirExpr)> = Vec::new();
    for kw in &call.arguments.keywords {
        if let Some(ref arg_name) = kw.arg {
            let expr = lower_expr(&kw.value, ctx)?;
            keyword_args.push((arg_name.to_string(), expr));
        }
    }

    // Resolve keyword arguments to positional order
    let args = if func_name == "print" {
        // print() is special - just pass positional args
        positional_args
    } else if keyword_args.is_empty() {
        // No keyword args - check count and use positional directly
        // Allow fewer args if there are defaults
        let is_vararg = ctx.vararg_functions.contains(&func_name);
        if is_vararg && positional_args.len() >= ft.params.len() - 1 {
            // Vararg function: collect extra args into a list for the last param
            let regular_count = ft.params.len() - 1; // all params except the vararg
            let mut args = Vec::new();
            for i in 0..regular_count {
                args.push(positional_args[i].clone());
            }
            // Collect remaining args into a list literal
            let vararg_elements: Vec<HirExpr> = positional_args[regular_count..].to_vec();
            let elem_ty = if let Type::List(ref elem) = ft.params[regular_count].1 {
                *elem.clone()
            } else {
                Type::Any
            };
            args.push(HirExpr::ListLiteral {
                elements: vararg_elements,
                ty: Type::List(Box::new(elem_ty)),
            });
            // Skip the normal argument handling below
            let is_constructor = ctx.class_types.contains_key(&func_name);
            if is_constructor {
                let ty = ctx.class_types.get(&func_name).unwrap().clone();
                return Some(HirExpr::ConstructorCall {
                    class_name: func_name,
                    args,
                    ty,
                });
            }
            return Some(HirExpr::Call {
                func: func_name,
                args,
                ty: *ft.return_type.clone(),
            });
        } else if positional_args.len() > ft.params.len() {
            ctx.error(format!(
                "function '{}' expects at most {} argument(s), got {}",
                func_name,
                ft.params.len(),
                positional_args.len()
            ));
            return None;
        }
        // Fill in defaults for missing arguments
        if positional_args.len() < ft.params.len() {
            let defaults = ctx.function_defaults.get(&func_name).cloned();
            let mut filled = positional_args;
            for i in filled.len()..ft.params.len() {
                if let Some(ref defs) = defaults {
                    if let Some((_, default_expr)) = defs.iter().find(|(idx, _)| *idx == i) {
                        filled.push(default_expr.clone());
                    } else {
                        ctx.error(format!(
                            "function '{}': missing argument '{}' with no default value",
                            func_name, ft.params[i].0
                        ));
                        return None;
                    }
                } else {
                    ctx.error(format!(
                        "function '{}': missing argument '{}' with no default value",
                        func_name, ft.params[i].0
                    ));
                    return None;
                }
            }
            filled
        } else {
            positional_args
        }
    } else {
        // Resolve keyword arguments into positional order
        let mut resolved = Vec::new();
        let mut used_kwargs: std::collections::HashSet<String> = std::collections::HashSet::new();
        let defaults = ctx.function_defaults.get(&func_name).cloned();

        // Check: no positional args after keyword args (already enforced by parser)
        for (i, (param_name, _param_ty, _)) in ft.params.iter().enumerate() {
            if i < positional_args.len() {
                // Check no duplicate keyword for this position
                if keyword_args.iter().any(|(k, _)| k == param_name) {
                    ctx.error(format!(
                        "function '{}': argument '{}' given both positionally and as keyword",
                        func_name, param_name
                    ));
                    return None;
                }
                resolved.push(positional_args[i].clone());
            } else if let Some(pos) = keyword_args.iter().position(|(k, _)| k == param_name) {
                resolved.push(keyword_args[pos].1.clone());
                used_kwargs.insert(param_name.clone());
            } else {
                // Try to fill from default values
                if let Some(ref defs) = defaults {
                    if let Some((_, default_expr)) = defs.iter().find(|(idx, _)| *idx == i) {
                        resolved.push(default_expr.clone());
                    } else {
                        ctx.error(format!(
                            "function '{}': missing argument '{}' with no default value",
                            func_name, param_name
                        ));
                        return None;
                    }
                } else {
                    ctx.error(format!(
                        "function '{}': missing argument '{}' with no default value",
                        func_name, param_name
                    ));
                    return None;
                }
            }
        }

        // Check for unknown keyword arguments
        for (kw_name, _) in &keyword_args {
            if !ft.params.iter().any(|(p, _, _)| p == kw_name) {
                ctx.error(format!(
                    "function '{}': unexpected keyword argument '{}'",
                    func_name, kw_name
                ));
                return None;
            }
        }

        resolved
    };

    // Check argument types (skip for print)
    if func_name != "print" {
        for (i, (arg, (param_name, param_ty, _))) in args.iter().zip(ft.params.iter()).enumerate() {
            if !arg.ty().is_assignable_to(param_ty) {
                ctx.error(format!(
                    "argument {} ('{}') of function '{}': expected '{}', got '{}'",
                    i + 1,
                    param_name,
                    func_name,
                    param_ty.display_name(),
                    arg.ty().display_name()
                ));
            }
        }
    }

    // Exclusivity check: enforce that the same variable is not passed as mut twice,
    // or as both mut and immutable borrow in the same call.
    {
        let mut mut_borrowed: Vec<String> = Vec::new();
        let mut immut_borrowed: Vec<String> = Vec::new();
        for (i, arg) in args.iter().enumerate() {
            if let HirExpr::Name { name, ty } = arg {
                if ty.ownership() == sifr_type_system::OwnershipKind::Move {
                    let convention = ft.params.get(i).map(|(_, _, c)| *c).unwrap_or(ParamConvention::Borrow);
                    match convention {
                        ParamConvention::MutBorrow => {
                            if mut_borrowed.contains(name) {
                                ctx.error(format!(
                                    "cannot borrow '{}' as mutable more than once in the same call to '{}'",
                                    name, func_name
                                ));
                            } else if immut_borrowed.contains(name) {
                                ctx.error(format!(
                                    "cannot borrow '{}' as mutable because it is already borrowed as immutable in the same call to '{}'",
                                    name, func_name
                                ));
                            }
                            mut_borrowed.push(name.clone());
                        }
                        ParamConvention::Borrow => {
                            if mut_borrowed.contains(name) {
                                ctx.error(format!(
                                    "cannot borrow '{}' as immutable because it is already borrowed as mutable in the same call to '{}'",
                                    name, func_name
                                ));
                            }
                            immut_borrowed.push(name.clone());
                        }
                        ParamConvention::Own => {} // ownership transfer, no borrow conflict
                    }
                }
            }
        }
    }

    // Track ownership: only mark arguments as moved when the parameter convention is Own
    // and the argument type is Move. Borrow and MutBorrow do not consume the value.
    for (i, arg) in args.iter().enumerate() {
        if let HirExpr::Name { name, ty } = arg {
            if ty.ownership() == sifr_type_system::OwnershipKind::Move {
                let convention = ft.params.get(i).map(|(_, _, c)| *c).unwrap_or(ParamConvention::Borrow);
                if convention == ParamConvention::Own {
                    ctx.scope.mark_moved(name);
                }
            }
        }
    }

    // If this is a generic function, infer type variable bindings and substitute
    let return_type = if ctx.generic_functions.contains_key(&func_name) {
        let mut bindings = HashMap::new();
        for (arg, (_, param_ty, _)) in args.iter().zip(ft.params.iter()) {
            infer_type_var_bindings(param_ty, arg.ty(), &mut bindings);
        }
        // Check protocol bounds on type parameters (scoped to this function)
        let func_bounds = ctx.type_param_bounds.get(&func_name);
        let bound_errors: Vec<String> = bindings.iter().flat_map(|(tv_name, concrete_ty)| {
            func_bounds.into_iter().flat_map(move |owner_bounds| {
                owner_bounds.get(tv_name).into_iter().flat_map(move |bounds| {
                    bounds.iter().filter_map(move |bound| {
                        if !type_satisfies_bound(concrete_ty, bound) {
                            Some(format!(
                                "type '{}' does not implement protocol '{}' required by type parameter '{}'",
                                concrete_ty.display_name(), bound, tv_name
                            ))
                        } else {
                            None
                        }
                    })
                })
            })
        }).collect();
        for err in bound_errors {
            ctx.error(err);
        }
        if bindings.is_empty() {
            *ft.return_type
        } else {
            substitute_type_vars(&ft.return_type, &bindings)
        }
    } else {
        *ft.return_type
    };

    // If this is a class constructor call, emit ConstructorCall
    if ctx.class_types.contains_key(&func_name) {
        Some(HirExpr::ConstructorCall {
            class_name: func_name,
            args,
            ty: return_type,
        })
    } else {
        Some(HirExpr::Call {
            func: func_name,
            args,
            ty: return_type,
        })
    }
}

fn lower_fstring(fstring: &ExprFString, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let mut parts = Vec::new();

    for part in &fstring.value {
        match part {
            sifr_python_ast::FStringPart::Literal(s) => {
                parts.push(HirFStringPart::Literal(s.to_string()));
            }
            sifr_python_ast::FStringPart::FString(fs) => {
                for element in fs.elements.iter() {
                    match element {
                        FStringElement::Literal(lit) => {
                            parts.push(HirFStringPart::Literal(lit.value.to_string()));
                        }
                        FStringElement::Expression(expr_elem) => {
                            let expr = lower_expr(&expr_elem.expression, ctx)?;
                            parts.push(HirFStringPart::Expr(expr));
                        }
                    }
                }
            }
        }
    }

    Some(HirExpr::FString {
        parts,
        ty: Type::Str,
    })
}

fn lower_tuple_unpack_assign(tuple: &ExprTuple, value: &Expr, ctx: &mut LowerCtx) -> Option<HirStmt> {
    // Extract target names
    let mut target_names = Vec::new();
    for elt in &tuple.elts {
        match elt {
            Expr::Name(n) => target_names.push(n.id.to_string()),
            _ => {
                ctx.error("tuple unpacking target must be a simple name".to_string());
                return None;
            }
        }
    }

    // Lower the value expression
    let value_expr = lower_expr(value, ctx)?;
    let value_ty = value_expr.ty().clone();

    // Check that the value is a tuple with matching length
    let elem_types = match &value_ty {
        Type::Tuple(elems) => {
            if elems.len() != target_names.len() {
                ctx.error(format!(
                    "tuple unpacking: expected {} values, got {}",
                    target_names.len(),
                    elems.len()
                ));
                return None;
            }
            elems.clone()
        }
        _ => {
            ctx.error(format!(
                "cannot unpack non-tuple type '{}'",
                value_ty.display_name()
            ));
            return None;
        }
    };

    // Define variables in scope
    let mut targets = Vec::new();
    for (name, ty) in target_names.into_iter().zip(elem_types.into_iter()) {
        ctx.scope.define(name.clone(), ty.clone());
        targets.push((name, ty));
    }

    Some(HirStmt::TupleUnpack {
        targets,
        value: value_expr,
    })
}

fn lower_star_unpack_assign(tuple: &ExprTuple, value: &Expr, ctx: &mut LowerCtx) -> Option<HirStmt> {
    let value_expr = lower_expr(value, ctx)?;
    let value_ty = value_expr.ty().clone();

    // Get the element type from the list
    let elem_ty = match &value_ty {
        Type::List(elem) => *elem.clone(),
        _ => {
            ctx.error("star unpacking requires a list type".to_string());
            return None;
        }
    };

    let mut before = Vec::new();
    let mut star: Option<(String, Type)> = None;
    let mut after = Vec::new();

    for elt in &tuple.elts {
        match elt {
            Expr::Starred(starred) => {
                if star.is_some() {
                    ctx.error("multiple starred expressions in assignment".to_string());
                    return None;
                }
                if let Expr::Name(n) = starred.value.as_ref() {
                    let name = n.id.to_string();
                    let star_ty = Type::List(Box::new(elem_ty.clone()));
                    ctx.scope.define(name.clone(), star_ty.clone());
                    star = Some((name, star_ty));
                } else {
                    ctx.error("starred target must be a simple name".to_string());
                    return None;
                }
            }
            Expr::Name(n) => {
                let name = n.id.to_string();
                ctx.scope.define(name.clone(), elem_ty.clone());
                if star.is_none() {
                    before.push((name, elem_ty.clone()));
                } else {
                    after.push((name, elem_ty.clone()));
                }
            }
            _ => {
                ctx.error("star unpacking target must be a simple name".to_string());
                return None;
            }
        }
    }

    let star = star.unwrap_or_else(|| {
        ctx.error("star unpacking requires a starred expression".to_string());
        ("_".to_string(), Type::List(Box::new(elem_ty.clone())))
    });

    Some(HirStmt::StarUnpack {
        before,
        star,
        after,
        value: value_expr,
    })
}

fn lower_list_literal(list: &ExprList, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let mut elements = Vec::new();
    let mut elem_ty: Option<Type> = None;

    for elt in &list.elts {
        let expr = lower_expr(elt, ctx)?;
        let ty = expr.ty().clone();
        if let Some(ref expected) = elem_ty {
            if !ty.is_assignable_to(expected) {
                ctx.error(format!(
                    "list element type mismatch: expected '{}', got '{}'",
                    expected.display_name(),
                    ty.display_name()
                ));
            }
        } else {
            elem_ty = Some(ty);
        }
        elements.push(expr);
    }

    let final_elem_ty = elem_ty.unwrap_or(Type::Any);
    let list_ty = Type::List(Box::new(final_elem_ty));

    Some(HirExpr::ListLiteral {
        elements,
        ty: list_ty,
    })
}

fn lower_set_literal(set: &ExprSet, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let mut elements = Vec::new();
    let mut elem_ty: Option<Type> = None;

    for elt in &set.elts {
        let expr = lower_expr(elt, ctx)?;
        let ty = expr.ty().clone();
        if let Some(ref expected) = elem_ty {
            if !ty.is_assignable_to(expected) {
                ctx.error(format!(
                    "set element type mismatch: expected '{}', got '{}'",
                    expected.display_name(),
                    ty.display_name()
                ));
            }
        } else {
            elem_ty = Some(ty);
        }
        elements.push(expr);
    }

    let final_elem_ty = elem_ty.unwrap_or(Type::Any);
    let set_ty = Type::Set(Box::new(final_elem_ty));

    Some(HirExpr::SetLiteral {
        elements,
        ty: set_ty,
    })
}

fn lower_dict_literal(dict: &ExprDict, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let mut keys = Vec::new();
    let mut values = Vec::new();
    let mut key_ty: Option<Type> = None;
    let mut val_ty: Option<Type> = None;

    for item in &dict.items {
        if let Some(ref key_expr) = item.key {
            let key = lower_expr(key_expr, ctx)?;
            let kt = key.ty().clone();
            if let Some(ref expected) = key_ty {
                if !kt.is_assignable_to(expected) {
                    ctx.error(format!(
                        "dict key type mismatch: expected '{}', got '{}'",
                        expected.display_name(),
                        kt.display_name()
                    ));
                }
            } else {
                key_ty = Some(kt);
            }
            keys.push(key);
        } else {
            ctx.error("dict unpacking (**) not supported".to_string());
            return None;
        }

        let val = lower_expr(&item.value, ctx)?;
        let vt = val.ty().clone();
        if let Some(ref expected) = val_ty {
            if !vt.is_assignable_to(expected) {
                ctx.error(format!(
                    "dict value type mismatch: expected '{}', got '{}'",
                    expected.display_name(),
                    vt.display_name()
                ));
            }
        } else {
            val_ty = Some(vt);
        }
        values.push(val);
    }

    let final_key_ty = key_ty.unwrap_or(Type::Any);
    let final_val_ty = val_ty.unwrap_or(Type::Any);
    let dict_ty = Type::Dict(Box::new(final_key_ty), Box::new(final_val_ty));

    Some(HirExpr::DictLiteral {
        keys,
        values,
        ty: dict_ty,
    })
}

fn lower_tuple_literal(tuple: &ExprTuple, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let mut elements = Vec::new();
    let mut elem_types = Vec::new();

    for elt in &tuple.elts {
        let expr = lower_expr(elt, ctx)?;
        elem_types.push(expr.ty().clone());
        elements.push(expr);
    }

    let tuple_ty = Type::Tuple(elem_types);

    Some(HirExpr::TupleLiteral {
        elements,
        ty: tuple_ty,
    })
}

fn lower_subscript(sub: &ExprSubscript, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let object = lower_expr(&sub.value, ctx)?;
    let object_ty = object.ty().clone();

    // Check if the slice is a Slice expression (x[start:stop] or x[start:stop:step])
    if let Expr::Slice(slice_expr) = sub.slice.as_ref() {
        let start = if let Some(ref s) = slice_expr.lower {
            Some(Box::new(lower_expr(s, ctx)?))
        } else {
            None
        };
        let stop = if let Some(ref s) = slice_expr.upper {
            Some(Box::new(lower_expr(s, ctx)?))
        } else {
            None
        };
        let step = if let Some(ref s) = slice_expr.step {
            Some(Box::new(lower_expr(s, ctx)?))
        } else {
            None
        };

        // Determine result type for slicing
        let result_ty = match &object_ty {
            Type::List(elem_ty) => Type::List(elem_ty.clone()),
            Type::Str => Type::Str,
            Type::Tuple(elems) => {
                // Compile-time tuple slicing: indices must be integer literals
                if let (Some(start_expr), Some(stop_expr)) = (&start, &stop) {
                    if let (HirExpr::IntLiteral(s), HirExpr::IntLiteral(e)) = (start_expr.as_ref(), stop_expr.as_ref()) {
                        let s = if *s < 0 { (elems.len() as i64 + s) as usize } else { *s as usize };
                        let e = if *e < 0 { (elems.len() as i64 + e) as usize } else { *e as usize };
                        if s <= e && e <= elems.len() {
                            Type::Tuple(elems[s..e].to_vec())
                        } else {
                            ctx.error("tuple slice indices out of range".to_string());
                            Type::Any
                        }
                    } else {
                        ctx.error("tuple slicing requires compile-time constant indices".to_string());
                        Type::Any
                    }
                } else {
                    // Partial slice on tuple
                    let s = start.as_ref().and_then(|e| if let HirExpr::IntLiteral(v) = e.as_ref() { Some(*v as usize) } else { None }).unwrap_or(0);
                    let e = stop.as_ref().and_then(|e| if let HirExpr::IntLiteral(v) = e.as_ref() { Some(*v as usize) } else { None }).unwrap_or(elems.len());
                    if s <= e && e <= elems.len() {
                        Type::Tuple(elems[s..e].to_vec())
                    } else {
                        Type::Tuple(elems.clone())
                    }
                }
            }
            _ => {
                ctx.error(format!("cannot slice type '{}'", object_ty.display_name()));
                Type::Any
            }
        };

        return Some(HirExpr::Slice {
            object: Box::new(object),
            start,
            stop,
            step,
            ty: result_ty,
        });
    }

    let index = lower_expr(&sub.slice, ctx)?;
    let index_ty = index.ty().clone();

    let result_ty = object_ty.index_result_type(&index_ty).unwrap_or_else(|| {
        ctx.error(format!(
            "cannot index type '{}' with '{}'",
            object_ty.display_name(),
            index_ty.display_name()
        ));
        Type::Any
    });

    Some(HirExpr::Index {
        object: Box::new(object),
        index: Box::new(index),
        ty: result_ty,
    })
}

fn lower_attribute(attr: &ExprAttribute, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let field_name = attr.attr.to_string();

    // Check for enum variant access: Color.RED
    if let Expr::Name(name) = attr.value.as_ref() {
        let class_name = name.id.to_string();
        if let Some(ty) = ctx.class_types.get(&class_name).cloned() {
            if let Type::Enum { ref variants, .. } = ty {
                if variants.iter().any(|(v, _)| v == &field_name) {
                    return Some(HirExpr::EnumVariant {
                        enum_name: class_name,
                        variant: field_name,
                        ty,
                    });
                }
            }
        }
    }

    let object = lower_expr(&attr.value, ctx)?;
    let object_ty = object.ty().clone();

    // Check if the object is a class instance with this field
    if let Type::Class { name: _, fields, .. } = &object_ty {
        if let Some((_, field_ty)) = fields.iter().find(|(n, _)| n == &field_name) {
            return Some(HirExpr::FieldAccess {
                object: Box::new(object),
                field: field_name,
                ty: field_ty.clone(),
            });
        }
        ctx.error(format!(
            "type '{}' has no field '{}'",
            object_ty.display_name(),
            field_name
        ));
        return None;
    }

    // Check if the object is an enum instance - access .name or .value
    if let Type::Enum { name: enum_name, .. } = &object_ty {
        match field_name.as_str() {
            "name" => {
                return Some(HirExpr::FieldAccess {
                    object: Box::new(object),
                    field: "name".to_string(),
                    ty: Type::Str,
                });
            }
            "value" => {
                return Some(HirExpr::FieldAccess {
                    object: Box::new(object),
                    field: "value".to_string(),
                    ty: Type::Int,
                });
            }
            _ => {
                ctx.error(format!("enum '{}' has no attribute '{}'", enum_name, field_name));
                return None;
            }
        }
    }

    // Not a class field access -- report unsupported
    ctx.error(format!("attribute access '.{}' is not supported as an expression; use as a method call", field_name));
    None
}

fn lower_method_call(attr: &ExprAttribute, call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    // Handle super().__init__() and super().method() calls
    if let Expr::Call(super_call) = attr.value.as_ref() {
        if let Expr::Name(name) = super_call.func.as_ref() {
            if name.id.as_str() == "super" {
                let method_name = attr.attr.to_string();
                if let Some(parent_name) = ctx.current_parent_class.clone() {
                    // Lower arguments
                    let mut args = Vec::new();
                    for arg in &call.arguments.args {
                        let expr = lower_expr(arg, ctx)?;
                        args.push(expr);
                    }

                    return Some(HirExpr::SuperCall {
                        parent_class: parent_name,
                        method: if method_name == "__init__" { "new".to_string() } else { method_name },
                        args,
                        ty: Type::None,
                    });
                }
                ctx.error("super() used outside of a class with a parent".to_string());
                return None;
            }
        }
    }

    // Handle ClassName.method() calls (classmethod/staticmethod)
    if let Expr::Name(name) = attr.value.as_ref() {
        let class_name = name.id.to_string();
        if ctx.class_types.contains_key(&class_name) {
            let method_name = attr.attr.to_string();
            // Lower arguments
            let mut args = Vec::new();
            for arg in &call.arguments.args {
                let expr = lower_expr(arg, ctx)?;
                args.push(expr);
            }
            // Look up the method's return type from the class type
            if let Some(class_ty) = ctx.class_types.get(&class_name) {
                if let Type::Class { methods, .. } = class_ty {
                    if let Some((_, ft)) = methods.iter().find(|(n, _)| n == &method_name) {
                        let return_ty = *ft.return_type.clone();
                        return Some(HirExpr::Call {
                            func: format!("{}::{}", class_name, method_name),
                            args,
                            ty: return_ty,
                        });
                    }
                }
            }
            ctx.error(format!("type '{}' has no class/static method '{}'", class_name, method_name));
            return None;
        }
    }

    let object = lower_expr(&attr.value, ctx)?;
    let object_ty = object.ty().clone();
    let method_name = attr.attr.to_string();

    // Lower arguments
    let mut args = Vec::new();
    for arg in &call.arguments.args {
        let expr = lower_expr(arg, ctx)?;
        args.push(expr);
    }

    // Resolve method return type based on object type and method name
    let return_ty = resolve_method_type(&object_ty, &method_name, &args, ctx)?;

    Some(HirExpr::MethodCall {
        object: Box::new(object),
        method: method_name,
        args,
        ty: return_ty,
    })
}

/// Resolve the return type of a method call on a given type.
fn resolve_method_type(object_ty: &Type, method: &str, args: &[HirExpr], ctx: &mut LowerCtx) -> Option<Type> {
    match object_ty {
        Type::List(elem_ty) => match method {
            "append" => {
                if args.len() != 1 {
                    ctx.error(format!("list.append() takes exactly 1 argument, got {}", args.len()));
                    return None;
                }
                if !args[0].ty().is_assignable_to(elem_ty) {
                    ctx.error(format!(
                        "list.append() argument type '{}' is not compatible with list element type '{}'",
                        args[0].ty().display_name(),
                        elem_ty.display_name()
                    ));
                }
                Some(Type::None)
            }
            "extend" => {
                if args.len() != 1 {
                    ctx.error(format!("list.extend() takes exactly 1 argument, got {}", args.len()));
                    return None;
                }
                Some(Type::None)
            }
            "insert" => {
                if args.len() != 2 {
                    ctx.error(format!("list.insert() takes exactly 2 arguments, got {}", args.len()));
                    return None;
                }
                Some(Type::None)
            }
            "clear" => {
                if !args.is_empty() {
                    ctx.error("list.clear() takes no arguments".to_string());
                    return None;
                }
                Some(Type::None)
            }
            "copy" => {
                if !args.is_empty() {
                    ctx.error("list.copy() takes no arguments".to_string());
                    return None;
                }
                Some(Type::List(elem_ty.clone()))
            }
            "reverse" => {
                if !args.is_empty() {
                    ctx.error("list.reverse() takes no arguments".to_string());
                    return None;
                }
                Some(Type::None)
            }
            "sort" => {
                if !args.is_empty() {
                    ctx.error("list.sort() takes no arguments in this milestone".to_string());
                    return None;
                }
                Some(Type::None)
            }
            "count" => {
                if args.len() != 1 {
                    ctx.error(format!("list.count() takes exactly 1 argument, got {}", args.len()));
                    return None;
                }
                Some(Type::Int)
            }
            "contains" => {
                if args.len() != 1 {
                    ctx.error(format!("list.contains() takes exactly 1 argument, got {}", args.len()));
                    return None;
                }
                Some(Type::Bool)
            }
            "len" => {
                if !args.is_empty() {
                    ctx.error("list.len() takes no arguments".to_string());
                    return None;
                }
                Some(Type::Int)
            }
            "pop" => {
                if !args.is_empty() {
                    ctx.error("list.pop() takes no arguments".to_string());
                    return None;
                }
                // pop() returns Option[T] = T | None
                Some(Type::Union(vec![*elem_ty.clone(), Type::None]))
            }
            "remove" => {
                if args.len() != 1 {
                    ctx.error(format!("list.remove() takes exactly 1 argument, got {}", args.len()));
                    return None;
                }
                Some(Type::None)
            }
            "index" => {
                if args.len() != 1 {
                    ctx.error(format!("list.index() takes exactly 1 argument, got {}", args.len()));
                    return None;
                }
                // Returns Option[int] = int | None (safe: no panic if not found)
                Some(Type::Union(vec![Type::Int, Type::None]))
            }
            _ => {
                ctx.error(format!("list has no method '{}'", method));
                None
            }
        },
        Type::Dict(key_ty, val_ty) => match method {
            "len" => {
                if !args.is_empty() {
                    ctx.error("dict.len() takes no arguments".to_string());
                    return None;
                }
                Some(Type::Int)
            }
            "keys" => {
                if !args.is_empty() {
                    ctx.error("dict.keys() takes no arguments".to_string());
                    return None;
                }
                Some(Type::List(key_ty.clone()))
            }
            "values" => {
                if !args.is_empty() {
                    ctx.error("dict.values() takes no arguments".to_string());
                    return None;
                }
                Some(Type::List(val_ty.clone()))
            }
            "items" => {
                if !args.is_empty() {
                    ctx.error("dict.items() takes no arguments".to_string());
                    return None;
                }
                Some(Type::List(Box::new(Type::Tuple(vec![*key_ty.clone(), *val_ty.clone()]))))
            }
            "update" => {
                if args.len() != 1 {
                    ctx.error(format!("dict.update() takes exactly 1 argument, got {}", args.len()));
                    return None;
                }
                Some(Type::None)
            }
            "clear" => {
                if !args.is_empty() {
                    ctx.error("dict.clear() takes no arguments".to_string());
                    return None;
                }
                Some(Type::None)
            }
            "copy" => {
                if !args.is_empty() {
                    ctx.error("dict.copy() takes no arguments".to_string());
                    return None;
                }
                Some(Type::Dict(key_ty.clone(), val_ty.clone()))
            }
            "contains" => {
                if args.len() != 1 {
                    ctx.error(format!("dict.contains() takes exactly 1 argument, got {}", args.len()));
                    return None;
                }
                Some(Type::Bool)
            }
            "get" => {
                if args.is_empty() || args.len() > 2 {
                    ctx.error(format!("dict.get() takes 1 or 2 arguments, got {}", args.len()));
                    return None;
                }
                if args.len() == 2 {
                    // dict.get(key, default) -> V (returns default if key not found)
                    Some(*val_ty.clone())
                } else {
                    // dict.get(key) -> V | None
                    Some(Type::Union(vec![*val_ty.clone(), Type::None]))
                }
            }
            "pop" => {
                if args.len() != 1 {
                    ctx.error(format!("dict.pop() takes exactly 1 argument, got {}", args.len()));
                    return None;
                }
                // pop() returns Option[V] = V | None
                Some(Type::Union(vec![*val_ty.clone(), Type::None]))
            }
            _ => {
                ctx.error(format!("dict has no method '{}'", method));
                None
            }
        },
        Type::Set(elem_ty) => match method {
            "len" => {
                if !args.is_empty() {
                    ctx.error("set.len() takes no arguments".to_string());
                    return None;
                }
                Some(Type::Int)
            }
            "add" => {
                if args.len() != 1 {
                    ctx.error(format!("set.add() takes exactly 1 argument, got {}", args.len()));
                    return None;
                }
                Some(Type::None)
            }
            "remove" | "discard" => {
                if args.len() != 1 {
                    ctx.error(format!("set.{}() takes exactly 1 argument, got {}", method, args.len()));
                    return None;
                }
                Some(Type::None)
            }
            "contains" => {
                if args.len() != 1 {
                    ctx.error(format!("set.contains() takes exactly 1 argument, got {}", args.len()));
                    return None;
                }
                Some(Type::Bool)
            }
            "clear" => {
                if !args.is_empty() {
                    ctx.error("set.clear() takes no arguments".to_string());
                    return None;
                }
                Some(Type::None)
            }
            "copy" => {
                if !args.is_empty() {
                    ctx.error("set.copy() takes no arguments".to_string());
                    return None;
                }
                Some(Type::Set(elem_ty.clone()))
            }
            "union" | "intersection" | "difference" | "symmetric_difference" => {
                if args.len() != 1 {
                    ctx.error(format!("set.{}() takes exactly 1 argument, got {}", method, args.len()));
                    return None;
                }
                Some(Type::Set(elem_ty.clone()))
            }
            "issubset" | "issuperset" | "isdisjoint" => {
                if args.len() != 1 {
                    ctx.error(format!("set.{}() takes exactly 1 argument, got {}", method, args.len()));
                    return None;
                }
                Some(Type::Bool)
            }
            "pop" => {
                if !args.is_empty() {
                    ctx.error("set.pop() takes no arguments".to_string());
                    return None;
                }
                // Returns Option[T] = T | None (safe: no panic on empty set)
                Some(Type::Union(vec![*elem_ty.clone(), Type::None]))
            }
            _ => {
                ctx.error(format!("set has no method '{}'", method));
                None
            }
        },
        Type::Str => match method {
            "len" => Some(Type::Int),
            "upper" | "lower" | "strip" | "lstrip" | "rstrip" | "title" | "capitalize" | "swapcase" => Some(Type::Str),
            "startswith" | "endswith" => {
                if args.len() != 1 {
                    ctx.error(format!("str.{}() takes exactly 1 argument, got {}", method, args.len()));
                    return None;
                }
                Some(Type::Bool)
            }
            "isdigit" | "isalpha" | "isalnum" | "isspace" | "isupper" | "islower" => {
                if !args.is_empty() {
                    ctx.error(format!("str.{}() takes no arguments", method));
                    return None;
                }
                Some(Type::Bool)
            }
            "split" => {
                if args.len() > 1 {
                    ctx.error(format!("str.split() takes 0 or 1 arguments, got {}", args.len()));
                    return None;
                }
                Some(Type::List(Box::new(Type::Str)))
            }
            "replace" => {
                if args.len() != 2 {
                    ctx.error(format!("str.replace() takes exactly 2 arguments, got {}", args.len()));
                    return None;
                }
                Some(Type::Str)
            }
            "join" => {
                if args.len() != 1 {
                    ctx.error(format!("str.join() takes exactly 1 argument, got {}", args.len()));
                    return None;
                }
                Some(Type::Str)
            }
            "count" => {
                if args.len() != 1 {
                    ctx.error(format!("str.count() takes exactly 1 argument, got {}", args.len()));
                    return None;
                }
                Some(Type::Int)
            }
            "center" | "ljust" | "rjust" | "zfill" => {
                if args.len() != 1 {
                    ctx.error(format!("str.{}() takes exactly 1 argument, got {}", method, args.len()));
                    return None;
                }
                Some(Type::Str)
            }
            "find" => {
                if args.len() != 1 {
                    ctx.error(format!("str.find() takes exactly 1 argument, got {}", args.len()));
                    return None;
                }
                // find() returns Option[int] = int | None
                Some(Type::Union(vec![Type::Int, Type::None]))
            }
            _ => {
                ctx.error(format!("str has no method '{}'", method));
                None
            }
        },
        Type::Tuple(_) => match method {
            "len" => Some(Type::Int),
            "count" => {
                if args.len() != 1 {
                    ctx.error(format!("tuple.count() takes exactly 1 argument, got {}", args.len()));
                    return None;
                }
                Some(Type::Int)
            }
            _ => {
                ctx.error(format!("tuple has no method '{}'", method));
                None
            }
        },
        Type::Class { name, fields, methods, .. } => {
            if let Some((_, ft)) = methods.iter().find(|(n, _)| n == method) {
                // Check argument count
                if args.len() != ft.params.len() {
                    ctx.error(format!(
                        "{}.{}() takes {} argument(s), got {}",
                        name, method, ft.params.len(), args.len()
                    ));
                    return None;
                }
                // Check argument types
                for (i, (arg, (param_name, param_ty, _))) in args.iter().zip(ft.params.iter()).enumerate() {
                    if !arg.ty().is_assignable_to(param_ty) {
                        ctx.error(format!(
                            "argument {} ('{}') of {}.{}(): expected '{}', got '{}'",
                            i + 1, param_name, name, method,
                            param_ty.display_name(), arg.ty().display_name()
                        ));
                    }
                }
                Some(*ft.return_type.clone())
            } else if let Some((_, field_ty)) = fields.iter().find(|(n, _)| n == method) {
                // Check if the field is a Callable type — allow calling it like a method
                if let Type::Callable(param_types, _, ret_type) = field_ty {
                    if args.len() != param_types.len() {
                        ctx.error(format!(
                            "{}.{}() (callable field) takes {} argument(s), got {}",
                            name, method, param_types.len(), args.len()
                        ));
                        return None;
                    }
                    for (i, (arg, param_ty)) in args.iter().zip(param_types.iter()).enumerate() {
                        if !arg.ty().is_assignable_to(param_ty) {
                            ctx.error(format!(
                                "argument {} of {}.{}(): expected '{}', got '{}'",
                                i + 1, name, method,
                                param_ty.display_name(), arg.ty().display_name()
                            ));
                        }
                    }
                    Some(*ret_type.clone())
                } else {
                    ctx.error(format!("field '{}' of class '{}' is not callable (type: '{}')", method, name, field_ty.display_name()));
                    None
                }
            } else {
                ctx.error(format!("class '{}' has no method '{}'", name, method));
                None
            }
        }
        Type::Protocol { name, methods, .. } => {
            if let Some((_, ft)) = methods.iter().find(|(n, _)| n == method) {
                if args.len() != ft.params.len() {
                    ctx.error(format!(
                        "{}.{}() takes {} argument(s), got {}",
                        name, method, ft.params.len(), args.len()
                    ));
                }
                Some(*ft.return_type.clone())
            } else {
                ctx.error(format!("protocol '{}' has no method '{}'", name, method));
                None
            }
        }
        Type::Newtype { name, inner } => {
            // Newtype has a built-in `value()` method that returns the inner type
            if method == "value" {
                if !args.is_empty() {
                    ctx.error(format!("{}.value() takes no arguments", name));
                    return None;
                }
                Some(*inner.clone())
            } else {
                // Delegate to the inner type's methods
                resolve_method_type(inner, method, args, ctx)
            }
        }
        Type::Enum { name, .. } => {
            match method {
                "name" => {
                    if !args.is_empty() {
                        ctx.error(format!("{}.name() takes no arguments", name));
                        return None;
                    }
                    Some(Type::Str)
                }
                "value" => {
                    if !args.is_empty() {
                        ctx.error(format!("{}.value() takes no arguments", name));
                        return None;
                    }
                    Some(Type::Int)
                }
                _ => {
                    // Check user-defined methods registered in functions
                    let method_key = format!("{}.{}", name, method);
                    if let Some(ft) = ctx.functions.get(&method_key).cloned() {
                        return Some(*ft.return_type.clone());
                    }
                    ctx.error(format!("enum '{}' has no method '{}'", name, method));
                    None
                }
            }
        }
        Type::BigInt => {
            match method {
                "clone" => {
                    if !args.is_empty() {
                        ctx.error("bigint.clone() takes no arguments".to_string());
                        return None;
                    }
                    Some(Type::BigInt)
                }
                _ => {
                    ctx.error(format!("type 'bigint' has no method '{}'", method));
                    None
                }
            }
        }
        _ => {
            ctx.error(format!(
                "type '{}' has no method '{}'",
                object_ty.display_name(),
                method
            ));
            None
        }
    }
}

fn lower_len_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if call.arguments.args.len() != 1 {
        ctx.error(format!("len() takes exactly 1 argument, got {}", call.arguments.args.len()));
        return None;
    }
    let arg = lower_expr(&call.arguments.args[0], ctx)?;
    let arg_ty = arg.ty().clone();

    // len() works on str, list, dict, tuple, set
    // Also works on T|None where T is a valid len() argument (auto-unwrap)
    let effective_ty = if let Type::Union(members) = &arg_ty {
        let non_none: Vec<&Type> = members.iter().filter(|m| !matches!(m, Type::None)).collect();
        if non_none.len() == 1 {
            non_none[0].clone()
        } else {
            arg_ty.clone()
        }
    } else {
        arg_ty.clone()
    };
    match &effective_ty {
        Type::Str | Type::List(_) | Type::Dict(_, _) | Type::Tuple(_) | Type::Set(_) => {
            Some(HirExpr::MethodCall {
                object: Box::new(arg),
                method: "len".to_string(),
                args: vec![],
                ty: Type::Int,
            })
        }
        _ => {
            ctx.error(format!(
                "len() argument must be a string, list, dict, or tuple, got '{}'",
                arg_ty.display_name()
            ));
            None
        }
    }
}

fn lower_isinstance_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if call.arguments.args.len() != 2 {
        ctx.error(format!(
            "isinstance() takes exactly 2 arguments, got {}",
            call.arguments.args.len()
        ));
        return None;
    }
    let arg = lower_expr(&call.arguments.args[0], ctx)?;
    // Extract the type name as a string literal so codegen can use it for match arms
    let type_name = match &call.arguments.args[1] {
        Expr::Name(n) => n.id.to_string(),
        _ => "unknown".to_string(),
    };
    // isinstance() always returns bool -- the narrowing happens at the if-statement level
    // We pass both the variable and the type name string to codegen
    Some(HirExpr::Call {
        func: "isinstance".to_string(),
        args: vec![arg, HirExpr::StringLiteral(type_name)],
        ty: Type::Bool,
    })
}

fn lower_reveal_type_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if call.arguments.args.len() != 1 {
        ctx.error(format!(
            "reveal_type() takes exactly 1 argument, got {}",
            call.arguments.args.len()
        ));
        return None;
    }
    let arg = lower_expr(&call.arguments.args[0], ctx)?;
    let ty = arg.ty().clone();
    // Store the reveal_type diagnostic (not an error, just informational)
    ctx.reveal_types.push(format!("reveal_type: {}", ty.display_name()));
    // reveal_type returns the value unchanged, so we emit a print of the type at runtime
    // For now, just return the argument expression
    Some(arg)
}

fn lower_range_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let args: Vec<_> = call.arguments.args.iter().collect();

    match args.len() {
        1 => {
            // range(end) -> 0..end
            let end = lower_expr(args[0], ctx)?;
            if end.ty() != &Type::Int {
                ctx.error(format!(
                    "range() argument must be 'int', got '{}'",
                    end.ty().display_name()
                ));
                return None;
            }
            Some(HirExpr::RangeLiteral {
                start: Box::new(HirExpr::IntLiteral(0)),
                end: Box::new(end),
                step: None,
                ty: Type::Range,
            })
        }
        2 => {
            // range(start, end) -> start..end
            let start = lower_expr(args[0], ctx)?;
            let end = lower_expr(args[1], ctx)?;
            if start.ty() != &Type::Int {
                ctx.error(format!(
                    "range() start argument must be 'int', got '{}'",
                    start.ty().display_name()
                ));
                return None;
            }
            if end.ty() != &Type::Int {
                ctx.error(format!(
                    "range() end argument must be 'int', got '{}'",
                    end.ty().display_name()
                ));
                return None;
            }
            Some(HirExpr::RangeLiteral {
                start: Box::new(start),
                end: Box::new(end),
                step: None,
                ty: Type::Range,
            })
        }
        3 => {
            // range(start, end, step) -> (start..end).step_by(step)
            let start = lower_expr(args[0], ctx)?;
            let end = lower_expr(args[1], ctx)?;
            let step = lower_expr(args[2], ctx)?;
            Some(HirExpr::RangeLiteral {
                start: Box::new(start),
                end: Box::new(end),
                step: Some(Box::new(step)),
                ty: Type::Range,
            })
        }
        _ => {
            ctx.error(format!(
                "range() takes 1, 2, or 3 arguments, got {}",
                args.len()
            ));
            None
        }
    }
}

fn lower_if_expr(if_expr: &ExprIf, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let condition = lower_expr(&if_expr.test, ctx)?;
    let then_expr = lower_expr(&if_expr.body, ctx)?;
    let else_expr = lower_expr(&if_expr.orelse, ctx)?;

    let then_ty = then_expr.ty().clone();
    let else_ty = else_expr.ty().clone();

    if !then_ty.is_assignable_to(&else_ty) && !else_ty.is_assignable_to(&then_ty) {
        ctx.error(format!(
            "if expression branches have incompatible types: '{}' and '{}'",
            then_ty.display_name(),
            else_ty.display_name()
        ));
        return None;
    }

    Some(HirExpr::IfExpr {
        condition: Box::new(condition),
        then_expr: Box::new(then_expr),
        else_expr: Box::new(else_expr),
        ty: then_ty,
    })
}

/// Lower a lambda or regular expression with contextual type information for parameters.
/// If the expression is a lambda, use `context_types` for untyped parameters.
/// If it's not a lambda, just lower it normally.
fn lower_lambda_with_context(expr: &Expr, context_types: &[Type], ctx: &mut LowerCtx) -> Option<HirExpr> {
    if let Expr::Lambda(lambda) = expr {
        ctx.scope.push();

        let mut params = Vec::new();
        if let Some(ref parameters) = lambda.parameters {
            for (i, param) in parameters.args.iter().enumerate() {
                let param_name = param.parameter.name.to_string();
                let param_ty = if let Some(ref ann) = param.parameter.annotation {
                    resolve_annotation_expr(ann, ctx)
                } else if i < context_types.len() {
                    // Use contextual type
                    context_types[i].clone()
                } else {
                    Type::Any
                };
                ctx.scope.define(param_name.clone(), param_ty.clone());
                params.push(HirParam {
                    name: param_name,
                    ty: param_ty,
                    default: None,
                    keyword_only: false,
                    convention: ParamConvention::default(),
                });
            }
        }

        let body = lower_expr(&lambda.body, ctx)?;
        let body_ty = body.ty().clone();

        ctx.scope.pop();

        let param_types: Vec<(String, Type)> = params.iter().map(|p| (p.name.clone(), p.ty.clone())).collect();
        let fn_ty = Type::Function(FunctionType::new(param_types, body_ty));

        Some(HirExpr::Lambda {
            params,
            body: Box::new(body),
            ty: fn_ty,
        })
    } else {
        // Not a lambda, lower normally
        lower_expr(expr, ctx)
    }
}

fn lower_lambda(lambda: &ExprLambda, ctx: &mut LowerCtx) -> Option<HirExpr> {
    ctx.scope.push();

    let mut params = Vec::new();
    if let Some(ref parameters) = lambda.parameters {
        for param in &parameters.args {
            let param_name = param.parameter.name.to_string();
            let param_ty = if let Some(ref ann) = param.parameter.annotation {
                resolve_annotation_expr(ann, ctx)
            } else {
                // Lambda params without annotations: infer as Any for now
                // Contextual typing will refine this at call sites
                Type::Any
            };
            ctx.scope.define(param_name.clone(), param_ty.clone());
            params.push(HirParam {
                name: param_name,
                ty: param_ty,
                default: None,
                keyword_only: false,
                convention: ParamConvention::default(),
            });
        }
    }

    let body = lower_expr(&lambda.body, ctx)?;
    let body_ty = body.ty().clone();

    ctx.scope.pop();

    // Build the function type for the lambda
    let param_types: Vec<(String, Type)> = params.iter().map(|p| (p.name.clone(), p.ty.clone())).collect();
    let fn_ty = Type::Function(FunctionType::new(param_types, body_ty));

    Some(HirExpr::Lambda {
        params,
        body: Box::new(body),
        ty: fn_ty,
    })
}

fn lower_list_comp(comp: &ExprListComp, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if comp.generators.is_empty() {
        ctx.error("list comprehension must have at least one generator".to_string());
        return None;
    }

    let mut generators = Vec::new();
    let num_gens = comp.generators.len();

    // Process each generator: push scope, define var, lower iter
    for gen in &comp.generators {
        let var_name = match &gen.target {
            Expr::Name(n) => n.id.to_string(),
            Expr::Tuple(tup) => {
                let names: Vec<String> = tup.elts.iter().filter_map(|e| {
                    if let Expr::Name(n) = e { Some(n.id.to_string()) } else { None }
                }).collect();
                if names.len() != tup.elts.len() {
                    ctx.error("comprehension tuple target must contain only simple names".to_string());
                    return None;
                }
                names.join(",")
            }
            _ => {
                ctx.error("comprehension target must be a simple name or tuple".to_string());
                return None;
            }
        };

        let iter_expr = lower_expr(&gen.iter, ctx)?;
        let iter_ty = iter_expr.ty().clone();
        let elem_ty = match &iter_ty {
            Type::List(elem) => *elem.clone(),
            Type::Set(elem) => *elem.clone(),
            Type::Str => Type::Str,
            Type::Range => Type::Int,
            Type::Dict(key, _) => *key.clone(),
            Type::Tuple(elems) if !elems.is_empty() => elems[0].clone(),
            _ => {
                ctx.error(format!("cannot iterate over type '{}'", iter_ty.display_name()));
                return None;
            }
        };

        ctx.scope.push();
        if var_name.contains(',') {
            let names: Vec<&str> = var_name.split(',').collect();
            if let Type::Tuple(elem_types) = &elem_ty {
                for (i, name) in names.iter().enumerate() {
                    let ty = elem_types.get(i).cloned().unwrap_or(Type::Any);
                    ctx.scope.define(name.to_string(), ty);
                }
            } else {
                for name in &names { ctx.scope.define(name.to_string(), Type::Any); }
            }
        } else {
            ctx.scope.define(var_name.clone(), elem_ty.clone());
        }

        let filter = if !gen.ifs.is_empty() {
            let first = lower_expr(&gen.ifs[0], ctx)?;
            if gen.ifs.len() == 1 {
                Some(first)
            } else {
                let mut combined = first;
                for cond in &gen.ifs[1..] {
                    let next = lower_expr(cond, ctx)?;
                    combined = HirExpr::BoolOp {
                        op: "and".to_string(),
                        values: vec![combined, next],
                        ty: Type::Bool,
                    };
                }
                Some(combined)
            }
        } else {
            None
        };

        generators.push((var_name, iter_expr, filter));
    }

    // Lower the expression (all generator vars are in scope)
    let expr = lower_expr(&comp.elt, ctx)?;
    let expr_ty = expr.ty().clone();

    // Pop all scopes
    for _ in 0..num_gens {
        ctx.scope.pop();
    }

    let result_ty = Type::List(Box::new(expr_ty));

    Some(HirExpr::ListComp {
        expr: Box::new(expr),
        generators,
        ty: result_ty,
    })
}

fn lower_set_comp(comp: &ExprSetComp, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let mut generators = Vec::new();
    let num_gens = comp.generators.len();
    for gen in &comp.generators {
        let var_name = match &gen.target {
            Expr::Name(n) => n.id.to_string(),
            _ => { ctx.error("set comprehension target must be a simple name".to_string()); return None; }
        };
        let iter_expr = lower_expr(&gen.iter, ctx)?;
        let iter_ty = iter_expr.ty().clone();
        let elem_ty = match &iter_ty {
            Type::List(elem) => *elem.clone(),
            Type::Set(elem) => *elem.clone(),
            Type::Range => Type::Int,
            _ => { ctx.error(format!("cannot iterate over type '{}'", iter_ty.display_name())); return None; }
        };
        ctx.scope.push();
        ctx.scope.define(var_name.clone(), elem_ty);
        let filter = if !gen.ifs.is_empty() { Some(lower_expr(&gen.ifs[0], ctx)?) } else { None };
        generators.push((var_name, iter_expr, filter));
    }
    let expr = lower_expr(&comp.elt, ctx)?;
    let expr_ty = expr.ty().clone();
    for _ in 0..num_gens { ctx.scope.pop(); }
    let result_ty = Type::Set(Box::new(expr_ty));
    Some(HirExpr::SetComp { expr: Box::new(expr), generators, ty: result_ty })
}

fn lower_dict_comp(comp: &ExprDictComp, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let mut generators = Vec::new();
    let num_gens = comp.generators.len();
    for gen in &comp.generators {
        let var_name = match &gen.target {
            Expr::Name(n) => n.id.to_string(),
            Expr::Tuple(tup) => {
                let names: Vec<String> = tup.elts.iter().filter_map(|e| {
                    if let Expr::Name(n) = e { Some(n.id.to_string()) } else { None }
                }).collect();
                names.join(",")
            }
            _ => { ctx.error("dict comprehension target must be a simple name or tuple".to_string()); return None; }
        };
        let iter_expr = lower_expr(&gen.iter, ctx)?;
        let iter_ty = iter_expr.ty().clone();
        let elem_ty = match &iter_ty {
            Type::List(elem) => *elem.clone(),
            Type::Set(elem) => *elem.clone(),
            Type::Range => Type::Int,
            Type::Dict(key, _) => *key.clone(),
            _ => { ctx.error(format!("cannot iterate over type '{}'", iter_ty.display_name())); return None; }
        };
        ctx.scope.push();
        if var_name.contains(',') {
            let names: Vec<&str> = var_name.split(',').collect();
            if let Type::Tuple(elem_types) = &elem_ty {
                for (i, name) in names.iter().enumerate() {
                    let ty = elem_types.get(i).cloned().unwrap_or(Type::Any);
                    ctx.scope.define(name.to_string(), ty);
                }
            } else {
                for name in &names { ctx.scope.define(name.to_string(), Type::Any); }
            }
        } else {
            ctx.scope.define(var_name.clone(), elem_ty);
        }
        let filter = if !gen.ifs.is_empty() { Some(lower_expr(&gen.ifs[0], ctx)?) } else { None };
        generators.push((var_name, iter_expr, filter));
    }
    let key_expr = lower_expr(&comp.key, ctx)?;
    let val_expr = lower_expr(&comp.value, ctx)?;
    let key_ty = key_expr.ty().clone();
    let val_ty = val_expr.ty().clone();
    for _ in 0..num_gens { ctx.scope.pop(); }
    let result_ty = Type::Dict(Box::new(key_ty), Box::new(val_ty));
    Some(HirExpr::DictComp {
        key_expr: Box::new(key_expr),
        val_expr: Box::new(val_expr),
        generators,
        ty: result_ty,
    })
}

fn lower_generator_expr(gen: &ExprGenerator, ctx: &mut LowerCtx) -> Option<HirExpr> {
    // Only support single generator: (expr for var in iter) or (expr for var in iter if cond)
    if gen.generators.len() != 1 {
        ctx.error("only single-generator generator expressions are supported".to_string());
        return None;
    }

    let comp = &gen.generators[0];

    // Get the variable name
    let var_name = match &comp.target {
        Expr::Name(n) => n.id.to_string(),
        _ => {
            ctx.error("generator target must be a simple name".to_string());
            return None;
        }
    };

    // Lower the iterable
    let iter_expr = lower_expr(&comp.iter, ctx)?;
    let iter_ty = iter_expr.ty().clone();

    // Determine element type from the iterable
    let elem_ty = match &iter_ty {
        Type::List(elem) => *elem.clone(),
        Type::Str => Type::Str,
        _ => {
            ctx.error(format!("cannot iterate over type '{}'", iter_ty.display_name()));
            return None;
        }
    };

    // Push scope and define the loop variable
    ctx.scope.push();
    ctx.scope.define(var_name.clone(), elem_ty.clone());

    // Lower the expression
    let expr = lower_expr(&gen.elt, ctx)?;
    let expr_ty = expr.ty().clone();

    // Lower the filter condition if present
    let filter = if !comp.ifs.is_empty() {
        let first = lower_expr(&comp.ifs[0], ctx)?;
        if comp.ifs.len() == 1 {
            Some(Box::new(first))
        } else {
            let mut combined = first;
            for cond in &comp.ifs[1..] {
                let next = lower_expr(cond, ctx)?;
                combined = HirExpr::BoolOp {
                    op: "and".to_string(),
                    values: vec![combined, next],
                    ty: Type::Bool,
                };
            }
            Some(Box::new(combined))
        }
    } else {
        None
    };

    ctx.scope.pop();

    let result_ty = Type::List(Box::new(expr_ty));

    Some(HirExpr::GeneratorExpr {
        expr: Box::new(expr),
        var: var_name,
        iter: Box::new(iter_expr),
        filter,
        ty: result_ty,
    })
}

fn lower_named_expr(named: &ExprNamed, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let name = match named.target.as_ref() {
        Expr::Name(n) => n.id.to_string(),
        _ => {
            ctx.error("walrus operator target must be a simple name".to_string());
            return None;
        }
    };

    let value = lower_expr(&named.value, ctx)?;
    let ty = value.ty().clone();

    // Define the variable in the current scope
    ctx.scope.define(name.clone(), ty.clone());

    Some(HirExpr::WalrusExpr {
        name,
        value: Box::new(value),
        ty,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sifr_python_parser::parse_module;

    fn lower_source(source: &str) -> Result<HirModule, Vec<LoweringError>> {
        let parsed = parse_module(source).expect("parse failed");
        lower_module(parsed.suite()).map(|r| r.module)
    }

    #[test]
    fn test_simple_function() {
        let module = lower_source(
            "def add(a: int, b: int) -> int:\n    return a + b\n"
        ).unwrap();
        assert_eq!(module.functions.len(), 1);
        assert_eq!(module.functions[0].name, "add");
        assert_eq!(module.functions[0].return_type, Type::Int);
    }

    #[test]
    fn test_type_mismatch_error() {
        let result = lower_source(
            "def main():\n    x: int = \"hello\"\n"
        );
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("type mismatch")));
    }

    #[test]
    fn test_undefined_variable() {
        let result = lower_source(
            "def main():\n    print(x)\n"
        );
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("undefined variable")));
    }

    #[test]
    fn test_use_after_move() {
        // Under borrow-by-default, consume() needs `own` to move the argument.
        // Without `own`, the argument is borrowed and no move error occurs.
        let result = lower_source(
            "def consume(own s: str) -> str:\n    return s\ndef main():\n    s: str = \"hello\"\n    x: str = consume(s)\n    print(s)\n"
        );
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("moved value")));
    }

    #[test]
    fn test_borrow_by_default_no_move() {
        // Under borrow-by-default, passing to a function that borrows does NOT move.
        let result = lower_source(
            "def process(s: str) -> int:\n    return len(s)\ndef main():\n    s: str = \"hello\"\n    x: int = process(s)\n    print(s)\n"
        );
        assert!(result.is_ok(), "borrow-by-default should not cause use-after-move");
    }

    #[test]
    fn test_copy_type_no_move() {
        let module = lower_source(
            "def main():\n    x: int = 42\n    print(x)\n    print(x)\n"
        ).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_while_loop() {
        let module = lower_source(
            "def main():\n    i: int = 0\n    while i < 10:\n        i = i + 1\n"
        ).unwrap();
        assert_eq!(module.functions.len(), 1);
        // Body should contain a Let and a While
        assert!(module.functions[0].body.len() >= 2);
        assert!(matches!(module.functions[0].body[1], HirStmt::While { .. }));
    }

    #[test]
    fn test_for_range() {
        let module = lower_source(
            "def main():\n    for i in range(10):\n        print(i)\n"
        ).unwrap();
        assert_eq!(module.functions.len(), 1);
        assert!(matches!(module.functions[0].body[0], HirStmt::For { .. }));
    }

    #[test]
    fn test_for_range_start_end() {
        let module = lower_source(
            "def main():\n    for i in range(1, 5):\n        print(i)\n"
        ).unwrap();
        assert_eq!(module.functions.len(), 1);
        assert!(matches!(module.functions[0].body[0], HirStmt::For { .. }));
    }

    #[test]
    fn test_break_outside_loop() {
        let result = lower_source(
            "def main():\n    break\n"
        );
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("'break' outside of loop")));
    }

    #[test]
    fn test_continue_outside_loop() {
        let result = lower_source(
            "def main():\n    continue\n"
        );
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("'continue' outside of loop")));
    }

    #[test]
    fn test_break_inside_loop() {
        let module = lower_source(
            "def main():\n    while True:\n        break\n"
        ).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_nested_loops() {
        let module = lower_source(
            "def main():\n    for i in range(3):\n        for j in range(2):\n            print(i)\n"
        ).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_fstring_basic() {
        let module = lower_source(
            "def main():\n    name: str = \"Alice\"\n    msg: str = f\"Hello, {name}!\"\n    print(msg)\n"
        ).unwrap();
        assert_eq!(module.functions.len(), 1);
        // Should have 3 statements: let name, let msg, print
        assert_eq!(module.functions[0].body.len(), 3);
    }

    #[test]
    fn test_fstring_with_expression() {
        let module = lower_source(
            "def main():\n    a: int = 2\n    b: int = 3\n    print(f\"{a} + {b} = {a + b}\")\n"
        ).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_tuple_unpack() {
        let module = lower_source(
            "def main():\n    pair: tuple[int, str] = (1, \"hello\")\n    x, y = pair\n    print(x)\n"
        ).unwrap();
        assert_eq!(module.functions.len(), 1);
        // Should have: let pair, tuple_unpack, print
        assert!(module.functions[0].body.len() >= 3);
        assert!(matches!(module.functions[0].body[1], HirStmt::TupleUnpack { .. }));
    }

    #[test]
    fn test_tuple_unpack_wrong_count() {
        let result = lower_source(
            "def main():\n    pair: tuple[int, str] = (1, \"hello\")\n    x, y, z = pair\n"
        );
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("expected 3 values, got 2")));
    }

    #[test]
    fn test_tuple_unpack_non_tuple() {
        let result = lower_source(
            "def main():\n    x: int = 42\n    a, b = x\n"
        );
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("cannot unpack non-tuple")));
    }
}
