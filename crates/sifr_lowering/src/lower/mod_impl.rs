use super::python_interop;
use super::{
    async_effects, collect_class_type, collect_function_defaults, collect_type_alias_decls,
    collect_type_vars, compiler_intrinsics, extract_function_type, function_body_contains_yield,
    import_diagnostics, import_resolution, imported_defaults, imports, integer_literal_diagnostics,
    lower_class, lower_function, module_constants_lowering, module_function_registry,
    name_diagnostics, parse_typevar_bound_expr, parse_typevar_declaration_specs,
    predeclare_type_aliases, private_stdlib_imports, register_builtins, resolve_imports_early,
    resolve_type_aliases, str, workload_annotations, Expr, ExternalDefs, FunctionType,
    HirDiagnostic, HirExpr, HirImport, HirModule, LowerCtx, Ranged, Stmt, TextRange, Type,
};
use sifr_ir::LoweringResult;
/// Internal implementation of module lowering.
pub(in crate::lower) fn lower_module_impl(
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
    // defined later in the file (e.g., LinkedNode, TreeNode, Node).
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
    python_interop::collect_python_opaque_classes(stmts, &mut ctx);
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
    let mut module_compiler_intrinsics = Vec::new();
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
            compiler_intrinsics::register_declaration(func, &mut ctx);
            if compiler_intrinsics::has_decorator_syntax(&func.decorator_list) {
                if let Some(intrinsic) = ctx.compiler_intrinsics.get(&function_name).copied() {
                    module_compiler_intrinsics.push((function_name.clone(), intrinsic));
                }
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

            let previous_owner = ctx.current_owner.replace(function_name.clone());
            let ft = extract_function_type(func, &mut ctx);
            ctx.current_owner = previous_owner;
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
            if python_interop::has_python_interop_decorator_syntax(&func.decorator_list) {
                let callback_policies = python_interop::callback_call_policies(
                    &func.decorator_list,
                    &func.parameters,
                    false,
                );
                if !callback_policies.is_empty() {
                    ctx.python_callback_call_policies
                        .insert(function_name.clone(), callback_policies);
                }
                ctx.python_call_shapes.insert(
                    function_name.clone(),
                    python_interop::python_parameter_kinds(&func.parameters),
                );
                let regular_count =
                    func.parameters.args.len() + usize::from(func.parameters.vararg.is_some());
                let omitted = func
                    .parameters
                    .args
                    .iter()
                    .enumerate()
                    .chain(
                        func.parameters
                            .kwonlyargs
                            .iter()
                            .enumerate()
                            .map(|(index, parameter)| (regular_count + index, parameter)),
                    )
                    .filter(|(_, parameter)| {
                        parameter
                            .default
                            .as_deref()
                            .is_some_and(python_interop::is_python_omit)
                    })
                    .filter_map(|(index, _)| {
                        ft.params.get(index).map(|(_, ty, _)| {
                            (
                                index,
                                HirExpr::Call {
                                    func: "__sifr_python_omitted_argument".to_string(),
                                    args: Vec::new(),
                                    ty: ty.clone(),
                                },
                            )
                        })
                    })
                    .collect::<Vec<_>>();
                if !omitted.is_empty() {
                    ctx.function_defaults
                        .entry(function_name.clone())
                        .or_default()
                        .extend(omitted);
                }
            }
            if func.is_async && function_body_contains_yield(&func.body) {
                ctx.async_generator_functions.insert(function_name.clone());
            } else if func.is_async {
                ctx.async_functions.insert(function_name.clone());
            }
            if let Some(workload) =
                workload_annotations::annotation_for_decorators(func.decorator_list.iter())
            {
                ctx.function_workload_annotations
                    .insert(function_name.clone(), workload);
            } else if !func.is_async
                && python_interop::has_python_interop_decorator_syntax(&func.decorator_list)
            {
                ctx.function_workload_annotations.insert(
                    function_name.clone(),
                    workload_annotations::WorkloadKind::BlockingIo,
                );
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
            let module_range = module.range();
            let names: Vec<String> = import_from
                .names
                .iter()
                .map(|alias| alias.name.to_string())
                .collect();
            let imported_names = import_from
                .names
                .iter()
                .map(|alias| {
                    alias.asname.as_ref().map_or_else(
                        || alias.name.to_string(),
                        |asname| format!("{} as {asname}", alias.name),
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
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

            // Private stdlib declaration imports are source-origin gated.
            // `_sifr.*` is the canonical private sysroot namespace, not the trust boundary.
            if is_absolute_import && module_name.starts_with("_sifr.") {
                if !ctx.can_import_private_stdlib_declarations() {
                    import_diagnostics::forbidden_intrinsic(&mut ctx, &module_name, import_range);
                    continue;
                }
                if private_stdlib_imports::resolve_compiled_private_imports(
                    &mut ctx,
                    externals,
                    &module_name,
                    &names,
                    &aliases,
                    &imported_name_range,
                ) {
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
                        if stdlib_module_key == "sifr.collections" && name == "defaultdict" {
                            ctx.explicit_defaultdict_bindings.insert(local.clone());
                            found = true;
                        }
                        if stdlib_module_key == "sifr.parallel" {
                            match name.as_str() {
                                "map" => {
                                    ctx.parallel_map_bindings.insert(local.clone());
                                }
                                "try_map" => {
                                    ctx.parallel_try_map_bindings.insert(local.clone());
                                }
                                _ => {}
                            }
                        }
                        if stdlib_module_key == "sifr.python" && name == "import_module" {
                            ctx.python_import_module_bindings.insert(local.clone());
                        }
                        // Check functions
                        if !found {
                            if let Some(module_fns) = externals.functions.get(&stdlib_module_key) {
                                if let Some(ft) = module_fns.get(name) {
                                    ctx.functions.insert(local.clone(), ft.clone());
                                    if let Some(module_intrinsics) =
                                        externals.compiler_intrinsics.get(&stdlib_module_key)
                                    {
                                        imported_defaults::import_callable_compiler_intrinsic(
                                            &mut ctx,
                                            module_intrinsics,
                                            name,
                                            &local,
                                        );
                                    }
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
                                    if let Some(module_shapes) = externals
                                        .function_python_call_shapes
                                        .get(&stdlib_module_key)
                                    {
                                        imported_defaults::import_python_call_shape(
                                            &mut ctx,
                                            module_shapes,
                                            name,
                                            &local,
                                        );
                                    }
                                    if let Some(module_workloads) =
                                        externals.function_workloads.get(&stdlib_module_key)
                                    {
                                        imported_defaults::import_callable_workload(
                                            &mut ctx,
                                            module_workloads,
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
                                        if let Some(module_workloads) =
                                            externals.function_workloads.get(&stdlib_module_key)
                                        {
                                            imported_defaults::import_class_method_workloads(
                                                &mut ctx,
                                                module_workloads,
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
                imports::report_unknown_stdlib_module(
                    &mut ctx,
                    &module_name,
                    &imported_names,
                    import_range,
                );
                continue;
            }

            // Check if the local module exists in externals before resolving
            let has_local_module =
                import_resolution::external_module_exists(externals, &module_name);
            if !has_local_module {
                if is_absolute_import {
                    if let Some(stdlib_match) =
                        sifr_stdlib_imports::is_bare_stdlib_tail(&module_name)
                    {
                        import_diagnostics::bare_stdlib(
                            &mut ctx,
                            &stdlib_match,
                            &imported_names,
                            module_range,
                        );
                        continue;
                    }
                }
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
                        if let Some(module_intrinsics) =
                            externals.compiler_intrinsics.get(&module_name)
                        {
                            imported_defaults::import_callable_compiler_intrinsic(
                                &mut ctx,
                                module_intrinsics,
                                name,
                                &local,
                            );
                        }
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
                        if let Some(module_shapes) =
                            externals.function_python_call_shapes.get(&module_name)
                        {
                            imported_defaults::import_python_call_shape(
                                &mut ctx,
                                module_shapes,
                                name,
                                &local,
                            );
                        }
                        if let Some(module_workloads) =
                            externals.function_workloads.get(&module_name)
                        {
                            imported_defaults::import_callable_workload(
                                &mut ctx,
                                module_workloads,
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
                                if let Some(module_workloads) =
                                    externals.function_workloads.get(&module_name)
                                {
                                    imported_defaults::import_class_method_workloads(
                                        &mut ctx,
                                        module_workloads,
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
                if let Some(stdlib_match) = sifr_stdlib_imports::is_bare_stdlib_tail(&module_name) {
                    import_diagnostics::bare_stdlib(
                        &mut ctx,
                        &stdlib_match,
                        "",
                        alias.name.range(),
                    );
                    continue;
                }
                import_diagnostics::unsupported_form(
                    &mut ctx,
                    format!("import {module_name}; use 'from {module_name} import <name>'")
                        .as_str(),
                    alias.name.range(),
                );
            }
        }
    }
    // The early import pass makes imported callable metadata available to signatures, while the
    // full import pass above records the final import list. Module function declarations shadow
    // those imported bindings for lowering purposes, so discard any imported intrinsic identity
    // that was reintroduced under a locally declared name and then restore canonical declarations.
    for stmt in stmts {
        if let Stmt::FunctionDef(func) = stmt {
            ctx.compiler_intrinsics.remove(func.name.as_str());
        }
    }
    ctx.compiler_intrinsics.extend(module_compiler_intrinsics);
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
    python_interop::validate_retained_callback_owner_errors(&functions, &classes, &mut ctx);
    if ctx.errors.is_empty() {
        let module = HirModule {
            functions,
            classes,
            imports,
            constants,
            generic_functions: ctx.generic_functions.clone(),
            type_param_bounds: ctx.type_param_bounds.clone(),
        };
        let flow_graph = crate::flow_graph::build_module_flow_graph(&module, &ctx.flow_effects);
        Ok(LoweringResult {
            module,
            flow_graph,
            function_defaults: ctx.function_defaults.clone(),
            function_varargs: ctx.vararg_functions.clone(),
            function_python_call_shapes: ctx.python_call_shapes.clone(),
            function_workloads: ctx
                .function_workload_annotations
                .iter()
                .map(|(name, workload)| (name.clone(), workload.label().to_string()))
                .collect(),
            constant_integer_values: ctx.const_integer_values.clone(),
            reveal_types: ctx.reveal_types,
            warnings: ctx.warnings,
        })
    } else {
        Err(ctx.errors)
    }
}
