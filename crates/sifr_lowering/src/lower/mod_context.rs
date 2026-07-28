use super::{
    async_effects, async_generator_advances, diagnostic_types, external_defs, function_scopes,
    len_aliases, mod_impl, numeric_sentinels, sequence_guards, sequence_pointers, sequence_shapes,
    str, workload_annotations,
};
use crate::hir_nodes::HirExpr;
use crate::scope::{ErrorTaint, Scope};
use async_effects::AsyncSuspensionSummary;
use diagnostic_types::{HirDiagnostic, LoweringWarningDiagnostic, RevealTypeDiagnostic};
use external_defs::ExternalDefs;
use len_aliases::LenAliasFact;
use mod_impl::lower_module_impl;
use ruff_text_size::TextRange;
use sequence_guards::SequenceGuard;
use sequence_pointers::SequencePointerFact;
use sifr_diagnostics::{DiagnosticArg, DiagnosticCode};
use sifr_ir::{CompilerIntrinsicId, FlowEffect, LoweringResult, PythonCleanupPolicy};
use sifr_python_ast::Stmt;
use sifr_type_system::{make_union, FunctionType, Type};
use std::collections::{BTreeMap, HashMap, HashSet};
use workload_annotations::WorkloadKind;
/// The lowering context that tracks state during AST->HIR conversion.
pub(in crate::lower) struct LowerCtx {
    /// Function signatures (name -> type)
    pub(in crate::lower) functions: HashMap<String, FunctionType>,
    /// Resolved local callable name -> typed compiler intrinsic identity.
    pub(in crate::lower) compiler_intrinsics: HashMap<String, CompilerIntrinsicId>,
    pub(in crate::lower) async_functions: std::collections::HashSet<String>,
    pub(in crate::lower) async_generator_functions: std::collections::HashSet<String>,
    pub(in crate::lower) async_suspension_summaries: HashMap<String, AsyncSuspensionSummary>,
    /// Workload classification decorators for user-defined functions.
    pub(in crate::lower) function_workload_annotations: HashMap<String, WorkloadKind>,
    /// Default parameter values for functions (name -> vec of (`param_index`, `default_expr`))
    pub(in crate::lower) function_defaults: HashMap<String, Vec<(usize, HirExpr)>>,
    /// Class type definitions (name -> `Type::Class`)
    pub(in crate::lower) class_types: HashMap<String, Type>,
    /// Instance methods, keyed as `Class.method`, including imported metadata.
    pub(in crate::lower) class_instance_methods: HashSet<String>,
    /// Nearest defining class for each flattened `Class.method` surface.
    pub(in crate::lower) class_method_origins: HashMap<String, String>,
    /// Class name -> declaration metadata for sealed Python-backed identities.
    pub(in crate::lower) python_opaque_classes: HashMap<String, sifr_ir::PythonInteropDeclaration>,
    /// General affine must-use obligations keyed by their current owning binding.
    pub(in crate::lower) live_must_use_bindings:
        HashMap<String, super::must_use_obligations::MustUseObligation>,
    /// Qualified opaque methods whose source receiver is declared `own self`.
    pub(in crate::lower) python_consuming_methods: HashSet<String>,
    /// Qualified Rust opaque close members selected by their class declaration.
    pub(in crate::lower) rust_consuming_methods: HashSet<String>,
    /// Qualified context exits callable only from dedicated Python-with lowering.
    pub(in crate::lower) python_context_exit_methods: HashSet<String>,
    /// Current scope for name resolution
    pub(in crate::lower) scope: Scope,
    /// First non-Never child error type observed for each in-scope `TaskGroup` binding.
    pub(in crate::lower) task_group_error_types: HashMap<String, Type>,
    /// In-scope task handle binding -> owning `TaskGroup` binding.
    pub(in crate::lower) task_handle_group_owners: HashMap<String, String>,
    /// `JoinSet` bindings that have accepted at least one handle and must be consumed.
    pub(in crate::lower) live_join_set_bindings: std::collections::HashSet<String>,
    /// Awaitable bindings produced by `JoinSet` terminal calls, mapped to their `JoinSet` owner.
    pub(in crate::lower) join_set_terminal_awaitables: HashMap<String, String>,
    /// `TaskGroup` bindings that are no longer proven Open after observing a child handle.
    pub(in crate::lower) task_groups_not_proven_open: std::collections::HashSet<String>,
    /// Nesting depth of active structured task owners while lowering a function body.
    pub(in crate::lower) active_task_owner_depth: usize,
    /// Active structured task owner bindings available to module-level scoped helpers.
    pub(in crate::lower) active_task_owner_bindings: Vec<(String, Type)>,
    /// Collected diagnostics that stop successful lowering.
    pub(in crate::lower) errors: Vec<HirDiagnostic>,
    /// Proof for the latest emitted lowering diagnostic.
    pub(in crate::lower) last_error_taint: Option<ErrorTaint>,
    /// Proof propagated by an expression suppressed because it reads a poisoned binding.
    propagated_error_taint: Option<ErrorTaint>,
    /// Loop nesting depth (for break/continue validation)
    pub(in crate::lower) loop_depth: usize,
    /// `reveal_type()` diagnostics (informational, not errors)
    pub(in crate::lower) reveal_types: Vec<RevealTypeDiagnostic>,
    /// Compiler warnings (non-fatal diagnostics)
    pub(in crate::lower) warnings: Vec<LoweringWarningDiagnostic>,
    /// Whether we're currently inside a class method (tracks `self` type)
    pub(in crate::lower) current_class: Option<String>,
    /// Current function/method owner name while lowering a body.
    pub(in crate::lower) current_owner: Option<String>,
    /// Current class method name, used to retain generated Rust trait requirements per method.
    pub(in crate::lower) current_method: Option<String>,
    /// The parent class name of the current class (for `super()` resolution)
    pub(in crate::lower) current_parent_class: Option<String>,
    /// Resolved parent class type of the current class.
    pub(in crate::lower) current_parent_type: Option<Type>,
    /// Whether we're inside a try block (auto-unwrap Result values)
    pub(in crate::lower) in_try_block: bool,
    /// Whether the currently lowered function body is async.
    pub(in crate::lower) current_function_is_async: bool,
    /// Whether the currently lowered function body is an `async def` containing `yield`.
    pub(in crate::lower) current_function_is_async_generator: bool,
    /// Return type of the currently lowered function body.
    pub(in crate::lower) current_function_return_type: Option<Type>,
    /// Error types collected from Result-returning calls during try body lowering.
    /// Each entry is an exact error class type encountered via auto-unwrap.
    pub(in crate::lower) try_block_error_types: std::collections::HashSet<Type>,
    /// Set of class names that are error types (class Foo(Error))
    pub(in crate::lower) error_types: std::collections::HashSet<String>,
    /// Map of parent error type -> list of known child error types (for exhaustiveness checking)
    pub(in crate::lower) error_hierarchy: HashMap<String, Vec<String>>,
    /// Map of function names to the parameter index of their *args (vararg) parameter
    pub(in crate::lower) vararg_functions: HashMap<String, usize>,
    /// Declaration-first Python parameter kinds, retained for call-shape lowering.
    pub(in crate::lower) python_call_shapes: HashMap<String, Vec<sifr_ir::PythonParameterKind>>,
    /// Callback attachment policies keyed by the callable surface used at each call site.
    pub(in crate::lower) python_callback_call_policies:
        HashMap<String, Vec<super::python_interop::CallbackCallPolicy>>,
    /// Callable parameter indices for Rust declarations whose retained
    /// callbacks must be capture-checked at every attachment call site.
    pub(in crate::lower) rust_threadsafe_callback_targets: HashMap<String, Vec<usize>>,
    /// Directly declared nested handlers that must own their capture
    /// environments because they escape into a retained Rust callback.
    pub(in crate::lower) rust_threadsafe_callback_move_handlers: HashMap<String, Vec<String>>,
    /// Set of registered type variable names (e.g., T, K, V from `TypeVar` declarations)
    pub(in crate::lower) type_vars: std::collections::HashSet<String>,
    /// Map of generic function names to their type variable names
    pub(in crate::lower) generic_functions: HashMap<String, Vec<String>>,
    /// Map of owner (function or class name) -> (`type_var_name` -> protocol bounds)
    pub(in crate::lower) type_param_bounds: HashMap<String, HashMap<String, Vec<String>>>,
    /// Global `TypeVar(...)` declaration bounds/constraints by declared type variable name.
    /// Constraints are encoded with `TYPEVAR_CONSTRAINT_PREFIX`.
    pub(in crate::lower) declared_type_var_bounds: HashMap<String, Vec<String>>,
    /// Origin of the source currently being lowered.
    pub(in crate::lower) source_origin: LoweringSourceOrigin,
    /// Set of parameter names that are immutably borrowed (&T) in the current function.
    pub(in crate::lower) borrowed_params: std::collections::HashSet<String>,
    /// Opaque values borrowed from Python context entry for the current lexical block.
    pub(in crate::lower) python_context_borrows: HashMap<String, ruff_text_size::TextRange>,
    /// Map of class names to their declared type parameters (from PEP 695 class C[T])
    pub(in crate::lower) class_declared_type_params: HashMap<String, Vec<String>>,
    /// Class -> method -> type parameter -> generated Rust trait requirements.
    pub(in crate::lower) generic_method_requirements:
        HashMap<String, HashMap<String, HashMap<String, HashSet<String>>>>,
    /// Class -> method -> directly delegated `self.method()` calls.
    pub(in crate::lower) generic_method_dependencies:
        HashMap<String, HashMap<String, HashSet<String>>>,
    pub(in crate::lower) current_module_name: Option<String>,
    pub(in crate::lower) externals: ExternalDefs,
    pub(in crate::lower) explicit_defaultdict_bindings: HashSet<String>,
    pub(in crate::lower) parallel_map_bindings: HashSet<String>,
    pub(in crate::lower) parallel_try_map_bindings: HashSet<String>,
    pub(in crate::lower) python_import_module_bindings: HashSet<String>,
    pub(in crate::lower) current_function_trusts_dynamic_python: bool,
    pub(in crate::lower) python_trust_policy: Option<PythonTrustPolicy>,
    pub(in crate::lower) python_bridge_authorities:
        std::collections::BTreeMap<String, PythonBridgeTargetAuthority>,
    /// Nested local function captures observed while lowering the current statement block.
    pub(in crate::lower) nested_function_captures: HashMap<String, Vec<(String, Type)>>,
    /// Captured bindings mutated by each nested local function.
    pub(in crate::lower) nested_function_mutated_captures: HashMap<String, Vec<String>>,
    pub(in crate::lower) sequence_guards: Vec<SequenceGuard>,
    pub(in crate::lower) len_aliases: Vec<LenAliasFact>,
    pub(in crate::lower) sequence_pointers: Vec<SequencePointerFact>,
    pub(in crate::lower) numeric_sentinel_vars:
        HashMap<String, numeric_sentinels::NumericSentinelFact>,
    pub(in crate::lower) pending_numeric_sentinel_patches:
        HashMap<String, numeric_sentinels::NumericSentinelPatch>,
    pub(in crate::lower) pending_container_specialization_patches: HashMap<String, Type>,
    pub(in crate::lower) async_generator_advances:
        async_generator_advances::AsyncGeneratorAdvanceTracker,
    pub(in crate::lower) sequence_shapes: Vec<sequence_shapes::SequenceShapeFact>,
    pub(in crate::lower) proven_nonzero_integer_bindings: std::collections::HashSet<String>,
    pub(in crate::lower) function_scopes: Vec<function_scopes::FunctionScopeState>,
    pub(in crate::lower) inferred_binding_hints: Vec<HashMap<String, Type>>,
    pub(in crate::lower) empty_collection_hint_adoption: Vec<bool>,
    /// Expected type for a specific expression range while lowering a typed initializer.
    pub(in crate::lower) contextual_expr_types: Vec<(TextRange, Type)>,
    pub(in crate::lower) empty_dict_specializations: HashMap<String, Type>,
    pub(in crate::lower) const_integer_values: HashMap<String, num_bigint::BigInt>,
    pub(in crate::lower) flow_effects: Vec<FlowEffect>,
}

impl LowerCtx {
    pub(in crate::lower) fn new() -> Self {
        Self {
            functions: HashMap::new(),
            compiler_intrinsics: HashMap::new(),
            async_functions: std::collections::HashSet::new(),
            async_generator_functions: std::collections::HashSet::new(),
            async_suspension_summaries: HashMap::new(),
            function_workload_annotations: HashMap::new(),
            function_defaults: HashMap::new(),
            class_types: HashMap::new(),
            class_instance_methods: HashSet::new(),
            class_method_origins: HashMap::new(),
            python_opaque_classes: HashMap::new(),
            live_must_use_bindings: HashMap::new(),
            python_consuming_methods: HashSet::new(),
            rust_consuming_methods: HashSet::new(),
            python_context_exit_methods: HashSet::new(),
            scope: Scope::new(),
            task_group_error_types: HashMap::new(),
            task_handle_group_owners: HashMap::new(),
            live_join_set_bindings: std::collections::HashSet::new(),
            join_set_terminal_awaitables: HashMap::new(),
            task_groups_not_proven_open: std::collections::HashSet::new(),
            active_task_owner_depth: 0,
            active_task_owner_bindings: Vec::new(),
            errors: Vec::new(),
            last_error_taint: None,
            propagated_error_taint: None,
            loop_depth: 0,
            reveal_types: Vec::new(),
            warnings: Vec::new(),
            current_class: None,
            current_owner: None,
            current_method: None,
            current_parent_class: None,
            current_parent_type: None,
            in_try_block: false,
            current_function_is_async: false,
            current_function_is_async_generator: false,
            current_function_return_type: None,
            try_block_error_types: std::collections::HashSet::new(),
            error_types: std::collections::HashSet::new(),
            error_hierarchy: HashMap::new(),
            vararg_functions: HashMap::new(),
            python_call_shapes: HashMap::new(),
            python_callback_call_policies: HashMap::new(),
            rust_threadsafe_callback_targets: HashMap::new(),
            rust_threadsafe_callback_move_handlers: HashMap::new(),
            type_vars: std::collections::HashSet::new(),
            generic_functions: HashMap::new(),
            type_param_bounds: HashMap::new(),
            declared_type_var_bounds: HashMap::new(),
            source_origin: LoweringSourceOrigin::User,
            borrowed_params: std::collections::HashSet::new(),
            python_context_borrows: HashMap::new(),
            class_declared_type_params: HashMap::new(),
            generic_method_requirements: HashMap::new(),
            generic_method_dependencies: HashMap::new(),
            current_module_name: None,
            externals: ExternalDefs::default(),
            explicit_defaultdict_bindings: HashSet::new(),
            parallel_map_bindings: HashSet::new(),
            parallel_try_map_bindings: HashSet::new(),
            python_import_module_bindings: HashSet::new(),
            current_function_trusts_dynamic_python: false,
            python_trust_policy: None,
            python_bridge_authorities: std::collections::BTreeMap::new(),
            nested_function_captures: HashMap::new(),
            nested_function_mutated_captures: HashMap::new(),
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
            contextual_expr_types: Vec::new(),
            empty_dict_specializations: HashMap::new(),
            const_integer_values: HashMap::new(),
            flow_effects: Vec::new(),
        }
    }

    pub(in crate::lower) fn is_stdlib_lowering(&self) -> bool {
        self.source_origin.is_sysroot_source()
    }

    pub(in crate::lower) fn must_use_obligation_for_type(
        &self,
        ty: &Type,
    ) -> Option<super::must_use_obligations::MustUseObligation> {
        use super::must_use_obligations::{MustUseObligation, MustUseObligationKind};
        match ty.resolve_alias() {
            Type::Class { name, .. } => self
                .python_opaque_classes
                .get(name)
                .and_then(|declaration| declaration.cleanup)
                .filter(|cleanup| *cleanup != PythonCleanupPolicy::Drop)
                .map(|cleanup| MustUseObligation {
                    kind: match cleanup {
                        PythonCleanupPolicy::Context => MustUseObligationKind::ContextOnly,
                        PythonCleanupPolicy::AsyncContext => {
                            MustUseObligationKind::AsyncContextOnly
                        }
                        PythonCleanupPolicy::Close | PythonCleanupPolicy::AsyncClose => {
                            MustUseObligationKind::CloseLike
                        }
                        PythonCleanupPolicy::Drop => unreachable!("drop was filtered above"),
                    },
                    label: format!("Python opaque {name} ({cleanup:?})"),
                }),
            Type::List(item) => self.must_use_obligation_for_type(item),
            Type::Tuple(items) | Type::Union(items) => items
                .iter()
                .find_map(|item| self.must_use_obligation_for_type(item)),
            Type::Dict(key, value) => self
                .must_use_obligation_for_type(key)
                .or_else(|| self.must_use_obligation_for_type(value)),
            Type::Result(ok, _) => self.must_use_obligation_for_type(ok),
            _ => None,
        }
    }

    pub(in crate::lower) fn record_must_use_binding(&mut self, name: &str, ty: &Type) {
        if let Some(obligation) = self.must_use_obligation_for_type(ty) {
            self.live_must_use_bindings
                .insert(name.to_string(), obligation);
        } else {
            self.live_must_use_bindings.remove(name);
        }
    }

    pub(in crate::lower) fn is_sysroot_private_declaration(&self) -> bool {
        matches!(
            self.source_origin,
            LoweringSourceOrigin::SysrootPrivateDeclaration
        )
    }

    pub(in crate::lower) fn can_import_private_stdlib_declarations(&self) -> bool {
        self.source_origin.can_import_private_stdlib_declarations()
    }

    #[must_use]
    pub(in crate::lower) fn with_options(mut self, options: LoweringOptions) -> Self {
        self.python_trust_policy = options.python_trust_policy;
        self.python_bridge_authorities = options.python_bridge_authorities;
        self
    }

    pub(in crate::lower) fn error_with_code_at(
        &mut self,
        code: DiagnosticCode,
        message: String,
        range: TextRange,
    ) -> ErrorTaint {
        self.error_with_code_args_help_at(code, message, BTreeMap::new(), None, range)
    }

    pub(in crate::lower) fn error_with_code_args_help_at(
        &mut self,
        code: DiagnosticCode,
        message: String,
        args: BTreeMap<String, DiagnosticArg>,
        help: Option<String>,
        range: TextRange,
    ) -> ErrorTaint {
        let taint = ErrorTaint::emitted();
        self.errors.push(HirDiagnostic {
            code: Some(code),
            message,
            args,
            help,
            primary_range: Some(range),
            line: None,
            col: None,
        });
        self.last_error_taint = Some(taint);
        taint
    }
    pub(in crate::lower) fn error_count(&self) -> usize {
        self.errors.len()
    }
    pub(in crate::lower) fn error_taint_since(
        &self,
        previous_error_count: usize,
    ) -> Option<ErrorTaint> {
        (self.errors.len() > previous_error_count)
            .then_some(self.last_error_taint)
            .flatten()
    }
    pub(in crate::lower) fn begin_initializer_lowering(&mut self) -> usize {
        self.propagated_error_taint = None;
        self.error_count()
    }
    pub(in crate::lower) fn initializer_error_taint_since(
        &mut self,
        previous_error_count: usize,
    ) -> Option<ErrorTaint> {
        self.error_taint_since(previous_error_count)
            .or_else(|| self.propagated_error_taint.take())
    }
    pub(in crate::lower) fn propagate_poisoned_binding_error(&mut self, name: &str) -> bool {
        let taint = self
            .scope
            .lookup(name)
            .and_then(crate::scope::VarInfo::error_taint);
        if taint.is_some() {
            self.propagated_error_taint = taint;
            true
        } else {
            false
        }
    }
    pub(in crate::lower) fn in_loop(&self) -> bool {
        self.loop_depth > 0
    }
    pub(in crate::lower) fn inferred_binding_hint(&self, name: &str) -> Option<&Type> {
        self.inferred_binding_hints
            .iter()
            .rev()
            .find_map(|hints| hints.get(name))
    }

    pub(in crate::lower) fn push_empty_collection_hint_adoption(&mut self, allow: bool) {
        self.empty_collection_hint_adoption.push(allow);
    }

    pub(in crate::lower) fn pop_empty_collection_hint_adoption(&mut self) {
        let _ = self.empty_collection_hint_adoption.pop();
    }

    pub(in crate::lower) fn can_adopt_empty_collection_hints(&self) -> bool {
        self.empty_collection_hint_adoption
            .last()
            .copied()
            .unwrap_or(false)
    }

    pub(in crate::lower) fn push_contextual_expr_type(&mut self, range: TextRange, ty: Type) {
        self.contextual_expr_types.push((range, ty));
    }

    pub(in crate::lower) fn pop_contextual_expr_type(&mut self) {
        let _ = self.contextual_expr_types.pop();
    }

    pub(in crate::lower) fn contextual_expr_type(&self, range: TextRange) -> Option<&Type> {
        self.contextual_expr_types
            .iter()
            .rev()
            .find_map(|(candidate_range, ty)| (*candidate_range == range).then_some(ty))
    }

    pub(in crate::lower) fn record_flow_effect(&mut self, effect: FlowEffect) {
        self.flow_effects.push(effect);
    }

    pub(in crate::lower) fn narrow_var_with_flow(
        &mut self,
        name: &str,
        narrowed_type: Type,
        condition: String,
        is_true: bool,
    ) {
        self.scope.narrow_var(name, narrowed_type.clone());
        self.record_flow_effect(FlowEffect::Narrow {
            binding: name.to_string(),
            narrowed_type,
            condition,
            is_true,
        });
    }

    pub(in crate::lower) fn clear_narrowing_with_flow(&mut self, name: &str) {
        self.scope.clear_narrowing(name);
        self.record_flow_effect(FlowEffect::ClearNarrowing {
            binding: name.to_string(),
        });
    }

    pub(in crate::lower) fn mark_moved_with_flow(&mut self, name: &str) -> bool {
        if let Some(range) = self.python_context_borrows.get(name).copied() {
            self.error_with_code_at(
                DiagnosticCode::PYCTX_INVALID_DECLARATION,
                format!(
                    "invalid Python context declaration: entered binding '{name}' is a context-scoped borrow and cannot be moved or closed independently"
                ),
                range,
            );
            return false;
        }
        let moved = self.scope.mark_moved(name);
        if moved {
            self.record_flow_effect(FlowEffect::Move {
                binding: name.to_string(),
            });
        }
        moved
    }

    pub(in crate::lower) fn mark_binding_moved_with_flow(&mut self, name: &str) -> bool {
        let moved = self.scope.mark_binding_moved(name);
        if moved {
            self.record_flow_effect(FlowEffect::Move {
                binding: name.to_string(),
            });
        }
        moved
    }

    pub(in crate::lower) fn reset_moved_with_flow(&mut self, name: &str) {
        self.scope.reset_moved(name);
        self.record_flow_effect(FlowEffect::ResetMove {
            binding: name.to_string(),
        });
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PythonTrustPolicy {
    pub required_import_roots: Vec<String>,
    pub trusted_import_roots: Vec<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LoweringOptions {
    pub python_trust_policy: Option<PythonTrustPolicy>,
    pub python_bridge_authorities: std::collections::BTreeMap<String, PythonBridgeTargetAuthority>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PythonBridgeTargetAuthority {
    pub runtime_package: String,
    pub modules: std::collections::BTreeSet<String>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LoweringSourceOrigin {
    #[default]
    User,
    SysrootPublicStdlib,
    SysrootPrivateDeclaration,
}

impl LoweringSourceOrigin {
    const fn is_sysroot_source(self) -> bool {
        matches!(
            self,
            Self::SysrootPublicStdlib | Self::SysrootPrivateDeclaration
        )
    }

    const fn can_import_private_stdlib_declarations(self) -> bool {
        matches!(self, Self::SysrootPublicStdlib)
    }
}
/// Substitute type variables in a type with concrete types.
pub(in crate::lower) fn substitute_type_vars(ty: &Type, bindings: &HashMap<String, Type>) -> Type {
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
        Type::PythonBuffer(elem) => {
            Type::PythonBuffer(Box::new(substitute_type_vars(elem, bindings)))
        }
        Type::PythonDlpackTensor(elem) => {
            Type::PythonDlpackTensor(Box::new(substitute_type_vars(elem, bindings)))
        }
        Type::JoinSet(ok, err) => Type::JoinSet(
            Box::new(substitute_type_vars(ok, bindings)),
            Box::new(substitute_type_vars(err, bindings)),
        ),
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
        Type::AsyncCallable(params, conventions, ret) => Type::AsyncCallable(
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
            identity,
            type_args,
            name,
            fields,
            methods,
            parent_class,
        } => Type::Class {
            identity: identity.clone(),
            type_args: type_args
                .iter()
                .map(|arg| substitute_type_vars(arg, bindings))
                .collect(),
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
/// Lower a parsed module AST into a typed HIR module.
pub fn lower_module(stmts: &[Stmt]) -> Result<LoweringResult, Vec<HirDiagnostic>> {
    lower_module_with_externals(stmts, &ExternalDefs::default())
}
/// Lower a public sysroot stdlib module.
pub fn lower_module_sysroot_public_stdlib(
    stmts: &[Stmt],
) -> Result<LoweringResult, Vec<HirDiagnostic>> {
    let mut ctx = LowerCtx::new();
    ctx.source_origin = LoweringSourceOrigin::SysrootPublicStdlib;
    lower_module_impl(stmts, &ExternalDefs::default(), ctx)
}

/// Lower a public sysroot stdlib module with external definitions.
pub fn lower_module_sysroot_public_stdlib_with_externals(
    stmts: &[Stmt],
    externals: &ExternalDefs,
) -> Result<LoweringResult, Vec<HirDiagnostic>> {
    let mut ctx = LowerCtx::new();
    ctx.source_origin = LoweringSourceOrigin::SysrootPublicStdlib;
    lower_module_impl(stmts, externals, ctx)
}

/// Lower a private sysroot stdlib declaration module with external definitions.
pub fn lower_module_sysroot_private_declaration_with_externals(
    stmts: &[Stmt],
    externals: &ExternalDefs,
) -> Result<LoweringResult, Vec<HirDiagnostic>> {
    let mut ctx = LowerCtx::new();
    ctx.source_origin = LoweringSourceOrigin::SysrootPrivateDeclaration;
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

pub fn lower_module_with_externals_name_and_options(
    module_name: &str,
    stmts: &[Stmt],
    externals: &ExternalDefs,
    options: LoweringOptions,
) -> Result<LoweringResult, Vec<HirDiagnostic>> {
    let ctx = LowerCtx::new()
        .with_current_module(module_name)
        .with_options(options);
    lower_module_impl(stmts, externals, ctx)
}
