//! AST to HIR lowering with type checking and name resolution.

use sifr_python_ast::*;
use sifr_type_system::{
    Type, FunctionType,
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
    /// Whether we're currently inside a class method (tracks `self` type)
    current_class: Option<String>,
    /// Whether we're inside a try block (auto-unwrap Result values)
    in_try_block: bool,
    /// Set of class names that are error types (class Foo(Error))
    error_types: std::collections::HashSet<String>,
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
            current_class: None,
            in_try_block: false,
            error_types: std::collections::HashSet::new(),
        }
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

/// Result of lowering, including the HIR module and any diagnostics.
pub struct LoweringResult {
    pub module: HirModule,
    /// reveal_type() diagnostics (informational, printed to stderr)
    pub reveal_types: Vec<String>,
}

/// External module definitions that can be imported.
#[derive(Debug, Clone, Default)]
pub struct ExternalDefs {
    /// Map of module_name -> (function_name -> FunctionType)
    pub functions: std::collections::HashMap<String, std::collections::HashMap<String, FunctionType>>,
    /// Map of module_name -> (class_name -> Type)
    pub classes: std::collections::HashMap<String, std::collections::HashMap<String, Type>>,
}

/// Lower a parsed module AST into a typed HIR module.
pub fn lower_module(stmts: &[Stmt]) -> Result<LoweringResult, Vec<LoweringError>> {
    lower_module_with_externals(stmts, &ExternalDefs::default())
}

/// Lower a parsed module AST into a typed HIR module, with external module definitions.
pub fn lower_module_with_externals(stmts: &[Stmt], externals: &ExternalDefs) -> Result<LoweringResult, Vec<LoweringError>> {
    let mut ctx = LowerCtx::new();

    // Register built-in functions
    register_builtins(&mut ctx);

    // First pass: collect all function signatures, type aliases, and class definitions
    for stmt in stmts {
        match stmt {
            Stmt::FunctionDef(func) => {
                if let Some(ft) = extract_function_type(func, &mut ctx) {
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
                }
            }
            // Handle `type X = ...` statement (Python 3.12 type alias)
            Stmt::TypeAlias(type_alias) => {
                let name = match type_alias.name.as_ref() {
                    Expr::Name(n) => n.id.to_string(),
                    _ => {
                        ctx.error("type alias name must be a simple name".to_string());
                        continue;
                    }
                };
                let ty = resolve_annotation_expr(&type_alias.value, &mut ctx);
                ctx.scope.define_type_alias(name, ty);
            }
            // First pass for classes: collect fields and method signatures
            Stmt::ClassDef(class_def) => {
                collect_class_type(class_def, &mut ctx);
            }
            _ => {}
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

                // Resolve imported names from external definitions
                for name in &names {
                    // Check if it's a private name
                    if name.starts_with('_') {
                        ctx.error(format!("cannot import private name '{}' from module '{}'", name, module_name));
                        continue;
                    }

                    // Look up in external functions
                    if let Some(module_fns) = externals.functions.get(&module_name) {
                        if let Some(ft) = module_fns.get(name) {
                            ctx.functions.insert(name.clone(), ft.clone());
                        }
                    }
                    // Look up in external classes
                    if let Some(module_classes) = externals.classes.get(&module_name) {
                        if let Some(class_ty) = module_classes.get(name) {
                            ctx.class_types.insert(name.clone(), class_ty.clone());
                            // Also register the constructor
                            if let Type::Class { fields, .. } = class_ty {
                                let params: Vec<(String, Type)> = fields.clone();
                                let ft = FunctionType {
                                    params,
                                    return_type: Box::new(class_ty.clone()),
                                };
                                ctx.functions.insert(name.clone(), ft);
                            }
                        }
                    }
                }

                imports.push(HirImport {
                    module: module_name,
                    names,
                });
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
            module: HirModule { functions, classes, imports },
            reveal_types: ctx.reveal_types,
        })
    } else {
        Err(ctx.errors)
    }
}

/// Check if a class definition has `(Error)` as its base class.
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

/// First pass: collect class fields and method signatures, register the class type.
fn collect_class_type(class_def: &StmtClassDef, ctx: &mut LowerCtx) {
    let class_name = class_def.name.to_string();
    let mut fields: Vec<(String, Type)> = Vec::new();
    let mut methods: Vec<(String, FunctionType)> = Vec::new();
    let is_error = is_error_class(class_def);

    // For error types, ensure a 'message' field exists (add if not explicitly declared)
    // This will be checked after collecting all fields

    // Register a preliminary class type so self-referential annotations work
    // (e.g., `def distance(self, other: Point)` inside class Point)
    ctx.class_types.insert(class_name.clone(), Type::Class {
        name: class_name.clone(),
        fields: vec![],
        methods: vec![],
    });

    for stmt in &class_def.body {
        match stmt {
            // Field annotations: `x: float`
            Stmt::AnnAssign(ann) => {
                if let Expr::Name(name) = ann.target.as_ref() {
                    let ty = resolve_annotation_expr(&ann.annotation, ctx);
                    fields.push((name.id.to_string(), ty));
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
                    let constructor_ft = FunctionType {
                        params: params.clone(),
                        return_type: Box::new(Type::None), // placeholder, updated below
                    };
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
                    // Regular method: extract params (skip `self`)
                    let mut params = Vec::new();
                    for param in func.parameters.args.iter().skip(1) {
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
                    methods.push((method_name, FunctionType {
                        params,
                        return_type: Box::new(return_ty),
                    }));
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
    };

    // Update the constructor function to return the class type
    if let Some(ft) = ctx.functions.get_mut(&class_name) {
        ft.return_type = Box::new(class_ty.clone());
    } else {
        // No __init__ defined -- create a default constructor from fields
        let params: Vec<(String, Type)> = fields.clone();
        let ft = FunctionType {
            params,
            return_type: Box::new(class_ty.clone()),
        };
        ctx.functions.insert(class_name.clone(), ft);
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

    let (fields, method_types) = match &class_ty {
        Type::Class { fields, methods, .. } => (fields.clone(), methods.clone()),
        _ => return None,
    };

    // Determine if all fields are hashable (primitives: int, float, bool, str)
    let is_hashable = fields.iter().all(|(_, ty)| is_hashable_type(ty));

    let mut hir_methods = Vec::new();

    for stmt in &class_def.body {
        if let Stmt::FunctionDef(func) = stmt {
            let method_name = func.name.to_string();

            // Set current class context for `self` resolution
            ctx.current_class = Some(class_name.clone());

            // Push a new scope for the method
            ctx.scope.push();

            // Define `self` in scope
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
            let method_ft = FunctionType {
                params: params.iter().map(|p| (p.name.clone(), p.ty.clone())).collect(),
                return_type: Box::new(return_ty.clone()),
            };

            // Lower method body
            let body = lower_stmts(&func.body, &method_ft, ctx);

            // Determine receiver mutability: if any statement assigns to self.field, it's &mut self
            let is_mutating = method_name == "__init__" || body_contains_field_assign(&body);

            ctx.scope.pop();
            ctx.current_class = None;

            hir_methods.push(HirFunction {
                name: if method_name == "__init__" { "new".to_string() } else { method_name },
                params,
                return_type: return_ty,
                body,
            });
        }
    }

    let is_error = ctx.error_types.contains(&class_name);

    Some(HirClass {
        name: class_name,
        fields,
        methods: hir_methods,
        is_hashable,
        is_error_type: is_error,
    })
}

/// Check if a type is hashable (can derive Hash + Eq).
fn is_hashable_type(ty: &Type) -> bool {
    match ty {
        Type::Int | Type::Bool | Type::Str | Type::None => true,
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
        FunctionType {
            params: vec![("value".to_string(), Type::Any)],
            return_type: Box::new(Type::None),
        },
    );
}

fn extract_function_type(func: &StmtFunctionDef, ctx: &mut LowerCtx) -> Option<FunctionType> {
    let mut params = Vec::new();

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
        params.push((name, ty));
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
        params.push((name, ty));
    }

    let return_type = if let Some(returns) = &func.returns {
        resolve_annotation_expr(returns, ctx)
    } else {
        Type::None // default return type
    };

    Some(FunctionType {
        params,
        return_type: Box::new(return_type),
    })
}

fn resolve_annotation_expr(expr: &Expr, ctx: &mut LowerCtx) -> Type {
    match expr {
        Expr::Name(name) => {
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
                _ => {
                    ctx.error(format!("unknown generic type: '{}'", base_name));
                    Type::Any
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
        let ty = ft.params.get(i).map(|(_, t)| t.clone()).unwrap_or(Type::Any);
        ctx.scope.define(name.clone(), ty.clone());

        let default = param_def.default.as_ref().and_then(|d| lower_expr(d, ctx));

        params.push(HirParam {
            name,
            ty,
            default,
            keyword_only: false,
        });
    }

    // Keyword-only args (after * separator)
    let regular_count = func.parameters.args.len();
    for (i, param_def) in func.parameters.kwonlyargs.iter().enumerate() {
        let name = param_def.parameter.name.to_string();
        let ty = ft.params.get(regular_count + i).map(|(_, t)| t.clone()).unwrap_or(Type::Any);
        ctx.scope.define(name.clone(), ty.clone());

        let default = param_def.default.as_ref().and_then(|d| lower_expr(d, ctx));

        params.push(HirParam {
            name,
            ty,
            default,
            keyword_only: true,
        });
    }

    // Lower body
    let body = lower_stmts(&func.body, &ft, ctx);

    ctx.scope.pop();

    Some(HirFunction {
        name: func.name.to_string(),
        params,
        return_type: *ft.return_type,
        body,
    })
}

fn lower_stmts(stmts: &[Stmt], func_type: &FunctionType, ctx: &mut LowerCtx) -> Vec<HirStmt> {
    let mut result = Vec::new();
    for stmt in stmts {
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
                let value = lower_expr(exc, ctx)?;
                Some(HirStmt::Raise { value })
            } else {
                ctx.error("bare 'raise' without an expression is not supported".to_string());
                None
            }
        }
        Stmt::Try(try_stmt) => {
            let prev_in_try = ctx.in_try_block;
            ctx.in_try_block = true;
            let body = lower_stmts(&try_stmt.body, func_type, ctx);
            ctx.in_try_block = prev_in_try;
            let mut handlers = Vec::new();
            for handler in &try_stmt.handlers {
                if let ExceptHandler::ExceptHandler(h) = handler {
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
                    // Define the error variable in scope if named
                    ctx.scope.push();
                    if let Some(ref var_name) = name {
                        // Determine the type of the error variable
                        let error_var_ty = if let Some(ref et) = error_type {
                            if let Some(class_ty) = ctx.class_types.get(et) {
                                class_ty.clone()
                            } else {
                                Type::Str // Default: error messages are strings
                            }
                        } else {
                            Type::Str
                        };
                        ctx.scope.define(var_name.clone(), error_var_ty);
                    }
                    let handler_body = lower_stmts(&h.body, func_type, ctx);
                    ctx.scope.pop();
                    // Resolve the error type for codegen
                    let error_resolved_type = error_type.as_ref().and_then(|et| {
                        if let Some(class_ty) = ctx.class_types.get(et) {
                            Some(class_ty.clone())
                        } else if et == "str" {
                            Some(Type::Str)
                        } else {
                            None
                        }
                    });
                    handlers.push(HirExceptHandler {
                        error_type,
                        error_resolved_type,
                        name,
                        body: handler_body,
                    });
                }
            }
            Some(HirStmt::TryExcept { body, handlers })
        }
        _ => {
            ctx.error("unsupported statement type".to_string());
            None
        }
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
            if let Type::Result(ref ok_ty, _) = expr_ty {
                if ok_ty.as_ref().is_assignable_to(&declared_type) {
                    expr = HirExpr::QuestionMark {
                        expr: Box::new(expr),
                        ty: declared_type.clone(),
                    };
                }
            }
        }
        // Type check: value must be assignable to declared type
        let final_ty = expr.ty().clone();
        if !final_ty.is_assignable_to(&declared_type) {
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

    ctx.scope.define(name.clone(), declared_type.clone());

    Some(HirStmt::Let {
        name,
        ty: declared_type,
        value,
        is_mutable: true,
    })
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
        _ => {
            ctx.error("unsupported augmented assignment operator".to_string());
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

    // Apply narrowing for then-branch (condition is true)
    if let Some(ref cond) = narrowing_cond {
        apply_narrowing(ctx, cond, true);
    }

    ctx.scope.push();
    let then_body = lower_stmts(&if_stmt.body, func_type, ctx);
    ctx.scope.pop();

    // Restore state before processing elif/else
    ctx.scope.restore_narrowing_state(&saved_state);

    let mut elif_clauses = Vec::new();
    for clause in &if_stmt.elif_else_clauses {
        if let Some(test) = &clause.test {
            // For elif, apply the negation of the original condition first
            if let Some(ref cond) = narrowing_cond {
                apply_narrowing(ctx, cond, false);
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

            ctx.scope.restore_narrowing_state(&elif_saved);
        }
    }

    // For else-branch, apply narrowing with condition = false
    let else_body = if_stmt.elif_else_clauses.iter().find(|c| c.test.is_none()).map(|clause| {
        if let Some(ref cond) = narrowing_cond {
            apply_narrowing(ctx, cond, false);
        }
        ctx.scope.push();
        let body = lower_stmts(&clause.body, func_type, ctx);
        ctx.scope.pop();
        body
    });

    // Restore original narrowing state after all branches
    ctx.scope.restore_narrowing_state(&saved_state);

    Some(HirStmt::If {
        condition,
        then_body,
        elif_clauses,
        else_body,
    })
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
    if let Some(var_name) = condition.var_name() {
        if let Some(info) = ctx.scope.lookup(var_name) {
            let current_ty = info.effective_type().clone();
            let narrowed = narrow_type(&current_ty, condition, is_true);
            ctx.scope.narrow_var(var_name, narrowed);
        }
    }
}

fn lower_while(while_stmt: &StmtWhile, func_type: &FunctionType, ctx: &mut LowerCtx) -> Option<HirStmt> {
    let condition = lower_expr(&while_stmt.test, ctx)?;

    ctx.scope.push();
    ctx.loop_depth += 1;
    let body = lower_stmts(&while_stmt.body, func_type, ctx);
    ctx.loop_depth -= 1;
    ctx.scope.pop();

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

    // Extract the target variable name
    let target_name = match for_stmt.target.as_ref() {
        Expr::Name(n) => n.id.to_string(),
        _ => {
            ctx.error("for loop target must be a simple name".to_string());
            return None;
        }
    };

    // Create a new scope for the loop body, define the loop variable
    ctx.scope.push();
    ctx.scope.define(target_name.clone(), elem_ty.clone());
    ctx.loop_depth += 1;
    let body = lower_stmts(&for_stmt.body, func_type, ctx);
    ctx.loop_depth -= 1;
    ctx.scope.pop();

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
        Expr::Dict(dict) => lower_dict_literal(dict, ctx),
        Expr::Tuple(tuple) => lower_tuple_literal(tuple, ctx),
        Expr::Subscript(sub) => lower_subscript(sub, ctx),
        Expr::Attribute(attr) => lower_attribute(attr, ctx),
        Expr::FString(fstring) => lower_fstring(fstring, ctx),
        Expr::Named(named) => lower_named_expr(named, ctx),
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
        _ => {
            ctx.error(format!("unsupported binary operator"));
            return None;
        }
    };

    match type_check_binary_op(left.ty(), op_str, right.ty()) {
        Ok(result_ty) => Some(HirExpr::BinOp {
            left: Box::new(left),
            op: op_str.to_string(),
            right: Box::new(right),
            ty: result_ty,
        }),
        Err(e) => {
            ctx.error(e.message);
            None
        }
    }
}

fn lower_unaryop(unary: &ExprUnaryOp, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let operand = lower_expr(&unary.operand, ctx)?;

    let op_str = match unary.op {
        UnaryOp::USub => "-",
        UnaryOp::UAdd => "+",
        UnaryOp::Not => "not",
        _ => {
            ctx.error("unsupported unary operator".to_string());
            return None;
        }
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
                ctx.error(e.message);
                return None;
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
        // int(str) -> Result[int, str] (fallible)
        // int(float) -> int (infallible truncation)
        // int(int) -> int (identity)
        // int(bool) -> int (True=1, False=0)
        let result_ty = if arg_ty == Type::Str {
            Type::Result(Box::new(Type::Int), Box::new(Type::Str))
        } else {
            Type::Int
        };
        return Some(HirExpr::Call {
            func: "int".to_string(),
            args: vec![arg],
            ty: result_ty,
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
        // float(str) -> Result[float, str] (fallible)
        // float(int) -> float (infallible widening)
        // float(float) -> float (identity)
        let result_ty = if arg_ty == Type::Str {
            Type::Result(Box::new(Type::Float), Box::new(Type::Str))
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
        if positional_args.len() > ft.params.len() {
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
        for (i, (param_name, _param_ty)) in ft.params.iter().enumerate() {
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
            if !ft.params.iter().any(|(p, _)| p == kw_name) {
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
        for (i, (arg, (param_name, param_ty))) in args.iter().zip(ft.params.iter()).enumerate() {
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

    // Track ownership: move arguments of move types
    for arg in &args {
        if let HirExpr::Name { name, ty } = arg {
            if ty.ownership() == sifr_type_system::OwnershipKind::Move {
                ctx.scope.mark_moved(name);
            }
        }
    }

    // If this is a class constructor call, emit ConstructorCall
    if ctx.class_types.contains_key(&func_name) {
        Some(HirExpr::ConstructorCall {
            class_name: func_name,
            args,
            ty: *ft.return_type,
        })
    } else {
        Some(HirExpr::Call {
            func: func_name,
            args,
            ty: *ft.return_type,
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
    let object = lower_expr(&attr.value, ctx)?;
    let object_ty = object.ty().clone();
    let field_name = attr.attr.to_string();

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

    // Not a class field access -- report unsupported
    ctx.error(format!("attribute access '.{}' is not supported as an expression; use as a method call", field_name));
    None
}

fn lower_method_call(attr: &ExprAttribute, call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
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
                if args.len() != 1 {
                    ctx.error(format!("dict.get() takes exactly 1 argument, got {}", args.len()));
                    return None;
                }
                // get() returns Option[V] = V | None
                Some(Type::Union(vec![*val_ty.clone(), Type::None]))
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
        Type::Class { name, methods, .. } => {
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
                for (i, (arg, (param_name, param_ty))) in args.iter().zip(ft.params.iter()).enumerate() {
                    if !arg.ty().is_assignable_to(param_ty) {
                        ctx.error(format!(
                            "argument {} ('{}') of {}.{}(): expected '{}', got '{}'",
                            i + 1, param_name, name, method,
                            param_ty.display_name(), arg.ty().display_name()
                        ));
                    }
                }
                Some(*ft.return_type.clone())
            } else {
                ctx.error(format!("class '{}' has no method '{}'", name, method));
                None
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

    // len() works on str, list, dict, tuple
    match &arg_ty {
        Type::Str | Type::List(_) | Type::Dict(_, _) | Type::Tuple(_) => {
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
                ty: Type::Range,
            })
        }
        _ => {
            ctx.error(format!(
                "range() takes 1 or 2 arguments, got {}",
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
        let result = lower_source(
            "def main():\n    s: str = \"hello\"\n    print(s)\n    print(s)\n"
        );
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("moved value")));
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
