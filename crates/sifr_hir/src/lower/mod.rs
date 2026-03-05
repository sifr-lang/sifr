//! AST to HIR lowering with type checking and name resolution.

use crate::hir_nodes::{
    HirClass, HirClassKind, HirExceptHandler, HirExpr, HirFStringPart, HirFunction, HirImport,
    HirMatchArm, HirModule, HirParam, HirPattern, HirStmt, MethodKind,
};
use crate::scope::Scope;
use sifr_python_ast::{
    AstParamConvention, BoolOp, CmpOp, ExceptHandler, Expr, ExprAttribute, ExprBinOp, ExprBoolOp,
    ExprCall, ExprCompare, ExprDict, ExprDictComp, ExprFString, ExprGenerator, ExprIf, ExprLambda,
    ExprList, ExprListComp, ExprName, ExprNamed, ExprNumberLiteral, ExprSet, ExprSetComp,
    ExprSubscript, ExprTuple, ExprUnaryOp, FStringElement, Number, Operator, Pattern, Singleton,
    Stmt, StmtAnnAssign, StmtAssign, StmtAugAssign, StmtClassDef, StmtFor, StmtFunctionDef, StmtIf,
    StmtMatch, StmtReturn, StmtWhile, UnaryOp,
};
use sifr_type_system::infer::resolve_type_annotation;
use sifr_type_system::{
    make_union, narrow_type, type_check_binary_op, type_check_bool_op, type_check_comparison,
    type_check_unary_op, FunctionType, NarrowingCondition, OwnershipKind, ParamConvention, Type,
};
use std::collections::HashMap;

mod classes;
mod diagnostics;
mod expressions;
mod imports;
mod statements;
mod typing_and_functions;

use classes::*;
use diagnostics::*;
use expressions::*;
use imports::*;
use statements::*;
use typing_and_functions::*;

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
    /// Default parameter values for functions (name -> vec of (`param_index`, `default_expr`))
    function_defaults: HashMap<String, Vec<(usize, HirExpr)>>,
    /// Class type definitions (name -> `Type::Class`)
    class_types: HashMap<String, Type>,
    /// Current scope for name resolution
    scope: Scope,
    /// Collected errors
    errors: Vec<LoweringError>,
    /// Loop nesting depth (for break/continue validation)
    loop_depth: usize,
    /// `reveal_type()` diagnostics (informational, not errors)
    reveal_types: Vec<String>,
    /// Compiler warnings (non-fatal diagnostics printed to stderr)
    warnings: Vec<String>,
    /// Whether we're currently inside a class method (tracks `self` type)
    current_class: Option<String>,
    /// The parent class name of the current class (for `super()` resolution)
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
    /// Set of registered type variable names (e.g., T, K, V from `TypeVar` declarations)
    type_vars: std::collections::HashSet<String>,
    /// Map of generic function names to their type variable names
    generic_functions: HashMap<String, Vec<String>>,
    /// Map of owner (function or class name) -> (`type_var_name` -> protocol bounds)
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

/// Collect all `TypeVar` names used in a type.
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
        Type::TypeVar(name) => bindings.get(name).cloned().unwrap_or_else(|| ty.clone()),
        Type::List(elem) => Type::List(Box::new(substitute_type_vars(elem, bindings))),
        Type::Set(elem) => Type::Set(Box::new(substitute_type_vars(elem, bindings))),
        Type::Dict(k, v) => Type::Dict(
            Box::new(substitute_type_vars(k, bindings)),
            Box::new(substitute_type_vars(v, bindings)),
        ),
        Type::Tuple(elems) => Type::Tuple(
            elems
                .iter()
                .map(|e| substitute_type_vars(e, bindings))
                .collect(),
        ),
        Type::Union(members) => make_union(
            members
                .iter()
                .map(|m| substitute_type_vars(m, bindings))
                .collect(),
        ),
        Type::Callable(params, conventions, ret) => Type::Callable(
            params
                .iter()
                .map(|p| substitute_type_vars(p, bindings))
                .collect(),
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
        "Addable" => matches!(ty, Type::Int | Type::Float | Type::Str | Type::BigInt),
        "Hashable" => matches!(
            ty,
            Type::Int
                | Type::Str
                | Type::Bool
                | Type::BigInt
                | Type::None
                | Type::Enum { .. }
                | Type::LiteralStr(_)
                | Type::LiteralInt(_)
                | Type::LiteralBool(_)
        ),
        _ => true,
    }
}

/// Result of lowering, including the HIR module and any diagnostics.
pub struct LoweringResult {
    pub module: HirModule,
    /// `reveal_type()` diagnostics (informational, printed to stderr)
    pub reveal_types: Vec<String>,
    /// Compiler warnings (non-fatal, printed to stderr)
    pub warnings: Vec<String>,
}

/// External module definitions that can be imported.
#[derive(Debug, Clone, Default)]
pub struct ExternalDefs {
    /// Map of `module_name` -> (`function_name` -> `FunctionType`)
    pub functions:
        std::collections::HashMap<String, std::collections::HashMap<String, FunctionType>>,
    /// Map of `module_name` -> (`class_name` -> Type)
    pub classes: std::collections::HashMap<String, std::collections::HashMap<String, Type>>,
    /// Map of `module_name` -> (`constant_name` -> Type)
    pub constants: std::collections::HashMap<String, std::collections::HashMap<String, Type>>,
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
pub fn lower_module_stdlib_with_externals(
    stmts: &[Stmt],
    externals: &ExternalDefs,
) -> Result<LoweringResult, Vec<LoweringError>> {
    let mut ctx = LowerCtx::new();
    ctx.allow_intrinsic_imports = true;
    lower_module_impl(stmts, externals, ctx)
}

/// Lower a parsed module AST into a typed HIR module, with external module definitions.
pub fn lower_module_with_externals(
    stmts: &[Stmt],
    externals: &ExternalDefs,
) -> Result<LoweringResult, Vec<LoweringError>> {
    let ctx = LowerCtx::new();
    lower_module_impl(stmts, externals, ctx)
}

/// Internal implementation of module lowering.
fn lower_module_impl(
    stmts: &[Stmt],
    externals: &ExternalDefs,
    mut ctx: LowerCtx,
) -> Result<LoweringResult, Vec<LoweringError>> {
    // Register built-in functions
    register_builtins(&mut ctx);

    // Pass 0: Pre-register all class names as forward-reference placeholders.
    // This allows function signatures and other classes to reference classes
    // defined later in the file (e.g., ListNode, TreeNode, Node).
    for stmt in stmts {
        if let Stmt::ClassDef(class_def) = stmt {
            let class_name = class_def.name.to_string();
            if !ctx.class_types.contains_key(&class_name) {
                ctx.class_types.insert(
                    class_name.clone(),
                    Type::Class {
                        name: class_name,
                        fields: Vec::new(),
                        methods: Vec::new(),
                        parent_class: None,
                    },
                );
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
                                ctx.type_vars.insert(name.id.clone());
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
        if let Stmt::TypeAlias(type_alias) = stmt {
            let name = if let Expr::Name(n) = type_alias.name.as_ref() {
                n.id.clone()
            } else {
                ctx.error("type alias name must be a simple name".to_string());
                continue;
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
                ctx.scope
                    .define_generic_type_alias(name, alias_type_params.clone(), ty);
            }
            for tp_name in &alias_type_params {
                ctx.type_vars.remove(tp_name.as_str());
            }
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
                                Expr::Name(n) => n.id.clone(),
                                _ => continue,
                            };
                            ctx.type_param_bounds
                                .entry(func.name.to_string())
                                .or_default()
                                .entry(name)
                                .or_default()
                                .push(bound_name);
                        }
                    }
                }
            }

            let ft = extract_function_type(func, &mut ctx);
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
                ctx.generic_functions
                    .insert(func.name.to_string(), func_type_vars);
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
                ctx.function_defaults
                    .insert(func.name.to_string(), defaults);
            }
            ctx.functions.insert(func.name.to_string(), ft);
            // Track vararg functions
            if func.parameters.vararg.is_some() {
                ctx.vararg_functions.insert(func.name.to_string());
            }
        }
    }

    // Collect import statements and resolve imported names
    let mut imports = Vec::new();
    for stmt in stmts {
        if let Stmt::ImportFrom(import_from) = stmt {
            if import_from.level > 1 {
                let module_name = import_from
                    .module
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "<none>".to_string());
                ctx.error(format!(
                    "unsupported relative import level {} for module '{module_name}'",
                    import_from.level
                ));
                continue;
            }
            let Some(ref module) = import_from.module else {
                ctx.error("unsupported bare relative import; use 'from <module> import ...'".to_string());
                continue;
            };
            let module_name = module.to_string();
            let is_absolute_import = import_from.level == 0;
                let names: Vec<String> = import_from
                    .names
                    .iter()
                    .map(|alias| alias.name.to_string())
                    .collect();
                // Collect aliases: (original_name, local_alias)
                let aliases: Vec<(String, String)> = import_from
                    .names
                    .iter()
                    .filter_map(|alias| {
                        alias
                            .asname
                            .as_ref()
                            .map(|asname| (alias.name.to_string(), asname.to_string()))
                    })
                    .collect();

                // Build a mapping from original name -> local name (alias or original)
                let local_name_for = |original: &str| -> String {
                    aliases
                        .iter()
                        .find(|(orig, _)| orig == original)
                        .map(|(_, alias)| alias.clone())
                        .unwrap_or_else(|| original.to_string())
                };

                // Skip typing imports (TypeVar, Callable, etc.) - they are handled at the type level
                if is_absolute_import && module_name == "typing" {
                    continue;
                }

                // Skip enum imports (Enum is a built-in base class in Sifr)
                if is_absolute_import && module_name == "enum" {
                    continue;
                }

                // Block user imports of _sifr.* (internal intrinsics)
                // Stdlib .sifr files are allowed to import from _sifr.*
                if is_absolute_import && module_name.starts_with("_sifr.") {
                    if !ctx.allow_intrinsic_imports {
                        ctx.error(format!("cannot import from '{module_name}' — _sifr.* modules are internal compiler intrinsics"));
                        continue;
                    }
                    // Resolve intrinsic imports for stdlib .sifr files
                    if let Some(intrinsic_module) =
                        crate::stdlib::get_intrinsic_module(&module_name)
                    {
                        for name in &names {
                            let local = local_name_for(name);
                            if let Some(ft) = intrinsic_module.functions.get(name) {
                                ctx.functions.insert(local, ft.clone());
                            } else if let Some(const_ty) = intrinsic_module.constants.get(name) {
                                ctx.scope.define(local, const_ty.clone());
                            } else {
                                ctx.error(format!(
                                    "intrinsic module '{module_name}' has no member '{name}'"
                                ));
                            }
                        }
                        imports.push(HirImport {
                            module: module_name,
                            names,
                            aliases,
                        });
                        continue;
                    }
                    ctx.error(format!("unknown intrinsic module '{module_name}'"));
                    continue;
                }

                // Check if this is a stdlib import (sifr.*)
                // All sifr.* modules are now .sifr files compiled in the stdlib phase.
                // Resolve from pre-compiled stdlib modules (via externals).
                if is_absolute_import && module_name.starts_with("sifr.") {
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
                                    if let Some(module_gf) =
                                        externals.generic_functions.get(&stdlib_module_key)
                                    {
                                        if let Some(type_vars) = module_gf.get(name) {
                                            ctx.generic_functions
                                                .insert(local.clone(), type_vars.clone());
                                        }
                                    }
                                    if let Some(module_bounds) =
                                        externals.type_param_bounds.get(&stdlib_module_key)
                                    {
                                        if let Some(owner_bounds) = module_bounds.get(name) {
                                            ctx.type_param_bounds
                                                .insert(local.clone(), owner_bounds.clone());
                                        }
                                    }
                                }
                            }
                            // Check classes
                            if !found {
                                if let Some(module_classes) =
                                    externals.classes.get(&stdlib_module_key)
                                {
                                    if let Some(class_ty) = module_classes.get(name) {
                                        ctx.class_types.insert(local.clone(), class_ty.clone());
                                        // Register as error type if flagged in external defs
                                        if externals.error_types.contains(name) {
                                            ctx.error_types.insert(local.clone());
                                        }
                                        // Register constructor: prefer `new` method params if available
                                        if let Type::Class {
                                            fields, methods, ..
                                        } = class_ty
                                        {
                                            let ft = if let Some((_, new_ft)) =
                                                methods.iter().find(|(n, _)| n == "new")
                                            {
                                                let params: Vec<(String, Type)> = new_ft
                                                    .params
                                                    .iter()
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
                                        if let Some(module_bounds) =
                                            externals.type_param_bounds.get(&stdlib_module_key)
                                        {
                                            if let Some(owner_bounds) = module_bounds.get(name) {
                                                ctx.type_param_bounds
                                                    .insert(local.clone(), owner_bounds.clone());
                                            }
                                        }
                                        found = true;
                                    }
                                }
                            }
                            // Check constants
                            if !found {
                                if let Some(module_consts) =
                                    externals.constants.get(&stdlib_module_key)
                                {
                                    if let Some(const_ty) = module_consts.get(name) {
                                        ctx.scope.define(local, const_ty.clone());
                                        found = true;
                                    }
                                }
                            }
                            if !found {
                                ctx.error(format!("module '{module_name}' has no member '{name}'"));
                            }
                        }
                        imports.push(HirImport {
                            module: module_name,
                            names,
                            aliases,
                        });
                        continue;
                    }
                    // Module doesn't exist in stdlib — emit clear error at the import site
                    ctx.error(format!("unknown stdlib module '{module_name}'"));
                    continue;
                }

                // Check if the local module exists in externals before resolving
                let has_local_module = externals.functions.contains_key(&module_name)
                    || externals.classes.contains_key(&module_name)
                    || externals.constants.contains_key(&module_name);
                if !has_local_module {
                    ctx.error(format!("unknown module '{module_name}'"));
                    continue;
                }

                // Resolve imported names from external definitions (local modules)
                for name in &names {
                    let local = local_name_for(name);
                    // Check if it's a private name
                    if name.starts_with('_') {
                        ctx.error(format!(
                            "cannot import private name '{name}' from module '{module_name}'"
                        ));
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
                                if let Type::Class {
                                    fields, methods, ..
                                } = class_ty
                                {
                                    let ft = if let Some((_, new_ft)) =
                                        methods.iter().find(|(n, _)| n == "new")
                                    {
                                        // Use the actual __init__ parameters
                                        let params: Vec<(String, Type)> = new_ft
                                            .params
                                            .iter()
                                            .map(|(n, t, _)| (n.clone(), t.clone()))
                                            .collect();
                                        FunctionType::new(params, class_ty.clone())
                                    } else {
                                        // No __init__ — default constructor from fields
                                        let params: Vec<(String, Type)> = fields.clone();
                                        FunctionType::new(params, class_ty.clone())
                                    };
                                    ctx.functions.insert(local.clone(), ft);
                                }
                                found = true;
                            }
                        }
                    }
                    // Look up in external constants
                    if !found {
                        if let Some(module_consts) = externals.constants.get(&module_name) {
                            if let Some(const_ty) = module_consts.get(name) {
                                ctx.scope.define(local.clone(), const_ty.clone());
                                found = true;
                            }
                        }
                    }
                    if !found {
                        ctx.error(format!("module '{module_name}' has no member '{name}'"));
                    }
                }

                imports.push(HirImport {
                    module: module_name,
                    names,
                    aliases,
                });
        } else if let Stmt::Import(import_stmt) = stmt {
            for alias in &import_stmt.names {
                let module_name = alias.name.to_string();
                ctx.error(format!(
                    "unsupported import statement 'import {module_name}'; use 'from {module_name} import <name>'"
                ));
            }
        }
    }

    // Collect module-level constants (annotated assignments at top level)
    let mut constants = Vec::new();
    for stmt in stmts {
        if let Stmt::AnnAssign(ann) = stmt {
            if let Expr::Name(name) = ann.target.as_ref() {
                let var_name = name.id.clone();
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
                    let var_name = name.id.clone();
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
