use crate::SqlEditorDocumentView;
use crate::sql_schema_polymorphism::witness_type_parameter;
use sifr_ir::{
    HirExpr, HirFunction, HirModule, HirStmt, visit_hir_function_exprs_mut,
    visit_hir_stmts_exprs_mut,
};
use std::collections::{BTreeMap, BTreeSet, HashMap};

#[derive(Clone, Debug)]
pub struct PortableSqlQueryDeclaration {
    pub symbol: String,
    pub requirement_name: String,
    pub witness_parameter_index: usize,
    pub witness_type_parameter_index: usize,
    pub exported: bool,
    pub document: SqlEditorDocumentView,
    pub parameter_types: Vec<sifr_sql_contract::SifrType>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PortableSqlQuerySpecialization {
    pub symbol: String,
    pub requirement_name: String,
    pub profile_name: String,
    pub specialized_symbol: String,
    pub witness_parameter_index: usize,
    pub witness_type_parameter_index: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ImportedPortableSqlQuerySpecialization {
    pub owner_module: String,
    pub local_symbol: String,
    pub specialization: PortableSqlQuerySpecialization,
}

#[must_use]
pub fn sql_profile_local_names(
    module: &HirModule,
    profiles: &BTreeSet<String>,
) -> BTreeMap<String, String> {
    imported_namespace_names(module, "sifr.sql.schemas", profiles)
}

/// Discover portable query functions and their concrete, statically selected
/// profile uses. A portable function owns exactly one `SqlSchema[T]` parameter
/// and exactly one `schema.sql(template)` expression. This deliberately keeps
/// the compile-time query constructor closed and auditable.
pub fn portable_sql_query_plan(
    module: &HirModule,
    profiles: &BTreeSet<String>,
) -> Result<
    (
        Vec<PortableSqlQueryDeclaration>,
        Vec<PortableSqlQuerySpecialization>,
    ),
    String,
> {
    let profile_locals = imported_namespace_names(module, "sifr.sql.schemas", profiles);
    let requirement_locals =
        imported_namespace_names(module, "sifr.sql.requirements", &BTreeSet::new());
    let mut declarations = Vec::new();
    for function in &module.functions {
        let witness_parameters = function
            .params
            .iter()
            .enumerate()
            .filter_map(|(index, parameter)| {
                witness_type_parameter(&parameter.ty)
                    .map(|type_parameter| (index, parameter.name.as_str(), type_parameter))
            })
            .collect::<Vec<_>>();
        if witness_parameters.is_empty() {
            continue;
        }
        if witness_parameters.len() != 1 {
            return Err(format!(
                "portable SQL function '{}' must have exactly one SqlSchema witness parameter",
                function.name
            ));
        }
        let (witness_parameter_index, witness_name, type_parameter) = witness_parameters[0];
        let witness_type_parameter_index = function
            .type_params
            .iter()
            .position(|candidate| candidate == type_parameter)
            .ok_or_else(|| {
                format!(
                    "portable SQL function '{}' has an undeclared witness type parameter",
                    function.name
                )
            })?;
        let requirement_reference = module
            .type_param_bounds
            .get(&function.name)
            .and_then(|bounds| bounds.get(type_parameter))
            .and_then(|bounds| {
                let candidates = bounds
                    .iter()
                    .filter_map(|bound| bound.strip_suffix(".Schema"))
                    .collect::<Vec<_>>();
                (candidates.len() == 1).then_some(candidates[0])
            })
            .ok_or_else(|| {
                format!(
                    "portable SQL function '{}' must bind its witness to exactly one schema requirement",
                    function.name
                )
            })?;
        let requirement_name = requirement_locals
            .get(requirement_reference)
            .cloned()
            .unwrap_or_else(|| requirement_reference.to_string());

        let mut clone = function.clone();
        let mut witness_references = 0usize;
        let mut templates = Vec::new();
        visit_hir_function_exprs_mut(&mut clone, &mut |expression| {
            if matches!(expression, HirExpr::Name { name, .. } if name == witness_name) {
                witness_references += 1;
            }
            if let HirExpr::MethodCall {
                object,
                method,
                args,
                ..
            } = expression
                && method == "sql"
                && matches!(object.as_ref(), HirExpr::Name { name, .. } if name == witness_name)
                && let [HirExpr::TemplateString(template)] = args.as_slice()
            {
                templates.push(template.clone());
            }
        });
        if witness_references != 1 || templates.len() != 1 {
            return Err(format!(
                "portable SQL function '{}' may use its witness only once as schema.sql(typed_template)",
                function.name
            ));
        }
        let template = templates.remove(0);
        let parameter_types = template
            .interpolations
            .iter()
            .map(|interpolation| {
                crate::sql_query_source::sql_contract_type(&interpolation.value_type)
            })
            .collect::<Result<Vec<_>, _>>()?;
        declarations.push(PortableSqlQueryDeclaration {
            symbol: function.name.clone(),
            requirement_name,
            witness_parameter_index,
            witness_type_parameter_index,
            exported: !function.name.starts_with('_'),
            document: SqlEditorDocumentView::from_hir(&template),
            parameter_types,
        });
    }

    let by_symbol = declarations
        .iter()
        .map(|declaration| (declaration.symbol.as_str(), declaration))
        .collect::<BTreeMap<_, _>>();
    let mut selected = BTreeSet::new();
    let mut inspect = |expression: &mut HirExpr| {
        let (func, args) = match expression {
            HirExpr::Call { func, args, .. } | HirExpr::GenericCall { func, args, .. } => {
                (func.as_str(), args.as_slice())
            }
            _ => return,
        };
        let Some(declaration) = by_symbol.get(func) else {
            return;
        };
        let Some(argument) = args.get(declaration.witness_parameter_index) else {
            return;
        };
        let Some(local_profile) = direct_profile_witness(argument) else {
            return;
        };
        let Some(profile_name) = profile_locals.get(local_profile) else {
            return;
        };
        selected.insert((declaration.symbol.clone(), profile_name.clone()));
    };
    let mut inspected = module.clone();
    for function in &mut inspected.functions {
        visit_hir_function_exprs_mut(function, &mut inspect);
    }
    for class in &mut inspected.classes {
        for method in &mut class.methods {
            visit_hir_function_exprs_mut(method, &mut inspect);
        }
        for (_, method) in &mut class.operator_impls {
            visit_hir_function_exprs_mut(method, &mut inspect);
        }
    }
    let mut constants = inspected
        .constants
        .iter()
        .map(|(_, _, value)| HirStmt::Expr {
            expr: value.clone(),
        })
        .collect::<Vec<_>>();
    visit_hir_stmts_exprs_mut(&mut constants, &mut inspect);

    let mut specializations = Vec::new();
    for declaration in &declarations {
        for profile_name in selected
            .iter()
            .filter_map(|(symbol, profile)| (symbol == &declaration.symbol).then_some(profile))
        {
            specializations.push(PortableSqlQuerySpecialization {
                symbol: declaration.symbol.clone(),
                requirement_name: declaration.requirement_name.clone(),
                profile_name: profile_name.clone(),
                specialized_symbol: specialized_symbol(&declaration.symbol, profile_name),
                witness_parameter_index: declaration.witness_parameter_index,
                witness_type_parameter_index: declaration.witness_type_parameter_index,
            });
        }
    }
    validate_all_portable_calls(&mut inspected, &by_symbol, &profile_locals)?;
    Ok((declarations, specializations))
}

/// Discover portable queries imported from another project module. The
/// defining module owns the generated function and contract; this module gets
/// a compiler-generated import and a rewritten call.
pub fn imported_portable_sql_query_plan(
    module: &HirModule,
    declarations: &BTreeMap<String, Vec<PortableSqlQueryDeclaration>>,
    profiles: &BTreeSet<String>,
) -> Result<Vec<ImportedPortableSqlQuerySpecialization>, String> {
    let profile_locals = imported_namespace_names(module, "sifr.sql.schemas", profiles);
    let mut imported = BTreeMap::<String, (String, &PortableSqlQueryDeclaration)>::new();
    for import in &module.imports {
        let Some(owned) = declarations.get(&import.module) else {
            continue;
        };
        for original in &import.names {
            let Some(declaration) = owned
                .iter()
                .find(|declaration| declaration.symbol == *original)
            else {
                continue;
            };
            let local = import
                .aliases
                .iter()
                .find_map(|(name, alias)| (name == original).then_some(alias))
                .unwrap_or(original);
            imported.insert(local.clone(), (import.module.clone(), declaration));
        }
    }
    if imported.is_empty() {
        return Ok(Vec::new());
    }
    let mut output = BTreeSet::new();
    let mut invalid = None;
    let mut inspect = |expression: &mut HirExpr| {
        let (function, args) = match expression {
            HirExpr::Call { func, args, .. } | HirExpr::GenericCall { func, args, .. } => {
                (func.as_str(), args.as_slice())
            }
            _ => return,
        };
        let Some((owner_module, declaration)) = imported.get(function) else {
            return;
        };
        let profile = args
            .get(declaration.witness_parameter_index)
            .and_then(direct_profile_witness)
            .and_then(|local| profile_locals.get(local));
        let Some(profile) = profile else {
            invalid = Some(format!(
                "imported portable SQL function '{function}' requires one direct generated profile.schema argument"
            ));
            return;
        };
        output.insert(ImportedPortableSqlQuerySpecialization {
            owner_module: owner_module.clone(),
            local_symbol: function.to_string(),
            specialization: PortableSqlQuerySpecialization {
                symbol: declaration.symbol.clone(),
                requirement_name: declaration.requirement_name.clone(),
                profile_name: profile.clone(),
                specialized_symbol: specialized_symbol(&declaration.symbol, profile),
                witness_parameter_index: declaration.witness_parameter_index,
                witness_type_parameter_index: declaration.witness_type_parameter_index,
            },
        });
    };
    let mut inspected = module.clone();
    visit_module_expressions(&mut inspected, &mut inspect);
    invalid.map_or_else(|| Ok(output.into_iter().collect()), Err)
}

pub fn validate_profile_witness_consumption(
    module: &HirModule,
    local: &[PortableSqlQuerySpecialization],
    imported: &[ImportedPortableSqlQuerySpecialization],
    profiles: &BTreeSet<String>,
) -> Result<(), String> {
    let profile_locals = imported_namespace_names(module, "sifr.sql.schemas", profiles);
    let mut direct_witnesses = 0usize;
    let mut valid_arguments = 0usize;
    let local_calls = local
        .iter()
        .map(|item| (item.symbol.as_str(), item.witness_parameter_index))
        .chain(imported.iter().map(|item| {
            (
                item.local_symbol.as_str(),
                item.specialization.witness_parameter_index,
            )
        }))
        .collect::<BTreeMap<_, _>>();
    let mut inspect = |expression: &mut HirExpr| {
        if direct_profile_witness(expression).is_some_and(|name| profile_locals.contains_key(name))
        {
            direct_witnesses += 1;
        }
        let (function, args) = match expression {
            HirExpr::Call { func, args, .. } | HirExpr::GenericCall { func, args, .. } => {
                (func.as_str(), args.as_slice())
            }
            _ => return,
        };
        let Some(index) = local_calls.get(function) else {
            return;
        };
        if args
            .get(*index)
            .and_then(direct_profile_witness)
            .is_some_and(|name| profile_locals.contains_key(name))
        {
            valid_arguments += 1;
        }
    };
    let mut inspected = module.clone();
    visit_module_expressions(&mut inspected, &mut inspect);
    if direct_witnesses != valid_arguments {
        return Err(
            "profile.schema is compile-time-only and must be the direct witness argument of a portable SQL query"
                .to_string(),
        );
    }
    Ok(())
}

pub fn apply_imported_portable_sql_query_plan(
    module: &mut HirModule,
    specializations: &[ImportedPortableSqlQuerySpecialization],
) -> Result<(), String> {
    if specializations.is_empty() {
        return Ok(());
    }
    let profile_names = specializations
        .iter()
        .map(|item| item.specialization.profile_name.clone())
        .collect::<BTreeSet<_>>();
    let profile_locals = imported_namespace_names(module, "sifr.sql.schemas", &profile_names);
    let mut rewrites = BTreeMap::new();
    for item in specializations {
        let key = (
            item.local_symbol.clone(),
            item.specialization.profile_name.clone(),
        );
        if rewrites.insert(key, &item.specialization).is_some() {
            return Err("duplicate imported portable SQL specialization".to_string());
        }
        let import = module
            .imports
            .iter_mut()
            .find(|import| import.module == item.owner_module)
            .ok_or_else(|| "portable SQL specialization lost its source import".to_string())?;
        if !import
            .names
            .contains(&item.specialization.specialized_symbol)
        {
            import
                .names
                .push(item.specialization.specialized_symbol.clone());
        }
    }
    let mut rewrite = |expression: &mut HirExpr| {
        rewrite_imported_specialized_call(expression, &rewrites, &profile_locals);
    };
    visit_module_expressions(module, &mut rewrite);
    Ok(())
}

/// Materialize every concrete portable-query specialization and erase the
/// generic definition. The specialization names are compiler-reserved and
/// stable for the source function/profile pair.
pub fn apply_portable_sql_query_plan(
    module: &mut HirModule,
    declarations: &[PortableSqlQueryDeclaration],
    specializations: &[PortableSqlQuerySpecialization],
) -> Result<(), String> {
    let mut by_function = declarations
        .iter()
        .map(|declaration| (declaration.symbol.as_str(), Vec::new()))
        .collect::<BTreeMap<&str, Vec<&PortableSqlQuerySpecialization>>>();
    for specialization in specializations {
        by_function
            .entry(&specialization.symbol)
            .or_default()
            .push(specialization);
    }
    if by_function.is_empty() {
        return Ok(());
    }
    let existing = module
        .functions
        .iter()
        .map(|function| function.name.clone())
        .collect::<BTreeSet<_>>();
    let originals = module
        .functions
        .iter()
        .filter(|function| by_function.contains_key(function.name.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let mut generated = Vec::new();
    for original in &originals {
        let Some(instances) = by_function.get(original.name.as_str()) else {
            continue;
        };
        for specialization in instances {
            if existing.contains(&specialization.specialized_symbol) {
                return Err(format!(
                    "compiler SQL specialization '{}' collides with a source function",
                    specialization.specialized_symbol
                ));
            }
            let mut function = original.clone();
            function.name.clone_from(&specialization.specialized_symbol);
            function
                .params
                .remove(specialization.witness_parameter_index);
            function
                .type_params
                .remove(specialization.witness_type_parameter_index);
            erase_witness_method(&mut function);
            let remaining_type_params = function.type_params.clone();
            generated.push(function);

            if let Some(bounds) = module.type_param_bounds.get(&original.name).cloned() {
                let witness_type_parameter =
                    original.type_params[specialization.witness_type_parameter_index].as_str();
                let filtered = bounds
                    .into_iter()
                    .filter(|(name, _)| name != witness_type_parameter)
                    .collect::<HashMap<_, _>>();
                if !filtered.is_empty() {
                    module
                        .type_param_bounds
                        .insert(specialization.specialized_symbol.clone(), filtered);
                }
            }
            if !remaining_type_params.is_empty() {
                module.generic_functions.insert(
                    specialization.specialized_symbol.clone(),
                    remaining_type_params,
                );
            }
        }
    }

    let by_call = specializations
        .iter()
        .map(|specialization| {
            (
                (
                    specialization.symbol.as_str(),
                    specialization.profile_name.as_str(),
                ),
                specialization,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let profile_locals = imported_namespace_names(
        module,
        "sifr.sql.schemas",
        &specializations
            .iter()
            .map(|specialization| specialization.profile_name.clone())
            .collect(),
    );
    let mut rewrite = |expression: &mut HirExpr| {
        rewrite_specialized_call(expression, &by_call, &profile_locals);
    };
    for function in &mut module.functions {
        visit_hir_function_exprs_mut(function, &mut rewrite);
    }
    for class in &mut module.classes {
        for method in &mut class.methods {
            visit_hir_function_exprs_mut(method, &mut rewrite);
        }
        for (_, method) in &mut class.operator_impls {
            visit_hir_function_exprs_mut(method, &mut rewrite);
        }
    }
    let mut constants = module
        .constants
        .drain(..)
        .map(|(name, ty, value)| (name, ty, HirStmt::Expr { expr: value }))
        .collect::<Vec<_>>();
    for (_, _, statement) in &mut constants {
        visit_hir_stmts_exprs_mut(std::slice::from_mut(statement), &mut rewrite);
    }
    module.constants = constants
        .into_iter()
        .filter_map(|(name, ty, statement)| match statement {
            HirStmt::Expr { expr } => Some((name, ty, expr)),
            _ => None,
        })
        .collect();
    module
        .functions
        .retain(|function| !by_function.contains_key(function.name.as_str()));
    module.functions.extend(generated);
    for original in by_function.keys() {
        module.generic_functions.remove(*original);
        module.type_param_bounds.remove(*original);
    }
    Ok(())
}

fn imported_namespace_names(
    module: &HirModule,
    namespace: &str,
    known: &BTreeSet<String>,
) -> BTreeMap<String, String> {
    let mut names = BTreeMap::new();
    for import in module
        .imports
        .iter()
        .filter(|import| import.module == namespace)
    {
        for original in &import.names {
            let local = import
                .aliases
                .iter()
                .find_map(|(name, alias)| (name == original).then_some(alias))
                .unwrap_or(original);
            if known.is_empty() || known.contains(original) {
                names.insert(local.clone(), original.clone());
            }
        }
    }
    names
}

fn visit_module_expressions<F>(module: &mut HirModule, visit: &mut F)
where
    F: FnMut(&mut HirExpr),
{
    for function in &mut module.functions {
        visit_hir_function_exprs_mut(function, visit);
    }
    for class in &mut module.classes {
        for method in &mut class.methods {
            visit_hir_function_exprs_mut(method, visit);
        }
        for (_, method) in &mut class.operator_impls {
            visit_hir_function_exprs_mut(method, visit);
        }
    }
    for (_, _, value) in &mut module.constants {
        let mut statement = HirStmt::Expr {
            expr: value.clone(),
        };
        visit_hir_stmts_exprs_mut(std::slice::from_mut(&mut statement), visit);
        if let HirStmt::Expr { expr } = statement {
            *value = expr;
        }
    }
}

fn direct_profile_witness(expression: &HirExpr) -> Option<&str> {
    let HirExpr::FieldAccess { object, field, .. } = expression else {
        return None;
    };
    if field != "schema" {
        return None;
    }
    match object.as_ref() {
        HirExpr::Name { name, .. } => Some(name),
        _ => None,
    }
}

fn validate_all_portable_calls(
    module: &mut HirModule,
    declarations: &BTreeMap<&str, &PortableSqlQueryDeclaration>,
    profiles: &BTreeMap<String, String>,
) -> Result<(), String> {
    let mut invalid = None;
    let mut inspect = |expression: &mut HirExpr| {
        let (func, args) = match expression {
            HirExpr::Call { func, args, .. } | HirExpr::GenericCall { func, args, .. } => {
                (func.as_str(), args.as_slice())
            }
            _ => return,
        };
        let Some(declaration) = declarations.get(func) else {
            return;
        };
        if args
            .get(declaration.witness_parameter_index)
            .and_then(direct_profile_witness)
            .is_none_or(|name| !profiles.contains_key(name))
        {
            invalid = Some(format!(
                "portable SQL function '{}' requires one direct generated profile.schema argument",
                declaration.symbol
            ));
        }
    };
    for function in &mut module.functions {
        visit_hir_function_exprs_mut(function, &mut inspect);
    }
    for class in &mut module.classes {
        for method in &mut class.methods {
            visit_hir_function_exprs_mut(method, &mut inspect);
        }
        for (_, method) in &mut class.operator_impls {
            visit_hir_function_exprs_mut(method, &mut inspect);
        }
    }
    let mut constants = module
        .constants
        .iter()
        .map(|(_, _, value)| HirStmt::Expr {
            expr: value.clone(),
        })
        .collect::<Vec<_>>();
    visit_hir_stmts_exprs_mut(&mut constants, &mut inspect);
    invalid.map_or(Ok(()), Err)
}

fn specialized_symbol(function: &str, profile: &str) -> String {
    let encoded = lower_hex(profile.as_bytes());
    format!("__sifr_sql_{function}_{encoded}")
}

fn lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

fn erase_witness_method(function: &mut HirFunction) {
    visit_hir_function_exprs_mut(function, &mut |expression| {
        let HirExpr::MethodCall {
            object,
            method,
            args,
            ..
        } = expression
        else {
            return;
        };
        if method == "sql" && witness_type_parameter(object.ty()).is_some() && args.len() == 1 {
            *expression = args.remove(0);
        }
    });
}

fn rewrite_specialized_call(
    expression: &mut HirExpr,
    specializations: &BTreeMap<(&str, &str), &PortableSqlQuerySpecialization>,
    profiles: &BTreeMap<String, String>,
) {
    match expression {
        HirExpr::Call {
            func,
            args,
            mutable_arg_places,
            ..
        } => {
            let Some((profile, witness_index)) =
                selected_profile(func, args, specializations, profiles)
            else {
                return;
            };
            let Some(specialization) = specializations.get(&(func.as_str(), profile.as_str()))
            else {
                return;
            };
            let specialized_symbol = specialization.specialized_symbol.clone();
            *func = specialized_symbol;
            args.remove(witness_index);
            if mutable_arg_places.len() > witness_index {
                mutable_arg_places.remove(witness_index);
            }
        }
        HirExpr::GenericCall {
            func,
            type_args,
            args,
            mutable_arg_places,
            ..
        } => {
            let Some((profile, witness_index)) =
                selected_profile(func, args, specializations, profiles)
            else {
                return;
            };
            let Some(specialization) = specializations.get(&(func.as_str(), profile.as_str()))
            else {
                return;
            };
            let specialized_symbol = specialization.specialized_symbol.clone();
            let witness_type_parameter_index = specialization.witness_type_parameter_index;
            *func = specialized_symbol;
            args.remove(witness_index);
            if mutable_arg_places.len() > witness_index {
                mutable_arg_places.remove(witness_index);
            }
            if type_args.len() > witness_type_parameter_index {
                type_args.remove(witness_type_parameter_index);
            }
        }
        _ => {}
    }
}

fn rewrite_imported_specialized_call(
    expression: &mut HirExpr,
    specializations: &BTreeMap<(String, String), &PortableSqlQuerySpecialization>,
    profiles: &BTreeMap<String, String>,
) {
    match expression {
        HirExpr::Call {
            func,
            args,
            mutable_arg_places,
            ..
        } => {
            let Some(specialization) =
                imported_specialization(func, args, specializations, profiles)
            else {
                return;
            };
            let witness_index = specialization.witness_parameter_index;
            *func = specialization.specialized_symbol.clone();
            args.remove(witness_index);
            if mutable_arg_places.len() > witness_index {
                mutable_arg_places.remove(witness_index);
            }
        }
        HirExpr::GenericCall {
            func,
            type_args,
            args,
            mutable_arg_places,
            ..
        } => {
            let Some(specialization) =
                imported_specialization(func, args, specializations, profiles)
            else {
                return;
            };
            let witness_index = specialization.witness_parameter_index;
            *func = specialization.specialized_symbol.clone();
            args.remove(witness_index);
            if mutable_arg_places.len() > witness_index {
                mutable_arg_places.remove(witness_index);
            }
            if type_args.len() > specialization.witness_type_parameter_index {
                type_args.remove(specialization.witness_type_parameter_index);
            }
        }
        _ => {}
    }
}

fn imported_specialization<'a>(
    function: &str,
    args: &[HirExpr],
    specializations: &'a BTreeMap<(String, String), &PortableSqlQuerySpecialization>,
    profiles: &BTreeMap<String, String>,
) -> Option<&'a PortableSqlQuerySpecialization> {
    let witness_index = specializations
        .iter()
        .find_map(|((local, _), specialization)| {
            (local == function).then_some(specialization.witness_parameter_index)
        })?;
    let local_profile = args.get(witness_index).and_then(direct_profile_witness)?;
    let profile = profiles.get(local_profile)?.clone();
    let specialization = specializations.get(&(function.to_string(), profile.clone()))?;
    Some(specialization)
}

fn selected_profile(
    function: &str,
    args: &[HirExpr],
    specializations: &BTreeMap<(&str, &str), &PortableSqlQuerySpecialization>,
    profiles: &BTreeMap<String, String>,
) -> Option<(String, usize)> {
    let witness_index = specializations
        .values()
        .find(|specialization| specialization.symbol == function)?
        .witness_parameter_index;
    let local = args.get(witness_index).and_then(direct_profile_witness)?;
    Some((profiles.get(local)?.clone(), witness_index))
}
