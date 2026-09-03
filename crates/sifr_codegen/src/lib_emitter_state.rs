use super::{
    Cell, ClassScope, HashMap, HashSet, HirExpr, HirFunction, HirModule, HirStmt, LoweringStats,
    NestedFnCapture, ParamConvention, RefCell, RuntimeNeeds, RustExpr, RustItem, RustStmt,
    RustType, ScopeContext, Type, body_contains_await, collect_locally_defined_vars,
    collect_mutated_vars_with_sigs, collect_referenced_vars_with_types, default_param_convention,
    hir_analysis, resolve_alias_type_for_plain_call,
    try_lower_simple_stmt_with_scope_result_and_bindings,
};
use crate::stmt_support_emitter::performance_lowering_gate::stmt_needs_performance_lowering;
#[path = "lib_emitter_structured_stmt.rs"]
mod structured_stmt_lowering;
pub struct RustEmitter {
    pub(crate) collection_needs: CollectionNeeds,
    pub(crate) runtime_needs: RuntimeNeeds,
    /// Track union enum types that need to be defined (name -> member types)
    pub(crate) union_enums: HashMap<String, Vec<Type>>,
    /// Canonical immutable structural-record layouts used by this module.
    pub(crate) structural_record_types: HashMap<String, sifr_type_system::StructuralRecordType>,
    /// Union enums used as exact try/except error carriers.
    pub(crate) try_error_carrier_enums: HashSet<String>,
    /// Union enums also used as ordinary source-level values.
    pub(crate) ordinary_union_enums: HashSet<String>,
    /// Ordinary union enums that participate in a structural bridge shape.
    pub(crate) structural_union_enums: HashSet<String>,
    /// Project-owned union enums whose definition is emitted by the crate-root prelude.
    /// The complete union map remains available to lowering and exhaustiveness checks.
    pub(crate) suppressed_union_enum_definitions: HashSet<String>,
    /// Canonical nominal identities mapped to their crate-rooted Rust paths.
    /// Project union definitions use these paths because they live outside source modules.
    pub(crate) project_nominal_type_paths: HashMap<String, String>,
    /// Accumulated union enum items to prepend
    pub(crate) enum_items: Vec<RustItem>,
    /// Accumulated non-enum body items to assemble before raw output rendering.
    pub(crate) body_items: Vec<RustItem>,
    /// The return type of the function currently being emitted
    pub(crate) current_return_type: Option<Type>,
    /// Active `async with task.timeout(...)` duration expressions for await lowering.
    pub(crate) active_timeout_durations: Vec<RustExpr>,
    /// Set of variable names currently narrowed via `if let Some(...)` unwrap
    pub(crate) option_unwrapped_vars: HashSet<String>,
    /// Function signatures: name -> (`param_types_with_conventions`, `return_type`)
    pub(crate) func_signatures: HashMap<String, (Vec<(Type, ParamConvention)>, Type)>,
    /// Stack tracking whether each active loop has an else clause.
    /// The last entry is the innermost active loop context.
    pub(crate) loop_else_stack: Vec<bool>,
    /// Set of variable names that are mutated in the current function body
    pub(crate) mutated_vars: HashSet<String>,
    /// Set of class names that have Display impl (via __str__ or error type)
    pub(crate) display_classes: HashSet<String>,
    /// Map from child class name -> (parent class name, set of parent field names)
    pub(crate) parent_fields: HashMap<String, (String, HashSet<String>)>,
    /// The class currently being emitted (for field access resolution)
    pub(crate) current_class_name: Option<String>,
    /// The source module currently being emitted, when known.
    pub(crate) current_module_name: Option<String>,
    /// Emit structural bridge implementations only when the project declares
    /// a structural Rust interop boundary.
    pub(crate) structural_interop_enabled: bool,
    /// Canonical record identities proven structurally supported against the
    /// complete project module graph. `None` keeps single-module eligibility.
    pub(crate) project_structural_record_identities: Option<HashSet<String>>,
    /// Compiler-owned structural shape expressions computed against the complete
    /// project graph, keyed by canonical source identity.
    pub(crate) project_structural_identity_expressions: Option<HashMap<String, String>>,
    /// Module qualifier used by structural wire identities. Crate-root modules
    /// have no qualifier even when project analysis names them `main`.
    pub(crate) structural_identity_module_name: Option<String>,
    /// Static-program type parameters for each structural bridge function.
    pub(crate) static_program_type_params: HashMap<String, HashSet<String>>,
    /// Type parameters with a source-level `Structural` or `StringStructural` bound.
    pub(crate) structural_type_params: HashMap<String, HashSet<String>>,
    /// Type parameters whose structural shape is restricted to string leaves.
    pub(crate) string_structural_type_params: HashMap<String, HashSet<String>>,
    /// Method-slot owners for each structural bridge function.
    pub(crate) method_slot_type_params: HashMap<String, HashSet<String>>,
    /// Caller-context type parameters for each structural bridge function.
    pub(crate) context_type_params: HashMap<String, HashSet<String>>,
    /// Closed Rust trait requirements for each module-level generic function.
    pub(crate) function_type_param_bounds: crate::function_generic_bounds::FunctionTypeParamBounds,
    /// Canonical module-function declarations used to propagate generic bounds
    /// into class methods without conflating same-named methods or lexical locals.
    pub(crate) module_generic_functions: HashMap<String, HirFunction>,
    /// Set of stdlib/intrinsic modules used (for Cargo dependency injection)
    pub used_stdlib_modules: HashSet<String>,
    /// Set of intrinsic function names (for codegen dispatch)
    pub(crate) intrinsic_functions: HashSet<String>,
    /// Stdlib/runtime features requested by intrinsic registry lowering.
    pub(crate) intrinsic_registry_features: HashSet<sifr_stdlib_manifest::StdlibFeature>,
    /// Set of (`class_name`, `field_name`) pairs that are self-referential and need Box<T>
    pub(crate) recursive_fields: HashSet<(String, String)>,
    /// Map of (`class_name`, `field_name`) -> concrete Rust type used for recursive field storage.
    pub(crate) recursive_field_rust_types: HashMap<(String, String), RustType>,
    /// Map from class name -> ordered list of field names (for constructor arg mapping)
    pub(crate) class_field_order: HashMap<String, Vec<String>>,
    /// Map of (`class_name`, `field_name`) -> field type for method receiver recovery.
    pub(crate) class_field_types: HashMap<(String, String), Type>,
    /// Map from nested function name -> list of captured variable (name, type) pairs
    /// Used to pass extra args at call sites for recursive+capturing nested functions
    pub(crate) nested_fn_captures: HashMap<String, Vec<NestedFnCapture>>,
    /// Map from module-level constant name -> (type, `rust_name`)
    /// For primitives: `rust_name` is the UPPERCASE const name
    /// For strings/complex: `rust_name` is __`const_name()` function call
    pub(crate) module_constants: HashMap<String, (Type, String)>,
    /// Set of class names that have generic type parameters
    pub(crate) generic_classes: HashSet<String>,
    /// Map of generic class name -> list of type parameter names (e.g., `Counter` -> `T`)
    pub(crate) generic_class_params: HashMap<String, Vec<String>>,
    /// Map of generic class name -> original HIR class template.
    pub(crate) generic_class_templates: HashMap<String, sifr_ir::HirClass>,
    /// Opaque Python class declarations available to direct wrapper lowering.
    pub(crate) python_opaque_classes: HashMap<String, sifr_ir::PythonInteropDeclaration>,
    /// Exact typed retained-callback failures stored on each opaque owner.
    pub(crate) python_retained_callback_errors: HashMap<String, Vec<Type>>,
    /// Set of parameter names that are borrowed (&T) in the current function.
    /// Used to emit dereference (*name) in comparisons where &String != String.
    pub(crate) borrowed_params: HashSet<String>,
    /// Set of parameter names that are mutably borrowed (&mut T) in the current function.
    /// Used to avoid double-borrowing: when a &mut param is passed to another &mut param,
    /// we must NOT emit `&mut name` (it's already &mut T); just pass `name` directly.
    pub(crate) mut_borrowed_params: HashSet<String>,
    /// Bindings represented as `Option<&RecursiveClass>` instead of owned
    /// source-level recursive options. This includes shared parameters and
    /// immutable local aliases of borrowed recursive fields.
    pub(crate) recursive_option_borrowed_views: HashSet<String>,
    /// Set of function names that are generators (contain yield statements)
    /// Used to emit .`collect()` when assigning generator results to list[T]
    pub(crate) generator_functions: HashSet<String>,
    /// Map of `module_name` -> set of imported names (for filtering preamble to only used functions)
    pub(crate) imported_stdlib_names: HashMap<String, HashSet<String>>,
    /// Local names imported from first-party project modules.
    pub(crate) imported_project_functions: HashSet<String>,
    /// Rust locals that checked mutable places require to remain mutable.
    pub(crate) protected_mutable_place_roots: HashSet<String>,
    /// Values established by structurally lowered checked-place guards.
    /// Keys describe the original HIR collection place and index expression.
    pub(crate) checked_place_read_witnesses:
        HashMap<String, crate::checked_place::CheckedPlaceReadWitness>,
    /// Witness keys consumed while lowering an isolated structured region.
    pub(crate) checked_place_read_witness_uses: RefCell<Option<HashSet<String>>>,
    /// Monotonic identifier for collision-free checked-place witness locals.
    pub(crate) next_checked_place_read_witness: usize,
    /// Prevents recursive re-entry under an atomic checked-read statement's witnesses.
    pub(crate) checked_place_atomic_guard_suppressed: bool,
    /// Statement-block depth whose current statement is already being lowered
    /// by checked-place mutation refresh handling.
    pub(crate) checked_place_refresh_suppressed_depth: Option<usize>,
    /// Local list comprehensions represented by `SifrNonEmptyVec`, keyed by
    /// binding name with their statically non-empty nesting depth.
    pub(crate) nonempty_list_bindings: HashMap<String, usize>,
    /// Whether we're inside a generator closure (yield -> return Some(val))
    pub(crate) emission_ctx: EmissionContext,
    /// Whether we're inside a `Display::fmt` implementation (for __str__ methods)
    /// Return statements in this context become write!(f, "{}", val) + return Ok(())
    /// Counter for generating unique try-block error enum names
    pub(crate) try_enum_counter: usize,
    /// Depth of try-block closures that capture return statements.
    pub(crate) try_closure_depth: usize,
    /// Per-try closure representation for returns captured from the enclosing function.
    pub(crate) try_closure_return_wrap: Vec<TryClosureReturnWrap>,
    /// Per-try closure target error type for `?` adaptation.
    pub(crate) try_closure_error_type: Vec<String>,
    /// Resolved error type for Python context cause classification in each try closure.
    pub(crate) try_closure_error_type_info: Vec<Option<Type>>,
    /// Monotonic suffix for compiler-generated Python context locals.
    pub(crate) python_context_counter: usize,
    /// Depth of enclosing Python context outcome envelopes while lowering a body.
    pub(crate) python_context_envelope_depth: usize,
    /// Map from variable name -> Callable parameter (type, convention) list.
    /// Populated per-function from params and locals with Callable types.
    /// Used to emit correct &arg/&mut arg/arg for Callable-typed variable calls.
    pub(crate) callable_var_conventions: HashMap<String, Vec<(Type, ParamConvention)>>,
    /// Map from local binding name -> declared type for the active function-like scope.
    /// Used to preserve assignment coercions that depend on the target local type.
    pub(crate) local_binding_types: HashMap<String, Type>,
    /// Per-function cache variables for borrowed string parameters that are indexed or sliced.
    pub(crate) string_char_cache_vars: HashMap<String, String>,
    /// One recursive prepass over the active function body, keyed by HIR block identity.
    pub(crate) body_analysis: crate::body_analysis::BodyAnalysis,
    /// HIR name-expression identities proven safe to move at their final owned boundary.
    pub(crate) last_use_move_exprs: HashSet<usize>,
    /// String names that are indexed/sliced/length-read and may need a local char cache.
    pub(crate) string_char_cache_required_names: HashSet<String>,
    /// Local string names whose loop body repeatedly reads their Unicode length.
    pub(crate) string_char_cache_loop_local_names: HashSet<String>,
    /// Read-only local dict literals that can be materialized once per process.
    pub(crate) hoistable_static_dict_locals: HashSet<String>,
    /// Monotonic suffix for generated hoisted literal static names.
    pub(crate) hoisted_literal_counter: usize,
    /// Local names widened to `T | None` due `name = None` reassignment in current scope.
    pub(crate) none_widened_local_bindings: HashSet<String>,
    /// Local names whose generated Rust binding has been promoted from plain `i64` storage to `SifrInt`.
    pub(crate) sifr_int_local_bindings: RefCell<HashSet<String>>,
    /// Local names pre-promoted to `SifrInt` because a later assignment needs exact-int storage.
    pub(crate) sifr_int_forced_local_bindings: RefCell<HashSet<String>>,
    /// Local names whose generated Rust `Result[int, E]` binding payload is `SifrInt`.
    pub(crate) sifr_int_result_local_bindings: RefCell<HashSet<String>>,
    /// Function names whose generated Rust return type has been promoted from plain `i64` storage to `SifrInt`.
    pub(crate) sifr_int_function_returns: RefCell<HashSet<String>>,
    /// Function names whose `Result[int, E]` generated Rust return payload is `SifrInt`.
    pub(crate) sifr_int_result_function_returns: RefCell<HashSet<String>>,
    /// Class method keys whose `Result[int, E]` generated Rust return payload is `SifrInt`.
    pub(crate) sifr_int_result_method_returns: RefCell<HashSet<String>>,
    /// Module-level function `int` parameters promoted from plain `i64` storage to `SifrInt`.
    pub(crate) sifr_int_function_params: RefCell<HashMap<String, HashSet<usize>>>,
    /// Module-level function `Result[int, E]` parameters promoted from plain `i64` payloads to `SifrInt`.
    pub(crate) sifr_int_result_function_params: RefCell<HashMap<String, HashSet<usize>>>,
    /// Class method `Result[int, E]` parameters promoted from plain `i64` payloads to `SifrInt`.
    pub(crate) sifr_int_result_method_params: RefCell<HashMap<String, HashSet<usize>>>,
    /// Whether the active function-like body returns `SifrInt` for source-level `int`.
    pub(crate) current_sifr_int_return: Cell<bool>,
    /// Whether the active function-like body returns `Result<SifrInt, E>` for source-level `Result[int, E]`.
    pub(crate) current_sifr_int_result_return: Cell<bool>,
    /// Stack used to capture structured statement emission as IR nodes.
    pub(crate) stmt_capture_stack: Vec<Vec<RustStmt>>,
    /// Depth of the currently lowered statement block inside a function-like body.
    pub(crate) stmt_block_depth: usize,
    /// Recursion guard for non-structured emitter paths.
    pub(crate) lowering_stats: LoweringStats,
}

#[derive(Default)]
pub(crate) struct CollectionNeeds {
    pub(crate) needs_hashmap: bool,
    pub(crate) needs_hashset: bool,
    pub(crate) needs_vecdeque: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TryClosureReturnWrap {
    Direct,
    Optional,
    ControlFlow { continue_type: String },
}

#[derive(Default)]
pub(crate) struct EmissionContext {
    pub(crate) in_generator_closure: bool,
    pub(crate) in_display_impl: bool,
}

impl RustEmitter {
    pub(crate) fn new() -> Self {
        Self {
            collection_needs: CollectionNeeds::default(),
            runtime_needs: RuntimeNeeds::default(),
            union_enums: HashMap::new(),
            structural_record_types: HashMap::new(),
            try_error_carrier_enums: HashSet::new(),
            ordinary_union_enums: HashSet::new(),
            structural_union_enums: HashSet::new(),
            suppressed_union_enum_definitions: HashSet::new(),
            project_nominal_type_paths: HashMap::new(),
            enum_items: Vec::new(),
            body_items: Vec::new(),
            current_return_type: None,
            active_timeout_durations: Vec::new(),
            option_unwrapped_vars: HashSet::new(),
            func_signatures: HashMap::new(),
            loop_else_stack: Vec::new(),
            mutated_vars: HashSet::new(),
            display_classes: HashSet::new(),
            parent_fields: HashMap::new(),
            current_class_name: None,
            current_module_name: None,
            structural_interop_enabled: false,
            project_structural_record_identities: None,
            project_structural_identity_expressions: None,
            structural_identity_module_name: None,
            static_program_type_params: HashMap::new(),
            structural_type_params: HashMap::new(),
            string_structural_type_params: HashMap::new(),
            method_slot_type_params: HashMap::new(),
            context_type_params: HashMap::new(),
            function_type_param_bounds: HashMap::new(),
            module_generic_functions: HashMap::new(),
            used_stdlib_modules: HashSet::new(),
            intrinsic_functions: HashSet::new(),
            intrinsic_registry_features: HashSet::new(),
            recursive_fields: HashSet::new(),
            recursive_field_rust_types: HashMap::new(),
            class_field_order: HashMap::new(),
            class_field_types: HashMap::new(),
            nested_fn_captures: HashMap::new(),
            module_constants: HashMap::new(),
            generic_classes: HashSet::new(),
            generic_class_params: HashMap::new(),
            generic_class_templates: HashMap::new(),
            python_opaque_classes: HashMap::new(),
            python_retained_callback_errors: HashMap::new(),
            borrowed_params: HashSet::new(),
            mut_borrowed_params: HashSet::new(),
            recursive_option_borrowed_views: HashSet::new(),
            generator_functions: HashSet::new(),
            imported_stdlib_names: HashMap::new(),
            imported_project_functions: HashSet::new(),
            protected_mutable_place_roots: HashSet::new(),
            checked_place_read_witnesses: HashMap::new(),
            checked_place_read_witness_uses: RefCell::new(None),
            next_checked_place_read_witness: 0,
            checked_place_atomic_guard_suppressed: false,
            checked_place_refresh_suppressed_depth: None,
            nonempty_list_bindings: HashMap::new(),
            emission_ctx: EmissionContext::default(),
            try_enum_counter: 0,
            try_closure_depth: 0,
            try_closure_return_wrap: Vec::new(),
            try_closure_error_type: Vec::new(),
            try_closure_error_type_info: Vec::new(),
            python_context_counter: 0,
            python_context_envelope_depth: 0,
            callable_var_conventions: HashMap::new(),
            local_binding_types: HashMap::new(),
            string_char_cache_vars: HashMap::new(),
            body_analysis: crate::body_analysis::BodyAnalysis::default(),
            last_use_move_exprs: HashSet::new(),
            string_char_cache_required_names: HashSet::new(),
            string_char_cache_loop_local_names: HashSet::new(),
            hoistable_static_dict_locals: HashSet::new(),
            hoisted_literal_counter: 0,
            none_widened_local_bindings: HashSet::new(),
            sifr_int_local_bindings: RefCell::new(HashSet::new()),
            sifr_int_forced_local_bindings: RefCell::new(HashSet::new()),
            sifr_int_result_local_bindings: RefCell::new(HashSet::new()),
            sifr_int_function_returns: RefCell::new(HashSet::new()),
            sifr_int_result_function_returns: RefCell::new(HashSet::new()),
            sifr_int_result_method_returns: RefCell::new(HashSet::new()),
            sifr_int_function_params: RefCell::new(HashMap::new()),
            sifr_int_result_function_params: RefCell::new(HashMap::new()),
            sifr_int_result_method_params: RefCell::new(HashMap::new()),
            current_sifr_int_return: Cell::new(false),
            current_sifr_int_result_return: Cell::new(false),
            stmt_capture_stack: Vec::new(),
            stmt_block_depth: 0,
            lowering_stats: LoweringStats::default(),
        }
    }

    pub(crate) fn capture_structured_stmts<F>(&mut self, emit: F) -> Vec<RustStmt>
    where
        F: FnOnce(&mut Self),
    {
        self.stmt_capture_stack.push(Vec::new());
        emit(self);
        self.stmt_capture_stack.pop().unwrap_or_default()
    }

    pub(crate) fn collect_recursive_nested_fn_captures(
        &self,
        func: &HirFunction,
    ) -> Vec<NestedFnCapture> {
        if !crate::hir_analysis::queries::body_calls_function(&func.body, &func.name) {
            return Vec::new();
        }

        self.collect_nested_fn_lexical_captures(func)
    }

    pub(crate) fn collect_nested_fn_lexical_captures(
        &self,
        func: &HirFunction,
    ) -> Vec<NestedFnCapture> {
        let param_names = func
            .params
            .iter()
            .map(|param| param.name.clone())
            .collect::<HashSet<_>>();
        let referenced_with_types = collect_referenced_vars_with_types(&func.body);
        let locally_defined = collect_locally_defined_vars(&func.body);
        let mutated_captures = collect_mutated_vars_with_sigs(&func.body, &self.func_signatures);

        let mut captures = referenced_with_types
            .into_iter()
            .filter(|(name, _)| !param_names.contains(name) && !locally_defined.contains(name))
            .map(|(name, ty)| {
                let convention = if ty.ownership() == sifr_type_system::OwnershipKind::Copy {
                    ParamConvention::own()
                } else if mutated_captures.contains(&name) {
                    ParamConvention::mut_borrow()
                } else {
                    default_param_convention(&ty)
                };
                NestedFnCapture {
                    name,
                    ty,
                    convention,
                }
            })
            .collect::<Vec<_>>();
        captures.sort_by(|left, right| left.name.cmp(&right.name));
        captures
    }

    pub(crate) fn emit_module(&mut self, module: &HirModule, module_public: bool, test_mode: bool) {
        self.emit_named_module(module, module_public, test_mode, None);
    }

    pub(crate) fn emit_named_module(
        &mut self,
        module: &HirModule,
        module_public: bool,
        test_mode: bool,
        module_name: Option<&str>,
    ) {
        let saved_module_name = self.current_module_name.clone();
        self.current_module_name = module_name.map(str::to_string);

        if self.reject_invalid_codegen_module_types(module) {
            self.current_module_name = saved_module_name;
            return;
        }

        self.prescan_module_metadata(module);

        self.emit_module_constants(module, module_public);
        self.register_sifr_int_function_returns(module);
        self.emit_module_body(module, module_public, test_mode);

        self.current_module_name = saved_module_name;
    }

    pub(crate) fn try_lower_structured_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        self.try_lower_structured_stmt_with_following(stmt, None)
    }
}

pub fn body_contains_yield(stmts: &[HirStmt]) -> bool {
    hir_analysis::queries::body_contains_yield(stmts)
}
