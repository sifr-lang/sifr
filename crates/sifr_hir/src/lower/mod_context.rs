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
