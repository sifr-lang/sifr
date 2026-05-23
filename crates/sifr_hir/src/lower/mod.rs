use crate::hir_nodes::{HirExpr, HirImport, HirModule};
use crate::scope::{ErrorTaint, Scope};
use sifr_python_ast::{Expr, Stmt};
use sifr_type_system::{make_union, FunctionType, Type};
use std::collections::HashMap;
mod append_growth_shapes;
mod arithmetic_warnings;
mod assignment_widening;
mod async_await;
mod async_comprehension_diagnostics;
mod async_comprehensions;
mod async_effects;
mod async_for;
mod async_generator_advances;
mod async_generator_methods;
mod async_with;
mod asyncio_run_entrypoint;
mod attribute_access;
mod aug_assign_lowering;
mod binding_mutability;
mod blocking_executor_calls;
mod builtin_calls;
mod bytes_methods;
mod call_argument_ranges;
mod class_field_inference;
mod classes;
mod compat_imports;
mod container_literal_diagnostics;
mod container_literal_specialization;
mod control_flow_conditions;
mod decimal_methods;
mod default_args;
mod defaultdict_refinement;
mod diagnostic_types;
mod diagnostics;
mod empty_collection_refinement;
mod expression_abs;
mod expression_diagnostics;
mod expression_functional_builtins;
mod expression_iter_builtins;
mod expression_operators;
mod expression_sum_sorted;
mod expressions;
#[cfg(test)]
mod expressions_tests;
mod external_defs;
mod fixed_width_arithmetic_methods;
mod fixed_width_class_payload;
mod fixed_width_fitting;
mod flow_diagnostics;
mod flow_helpers;
mod for_loop_safety;
mod fstring_support;
mod function_flow;
mod function_scopes;
mod generic_constructor_specialization;
mod generic_inference;
mod generic_receiver_specialization;
mod guarded_index;
mod if_branch_bindings;
mod if_expression;
mod import_diagnostics;
mod import_resolution;
mod imported_defaults;
mod imports;
mod integer_const_facts;
mod integer_failure_diagnostics;
mod integer_literal_diagnostics;
mod integer_literals;
mod integer_nonzero_guards;
mod len_aliases;
mod match_diagnostics;
mod match_lowering;
mod method_call_args;
mod method_diagnostics;
mod min_max_validation;
mod module_constants_lowering;
mod module_function_registry;
mod mutating_methods;
mod name_diagnostics;
#[cfg(test)]
mod name_import_diagnostics_tests;
mod narrowing;
mod nested_function_inference;
#[cfg(test)]
mod nested_function_tests;
mod nonempty_method_narrowing;
mod nonlocal_support;
mod numeric_sentinels;
#[cfg(test)]
mod own_mut_param_tests;
#[cfg(test)]
mod own_mut_semantics_tests;
mod ownership_diagnostics;
mod protocol_diagnostics;
mod result_diagnostics;
#[cfg(test)]
mod result_diagnostics_tests;
mod return_lowering;
mod scope_helpers;
mod sequence_guard_detection;
mod sequence_guard_updates;
mod sequence_guards;
mod sequence_pointers;
mod sequence_shapes;
mod simple_expr;
mod statement_diagnostics;
#[cfg(test)]
mod statement_diagnostics_tests;
mod statements;
mod subscript_type;
mod task_calls;
mod task_handle_calls;
mod task_scope_calls;
mod tuple_unpack;
#[cfg(test)]
mod type_alias_tests;
mod type_aliases;
mod type_bounds;
mod type_var_collection;
mod typevar_annotations;
mod typevar_shape_compat;
mod typing_and_functions;
mod warning_helpers;
mod workload_annotations;
use async_effects::AsyncSuspensionSummary;
use asyncio_run_entrypoint::function_uses_asyncio_run_entrypoint;
use classes::{collect_class_type, lower_class};
use default_args::collect_function_defaults;
pub use diagnostic_types::{HirDiagnostic, LoweringWarningDiagnostic, RevealTypeDiagnostic};
pub use external_defs::ExternalDefs;
use generic_inference::infer_type_var_bindings;
use imports::resolve_imports_early;
use len_aliases::LenAliasFact;
use ruff_text_size::{Ranged, TextRange};
use sequence_guards::SequenceGuard;
use sequence_pointers::SequencePointerFact;
use sifr_diagnostics::DiagnosticCode;
use type_aliases::{collect_type_alias_decls, predeclare_type_aliases, resolve_type_aliases};
use type_var_collection::collect_type_vars;
pub(super) use typevar_annotations::{
    decode_typevar_constraint, encode_typevar_constraint, parse_typevar_bound_expr,
    parse_typevar_declaration_specs,
};
use typing_and_functions::{
    extract_function_type, function_body_contains_yield, lower_function, register_builtins,
};
use workload_annotations::WorkloadKind;
/// The lowering context that tracks state during AST->HIR conversion.
pub(super) struct LowerCtx {
    /// Function signatures (name -> type)
    functions: HashMap<String, FunctionType>,
    async_functions: std::collections::HashSet<String>,
    async_generator_functions: std::collections::HashSet<String>,
    async_suspension_summaries: HashMap<String, AsyncSuspensionSummary>,
    /// Workload classification decorators for user-defined functions.
    function_workload_annotations: HashMap<String, WorkloadKind>,
    /// Default parameter values for functions (name -> vec of (`param_index`, `default_expr`))
    function_defaults: HashMap<String, Vec<(usize, HirExpr)>>,
    /// Class type definitions (name -> `Type::Class`)
    class_types: HashMap<String, Type>,
    /// Current scope for name resolution
    scope: Scope,
    /// First non-Never child error type observed for each in-scope `TaskGroup` binding.
    task_group_error_types: HashMap<String, Type>,
    /// In-scope task handle binding -> owning `TaskGroup` binding.
    task_handle_group_owners: HashMap<String, String>,
    /// `TaskGroup` bindings that are no longer proven Open after observing a child handle.
    task_groups_not_proven_open: std::collections::HashSet<String>,
    /// Collected diagnostics that stop successful lowering.
    errors: Vec<HirDiagnostic>,
    /// Proof for the latest emitted lowering diagnostic.
    last_error_taint: Option<ErrorTaint>,
    /// Loop nesting depth (for break/continue validation)
    loop_depth: usize,
    /// `reveal_type()` diagnostics (informational, not errors)
    reveal_types: Vec<RevealTypeDiagnostic>,
    /// Compiler warnings (non-fatal diagnostics)
    warnings: Vec<LoweringWarningDiagnostic>,
    /// Whether we're currently inside a class method (tracks `self` type)
    current_class: Option<String>,
    /// Current function/method owner name while lowering a body.
    current_owner: Option<String>,
    /// The parent class name of the current class (for `super()` resolution)
    current_parent_class: Option<String>,
    /// Whether we're inside a try block (auto-unwrap Result values)
    in_try_block: bool,
    /// Whether the currently lowered function body is async.
    current_function_is_async: bool,
    /// Whether the currently lowered function body is an `async def` containing `yield`.
    current_function_is_async_generator: bool,
    /// Return type of the currently lowered function body.
    current_function_return_type: Option<Type>,
    /// Error types collected from Result-returning calls during try body lowering.
    /// Each entry is the name of an error class encountered via auto-unwrap in the current try block.
    try_block_error_types: std::collections::HashSet<String>,
    /// Set of class names that are error types (class Foo(Error))
    error_types: std::collections::HashSet<String>,
    /// Map of parent error type -> list of known child error types (for exhaustiveness checking)
    error_hierarchy: HashMap<String, Vec<String>>,
    /// Map of function names to the parameter index of their *args (vararg) parameter
    vararg_functions: HashMap<String, usize>,
    /// Set of registered type variable names (e.g., T, K, V from `TypeVar` declarations)
    type_vars: std::collections::HashSet<String>,
    /// Map of generic function names to their type variable names
    generic_functions: HashMap<String, Vec<String>>,
    /// Map of owner (function or class name) -> (`type_var_name` -> protocol bounds)
    type_param_bounds: HashMap<String, HashMap<String, Vec<String>>>,
    /// Global `TypeVar(...)` declaration bounds/constraints by declared type variable name.
    /// Constraints are encoded with `TYPEVAR_CONSTRAINT_PREFIX`.
    declared_type_var_bounds: HashMap<String, Vec<String>>,
    /// Whether _sifr.* intrinsic imports are allowed (true for stdlib .sifr files)
    allow_intrinsic_imports: bool,
    /// Set of parameter names that are immutably borrowed (&T) in the current function.
    borrowed_params: std::collections::HashSet<String>,
    /// Map of class names to their declared type parameters (from PEP 695 class C[T])
    class_declared_type_params: HashMap<String, Vec<String>>,
    current_module_name: Option<String>,
    externals: ExternalDefs,
    asyncio_compat_imports: HashMap<String, String>,
    synthetic_imports: Vec<HirImport>,
    synthetic_import_aliases: HashMap<String, String>,
    sequence_guards: Vec<SequenceGuard>,
    len_aliases: Vec<LenAliasFact>,
    sequence_pointers: Vec<SequencePointerFact>,
    numeric_sentinel_vars: HashMap<String, numeric_sentinels::NumericSentinelFact>,
    pending_numeric_sentinel_patches: HashMap<String, numeric_sentinels::NumericSentinelPatch>,
    pending_container_specialization_patches: HashMap<String, Type>,
    async_generator_advances: async_generator_advances::AsyncGeneratorAdvanceTracker,
    sequence_shapes: Vec<sequence_shapes::SequenceShapeFact>,
    proven_nonzero_integer_bindings: std::collections::HashSet<String>,
    function_scopes: Vec<function_scopes::FunctionScopeState>,
    inferred_binding_hints: Vec<HashMap<String, Type>>,
    empty_collection_hint_adoption: Vec<bool>,
    empty_dict_specializations: HashMap<String, Type>,
    const_integer_values: HashMap<String, num_bigint::BigInt>,
}

impl LowerCtx {
    fn new() -> Self {
        Self {
            functions: HashMap::new(),
            async_functions: std::collections::HashSet::new(),
            async_generator_functions: std::collections::HashSet::new(),
            async_suspension_summaries: HashMap::new(),
            function_workload_annotations: HashMap::new(),
            function_defaults: HashMap::new(),
            class_types: HashMap::new(),
            scope: Scope::new(),
            task_group_error_types: HashMap::new(),
            task_handle_group_owners: HashMap::new(),
            task_groups_not_proven_open: std::collections::HashSet::new(),
            errors: Vec::new(),
            last_error_taint: None,
            loop_depth: 0,
            reveal_types: Vec::new(),
            warnings: Vec::new(),
            current_class: None,
            current_owner: None,
            current_parent_class: None,
            in_try_block: false,
            current_function_is_async: false,
            current_function_is_async_generator: false,
            current_function_return_type: None,
            try_block_error_types: std::collections::HashSet::new(),
            error_types: std::collections::HashSet::new(),
            error_hierarchy: HashMap::new(),
            vararg_functions: HashMap::new(),
            type_vars: std::collections::HashSet::new(),
            generic_functions: HashMap::new(),
            type_param_bounds: HashMap::new(),
            declared_type_var_bounds: HashMap::new(),
            allow_intrinsic_imports: false,
            borrowed_params: std::collections::HashSet::new(),
            class_declared_type_params: HashMap::new(),
            current_module_name: None,
            externals: ExternalDefs::default(),
            asyncio_compat_imports: HashMap::new(),
            synthetic_imports: Vec::new(),
            synthetic_import_aliases: HashMap::new(),
            sequence_guards: Vec::new(),
            len_aliases: Vec::new(),
            sequence_pointers: Vec::new(),
            numeric_sentinel_vars: HashMap::new(),
            pending_numeric_sentinel_patches: HashMap::new(),
            pending_container_specialization_patches: HashMap::new(),
            async_generator_advances: Default::default(),
            sequence_shapes: Vec::new(),
            proven_nonzero_integer_bindings: std::collections::HashSet::new(),
            function_scopes: Vec::new(),
            inferred_binding_hints: Vec::new(),
            empty_collection_hint_adoption: Vec::new(),
            empty_dict_specializations: HashMap::new(),
            const_integer_values: HashMap::new(),
        }
    }

    fn is_stdlib_lowering(&self) -> bool {
        self.allow_intrinsic_imports
    }

    fn error_with_code_at(
        &mut self,
        code: DiagnosticCode,
        message: String,
        range: TextRange,
    ) -> ErrorTaint {
        let taint = ErrorTaint::emitted();
        self.errors.push(HirDiagnostic {
            code: Some(code),
            message,
            primary_range: Some(range),
            line: None,
            col: None,
        });
        self.last_error_taint = Some(taint);
        taint
    }
    fn error_count(&self) -> usize {
        self.errors.len()
    }
    fn error_taint_since(&self, previous_error_count: usize) -> Option<ErrorTaint> {
        (self.errors.len() > previous_error_count)
            .then_some(self.last_error_taint)
            .flatten()
    }
    fn is_poisoned_binding(&self, name: &str) -> bool {
        self.scope
            .lookup(name)
            .is_some_and(crate::scope::VarInfo::is_poisoned_binding)
    }
    fn in_loop(&self) -> bool {
        self.loop_depth > 0
    }
    fn inferred_binding_hint(&self, name: &str) -> Option<&Type> {
        self.inferred_binding_hints
            .iter()
            .rev()
            .find_map(|hints| hints.get(name))
    }

    fn push_empty_collection_hint_adoption(&mut self, allow: bool) {
        self.empty_collection_hint_adoption.push(allow);
    }

    fn pop_empty_collection_hint_adoption(&mut self) {
        let _ = self.empty_collection_hint_adoption.pop();
    }

    fn can_adopt_empty_collection_hints(&self) -> bool {
        self.empty_collection_hint_adoption
            .last()
            .copied()
            .unwrap_or(false)
    }
}
#[cfg(test)]
mod diagnostic_transport_tests;
/// Substitute type variables in a type with concrete types.
fn substitute_type_vars(ty: &Type, bindings: &HashMap<String, Type>) -> Type {
    fn substitute_function_type(
        ft: &FunctionType,
        bindings: &HashMap<String, Type>,
    ) -> FunctionType {
        let params = ft
            .params
            .iter()
            .map(|(name, ty, convention)| {
                (
                    name.clone(),
                    substitute_type_vars(ty, bindings),
                    *convention,
                )
            })
            .collect();
        let return_type = Box::new(substitute_type_vars(&ft.return_type, bindings));
        FunctionType {
            params,
            return_type,
        }
    }

    match ty {
        Type::TypeVar(name) => bindings.get(name).cloned().unwrap_or_else(|| ty.clone()),
        Type::List(elem) => Type::List(Box::new(substitute_type_vars(elem, bindings))),
        Type::Set(elem) => Type::Set(Box::new(substitute_type_vars(elem, bindings))),
        Type::Iterable(elem) => Type::Iterable(Box::new(substitute_type_vars(elem, bindings))),
        Type::Iterator(elem) => Type::Iterator(Box::new(substitute_type_vars(elem, bindings))),
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
        Type::Result(ok, err) => Type::Result(
            Box::new(substitute_type_vars(ok, bindings)),
            Box::new(substitute_type_vars(err, bindings)),
        ),
        Type::Coroutine(ok, err) => Type::Coroutine(
            Box::new(substitute_type_vars(ok, bindings)),
            Box::new(substitute_type_vars(err, bindings)),
        ),
        Type::Task(ok, err) => Type::Task(
            Box::new(substitute_type_vars(ok, bindings)),
            Box::new(substitute_type_vars(err, bindings)),
        ),
        Type::TaskResult(ok, err) => Type::TaskResult(
            Box::new(substitute_type_vars(ok, bindings)),
            Box::new(substitute_type_vars(err, bindings)),
        ),
        Type::Failure(err) => Type::Failure(Box::new(substitute_type_vars(err, bindings))),
        Type::Select2(first, second) => Type::Select2(
            Box::new(substitute_type_vars(first, bindings)),
            Box::new(substitute_type_vars(second, bindings)),
        ),
        Type::TimeoutResult(err) => {
            Type::TimeoutResult(Box::new(substitute_type_vars(err, bindings)))
        }
        Type::BlockingTask(ok, err) => Type::BlockingTask(
            Box::new(substitute_type_vars(ok, bindings)),
            Box::new(substitute_type_vars(err, bindings)),
        ),
        Type::Awaitable(result) => {
            Type::Awaitable(Box::new(substitute_type_vars(result, bindings)))
        }
        Type::AsyncIterator(item, err) => Type::AsyncIterator(
            Box::new(substitute_type_vars(item, bindings)),
            Box::new(substitute_type_vars(err, bindings)),
        ),
        Type::AsyncGenerator(item, err) => Type::AsyncGenerator(
            Box::new(substitute_type_vars(item, bindings)),
            Box::new(substitute_type_vars(err, bindings)),
        ),
        Type::Alias {
            name,
            type_args,
            body,
        } => Type::Alias {
            name: name.clone(),
            type_args: type_args
                .iter()
                .map(|arg| substitute_type_vars(arg, bindings))
                .collect(),
            body: Box::new(substitute_type_vars(body, bindings)),
        },
        Type::Function(ft) => Type::Function(substitute_function_type(ft, bindings)),
        Type::AsyncFunction(ft) => Type::AsyncFunction(substitute_function_type(ft, bindings)),
        Type::Class {
            name,
            fields,
            methods,
            parent_class,
        } => Type::Class {
            name: name.clone(),
            fields: fields
                .iter()
                .map(|(field_name, field_ty)| {
                    (field_name.clone(), substitute_type_vars(field_ty, bindings))
                })
                .collect(),
            methods: methods
                .iter()
                .map(|(method_name, method_ft)| {
                    (
                        method_name.clone(),
                        substitute_function_type(method_ft, bindings),
                    )
                })
                .collect(),
            parent_class: parent_class.clone(),
        },
        _ => ty.clone(),
    }
}
/// Result of lowering, including the HIR module and any diagnostics.
pub struct LoweringResult {
    pub module: HirModule,
    pub function_defaults: std::collections::HashMap<String, Vec<(usize, HirExpr)>>,
    pub function_varargs: std::collections::HashMap<String, usize>,
    pub constant_integer_values: std::collections::HashMap<String, num_bigint::BigInt>,
    /// `reveal_type()` diagnostics (informational, printed to stderr)
    pub reveal_types: Vec<RevealTypeDiagnostic>,
    /// Compiler warnings (non-fatal diagnostics)
    pub warnings: Vec<LoweringWarningDiagnostic>,
}
/// Lower a parsed module AST into a typed HIR module.
pub fn lower_module(stmts: &[Stmt]) -> Result<LoweringResult, Vec<HirDiagnostic>> {
    lower_module_with_externals(stmts, &ExternalDefs::default())
}
/// Lower a stdlib .sifr module. Allows _sifr.* intrinsic imports.
pub fn lower_module_stdlib(stmts: &[Stmt]) -> Result<LoweringResult, Vec<HirDiagnostic>> {
    let mut ctx = LowerCtx::new();
    ctx.allow_intrinsic_imports = true;
    lower_module_impl(stmts, &ExternalDefs::default(), ctx)
}
/// Lower a stdlib .sifr module with external definitions (for inter-stdlib deps).
pub fn lower_module_stdlib_with_externals(
    stmts: &[Stmt],
    externals: &ExternalDefs,
) -> Result<LoweringResult, Vec<HirDiagnostic>> {
    let mut ctx = LowerCtx::new();
    ctx.allow_intrinsic_imports = true;
    lower_module_impl(stmts, externals, ctx)
}
/// Lower a parsed module AST into a typed HIR module, with external module definitions.
pub fn lower_module_with_externals(
    stmts: &[Stmt],
    externals: &ExternalDefs,
) -> Result<LoweringResult, Vec<HirDiagnostic>> {
    let ctx = LowerCtx::new();
    lower_module_impl(stmts, externals, ctx)
}

pub fn lower_module_with_externals_and_name(
    module_name: &str,
    stmts: &[Stmt],
    externals: &ExternalDefs,
) -> Result<LoweringResult, Vec<HirDiagnostic>> {
    let ctx = LowerCtx::new().with_current_module(module_name);
    lower_module_impl(stmts, externals, ctx)
}
/// Internal implementation of module lowering.
fn lower_module_impl(
    stmts: &[Stmt],
    externals: &ExternalDefs,
    mut ctx: LowerCtx,
) -> Result<LoweringResult, Vec<HirDiagnostic>> {
    ctx.externals = externals.clone();
    // Register built-in functions
    register_builtins(&mut ctx);
    integer_literal_diagnostics::validate_module_integer_literals(stmts, &mut ctx);
    // Pass 0: Pre-register all class names as forward references.
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
                                ctx.type_vars.insert(name.id.to_string());
                                let specs = parse_typevar_declaration_specs(call, &mut ctx);
                                if !specs.is_empty() {
                                    ctx.declared_type_var_bounds
                                        .insert(name.id.to_string(), specs);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    // Early import pass: resolve imported types so they're available for function signatures.
    // This must happen before function signature extraction so that imported error classes
    // (e.g., StatisticsError from sifr.statistics) can be used in Result[T, E] annotations.
    resolve_imports_early(stmts, externals, &mut ctx);
    let alias_decls = collect_type_alias_decls(stmts, &mut ctx);
    predeclare_type_aliases(&alias_decls, &mut ctx);
    // First class pass materializes full class shapes before alias resolution so aliases like
    // `type Shape = Circle | Square` see concrete class fields.
    for stmt in stmts {
        if let Stmt::ClassDef(class_def) = stmt {
            collect_class_type(class_def, &mut ctx, false);
        }
    }
    resolve_type_aliases(&alias_decls, &mut ctx);
    // Refresh class definitions after alias resolution so class field/method annotations that
    // depend on aliases declared later in the module see the final alias shapes.
    for stmt in stmts {
        if let Stmt::ClassDef(class_def) = stmt {
            collect_class_type(class_def, &mut ctx, true);
        }
    }
    let mut function_name_registry = module_function_registry::ModuleFunctionRegistry::default();
    for stmt in stmts {
        if let Stmt::FunctionDef(func) = stmt {
            let function_name = func.name.to_string();
            if !function_name_registry.note_module_decl(
                function_name.as_str(),
                func.name.range(),
                &mut ctx,
            ) {
                continue;
            }
            // PEP 695: register inline type params (def f[T](...)) as type variables
            let mut pep695_type_vars = Vec::new();
            if let Some(ref type_params) = func.type_params {
                for tp in type_params.iter() {
                    if let sifr_python_ast::TypeParam::TypeVar(tv) = tp {
                        let name = tv.name.to_string();
                        ctx.type_vars.insert(name.clone());
                        pep695_type_vars.push(name.clone());
                        if let Some(ref bound) = tv.bound {
                            let specs = parse_typevar_bound_expr(bound, &mut ctx);
                            if !specs.is_empty() {
                                ctx.type_param_bounds
                                    .entry(function_name.clone())
                                    .or_default()
                                    .entry(name)
                                    .or_default()
                                    .extend(specs);
                            }
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
                    .insert(function_name.clone(), func_type_vars);
            }

            // Apply globally declared `TypeVar(...)` bounds/constraints to this function's
            // referenced type variables.
            if let Some(type_vars) = ctx
                .generic_functions
                .get(func.name.to_string().as_str())
                .cloned()
            {
                for tv_name in &type_vars {
                    if let Some(specs) = ctx.declared_type_var_bounds.get(tv_name) {
                        ctx.type_param_bounds
                            .entry(function_name.clone())
                            .or_default()
                            .entry(tv_name.clone())
                            .or_default()
                            .extend(specs.clone());
                    }
                }
            }

            collect_function_defaults(&mut ctx, &function_name, func);
            let effective_is_async =
                func.is_async || function_uses_asyncio_run_entrypoint(func, &ctx);
            if effective_is_async && function_body_contains_yield(&func.body) {
                ctx.async_generator_functions.insert(function_name.clone());
            } else if effective_is_async {
                ctx.async_functions.insert(function_name.clone());
            }
            if let Some(workload) =
                workload_annotations::annotation_for_decorators(func.decorator_list.iter())
            {
                ctx.function_workload_annotations
                    .insert(function_name.clone(), workload);
            }
            ctx.functions.insert(function_name.clone(), ft);
            if func.parameters.vararg.is_some() {
                ctx.vararg_functions
                    .insert(function_name, func.parameters.args.len());
            }
        }
    }
    ctx.async_suspension_summaries = async_effects::collect_async_suspension_summaries(stmts);
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
                import_diagnostics::unsupported_form(
                    &mut ctx,
                    format!(
                        "relative import level {} for module '{module_name}'",
                        import_from.level
                    )
                    .as_str(),
                    import_from.range(),
                );
                continue;
            }
            let Some(ref module) = import_from.module else {
                import_diagnostics::unsupported_form(
                    &mut ctx,
                    "bare relative import; use 'from <module> import ...'",
                    import_from.range(),
                );
                continue;
            };
            let module_name =
                ctx.effective_import_module_name(module.as_ref(), import_from.level, externals);
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
            let import_range = import_from.range();
            let imported_name_range = |original: &str| -> TextRange {
                import_from
                    .names
                    .iter()
                    .find(|alias| alias.name.as_str() == original)
                    .map_or(import_range, Ranged::range)
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
                    import_diagnostics::forbidden_intrinsic(&mut ctx, &module_name, import_range);
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
                            name_diagnostics::missing_member(
                                &mut ctx,
                                &module_name,
                                name,
                                imported_name_range(name),
                            );
                        }
                    }
                    imports.push(HirImport {
                        module: module_name,
                        names,
                        aliases,
                    });
                    continue;
                }
                import_diagnostics::unknown_import_target(&mut ctx, &module_name, import_range);
                continue;
            }

            // Check if this is a stdlib import (sifr.*)
            // All sifr.* modules are now compiled from .sifr stdlib sources.
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
                                if let Some(module_defaults) =
                                    externals.function_defaults.get(&stdlib_module_key)
                                {
                                    imported_defaults::import_callable_defaults(
                                        &mut ctx,
                                        module_defaults,
                                        name,
                                        &local,
                                    );
                                }
                                if let Some(module_varargs) =
                                    externals.function_varargs.get(&stdlib_module_key)
                                {
                                    imported_defaults::import_callable_vararg(
                                        &mut ctx,
                                        module_varargs,
                                        name,
                                        &local,
                                    );
                                }
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
                            if let Some(module_classes) = externals.classes.get(&stdlib_module_key)
                            {
                                if let Some(class_ty) = module_classes.get(name) {
                                    ctx.class_types.insert(local.clone(), class_ty.clone());
                                    if let Some(module_class_type_params) =
                                        externals.class_type_params.get(&stdlib_module_key)
                                    {
                                        if let Some(type_params) =
                                            module_class_type_params.get(name)
                                        {
                                            ctx.class_declared_type_params
                                                .insert(local.clone(), type_params.clone());
                                            if !type_params.is_empty() {
                                                ctx.generic_functions
                                                    .insert(local.clone(), type_params.clone());
                                            }
                                        }
                                    }
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
                                        if let Some(module_defaults) =
                                            externals.function_defaults.get(&stdlib_module_key)
                                        {
                                            imported_defaults::import_class_method_defaults(
                                                &mut ctx,
                                                module_defaults,
                                                name,
                                                &local,
                                            );
                                        }
                                        if let Some(module_varargs) =
                                            externals.function_varargs.get(&stdlib_module_key)
                                        {
                                            imported_defaults::import_class_method_varargs(
                                                &mut ctx,
                                                module_varargs,
                                                name,
                                                &local,
                                            );
                                        }
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
                            if let Some(module_consts) = externals.constants.get(&stdlib_module_key)
                            {
                                if let Some(const_ty) = module_consts.get(name) {
                                    ctx.scope.define(local.clone(), const_ty.clone());
                                    if let Some(value) = externals
                                        .constant_integer_values
                                        .get(&stdlib_module_key)
                                        .and_then(|module_values| module_values.get(name))
                                    {
                                        ctx.const_integer_values.insert(local, value.clone());
                                    }
                                    found = true;
                                }
                            }
                        }
                        if !found {
                            imports::report_missing_stdlib_member(
                                &mut ctx,
                                &module_name,
                                name,
                                imported_name_range(name),
                            );
                        }
                    }
                    imports.push(HirImport {
                        module: module_name,
                        names,
                        aliases,
                    });
                    continue;
                }
                imports::report_unknown_stdlib_module(&mut ctx, &module_name, import_range);
                continue;
            }

            // Check if the local module exists in externals before resolving
            let has_local_module =
                import_resolution::external_module_exists(externals, &module_name);
            if !has_local_module {
                import_diagnostics::unknown_import_target(&mut ctx, &module_name, import_range);
                continue;
            }

            // Resolve imported names from external definitions (local modules)
            for name in &names {
                let local = local_name_for(name);
                // Check if it's a private name
                if name.starts_with('_') {
                    import_diagnostics::private_member(
                        &mut ctx,
                        &module_name,
                        name,
                        imported_name_range(name),
                    );
                    continue;
                }

                let mut found = false;
                // Look up in external functions
                if let Some(module_fns) = externals.functions.get(&module_name) {
                    if let Some(ft) = module_fns.get(name) {
                        ctx.functions.insert(local.clone(), ft.clone());
                        if let Some(module_defaults) = externals.function_defaults.get(&module_name)
                        {
                            imported_defaults::import_callable_defaults(
                                &mut ctx,
                                module_defaults,
                                name,
                                &local,
                            );
                        }
                        if let Some(module_varargs) = externals.function_varargs.get(&module_name) {
                            imported_defaults::import_callable_vararg(
                                &mut ctx,
                                module_varargs,
                                name,
                                &local,
                            );
                        }
                        found = true;
                    }
                }
                // Look up in external classes
                if !found {
                    if let Some(module_classes) = externals.classes.get(&module_name) {
                        if let Some(class_ty) = module_classes.get(name) {
                            ctx.class_types.insert(local.clone(), class_ty.clone());
                            if let Some(module_class_type_params) =
                                externals.class_type_params.get(&module_name)
                            {
                                if let Some(type_params) = module_class_type_params.get(name) {
                                    ctx.class_declared_type_params
                                        .insert(local.clone(), type_params.clone());
                                    if !type_params.is_empty() {
                                        ctx.generic_functions
                                            .insert(local.clone(), type_params.clone());
                                    }
                                }
                            }
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
                                if let Some(module_defaults) =
                                    externals.function_defaults.get(&module_name)
                                {
                                    imported_defaults::import_class_method_defaults(
                                        &mut ctx,
                                        module_defaults,
                                        name,
                                        &local,
                                    );
                                }
                                if let Some(module_varargs) =
                                    externals.function_varargs.get(&module_name)
                                {
                                    imported_defaults::import_class_method_varargs(
                                        &mut ctx,
                                        module_varargs,
                                        name,
                                        &local,
                                    );
                                }
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
                            if let Some(value) = externals
                                .constant_integer_values
                                .get(&module_name)
                                .and_then(|module_values| module_values.get(name))
                            {
                                ctx.const_integer_values
                                    .insert(local.clone(), value.clone());
                            }
                            found = true;
                        }
                    }
                }
                if !found {
                    name_diagnostics::missing_member(
                        &mut ctx,
                        &module_name,
                        name,
                        imported_name_range(name),
                    );
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
                import_diagnostics::unsupported_form(
                    &mut ctx,
                    format!("import {module_name}; use 'from {module_name} import <name>'")
                        .as_str(),
                    alias.name.range(),
                );
            }
        }
    }
    let constants = module_constants_lowering::collect_module_constants(stmts, &mut ctx);
    // Second pass: lower function bodies and class method bodies
    let mut functions = Vec::new();
    let mut classes = Vec::new();
    for stmt in stmts {
        match stmt {
            Stmt::FunctionDef(func) => {
                let function_name = func.name.to_string();
                if !function_name_registry.note_lowering(function_name.as_str()) {
                    continue;
                }
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
        imports.extend(ctx.synthetic_imports.clone());
        Ok(LoweringResult {
            module: HirModule {
                functions,
                classes,
                imports,
                constants,
                generic_functions: ctx.generic_functions.clone(),
                type_param_bounds: ctx.type_param_bounds.clone(),
            },
            function_defaults: ctx.function_defaults.clone(),
            function_varargs: ctx.vararg_functions.clone(),
            constant_integer_values: ctx.const_integer_values.clone(),
            reveal_types: ctx.reveal_types,
            warnings: ctx.warnings,
        })
    } else {
        Err(ctx.errors)
    }
}
