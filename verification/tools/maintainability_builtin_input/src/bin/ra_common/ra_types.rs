use hir::{GenericDef, GenericParam, Module, Type, TypeParam, db::HirDatabase};
use serde_json::{Value, json};
pub fn module(db: &dyn HirDatabase, mut module: Module) -> String {
    let mut names = vec![];
    while let Some(parent) = module.parent(db) {
        if let Some(name) = module.name(db) {
            names.push(name.as_str().to_string());
        }
        module = parent;
    }
    names.push(
        module
            .krate(db)
            .display_name(db)
            .expect("named selected crate")
            .to_string(),
    );
    names.reverse();
    names.join("::")
}
pub fn substitution<'db>(
    db: &'db dyn HirDatabase,
    implementation: hir::Impl,
) -> anyhow::Result<Vec<(TypeParam, Type<'db>)>> {
    let reference = implementation
        .trait_ref(db)
        .ok_or_else(|| anyhow::anyhow!("missing common trait ref"))?;
    let parameters = GenericDef::Trait(reference.trait_()).params(db);
    let mut result = vec![];
    for (index, parameter) in parameters.into_iter().enumerate() {
        if let GenericParam::TypeParam(parameter) = parameter {
            let ty = reference
                .get_type_argument(index)
                .ok_or_else(|| anyhow::anyhow!("missing actual common generic argument"))?;
            result.push((parameter, ty));
        }
    }
    Ok(result)
}
pub fn canonical(
    db: &dyn HirDatabase,
    ty: &Type<'_>,
    substitution: &[(TypeParam, Type<'_>)],
) -> anyhow::Result<Value> {
    if let Some((inner, mutable)) = ty.as_reference() {
        return Ok(
            json!({"reference":canonical(db,&inner,substitution)?,"mutable":mutable==hir::Mutability::Mut}),
        );
    }
    if let Some(adt) = ty.as_adt() {
        return Ok(
            json!({"adt":format!("{}::{}",module(db,adt.module(db)),adt.name(db).as_str()),"arguments":ty.type_arguments().map(|arg|canonical(db,&arg,substitution)).collect::<anyhow::Result<Vec<_>>>()?}),
        );
    }
    if ty.is_tuple() {
        return Ok(
            json!({"tuple":ty.tuple_fields(db).iter().map(|t|canonical(db,t,substitution)).collect::<anyhow::Result<Vec<_>>>()?}),
        );
    }
    if let Some(parameter) = ty.as_type_param(db) {
        if let Some((_, replacement)) = substitution.iter().find(|(p, _)| *p == parameter) {
            return canonical(db, replacement, &[]);
        }
        return Ok(json!({"parameter":parameter.name(db).as_str()}));
    }
    if ty.is_bool() {
        return Ok(json!({"builtin":"bool"}));
    }
    if let Some(integer) = ty.as_builtin() {
        return Ok(json!({"builtin":integer.name().as_str()}));
    }
    anyhow::bail!("unsupported canonical common type: {ty:?}")
}

pub fn bounds<'db>(db: &'db dyn HirDatabase, receiver: &Type<'db>) -> anyhow::Result<Vec<Value>> {
    let mut result = vec![];
    for argument in receiver
        .type_arguments()
        .filter(|arg| arg.as_type_param(db).is_some())
    {
        let mut bounds = vec![];
        for trait_ in argument.env_traits(db) {
            let parameters = GenericDef::Trait(trait_).params(db);
            let Some(GenericParam::TypeParam(self_parameter)) = parameters.first().copied() else {
                anyhow::bail!("missing trait Self parameter");
            };
            let mut arguments = vec![];
            for parameter in parameters.iter().skip(1) {
                let GenericParam::TypeParam(parameter) = parameter else {
                    anyhow::bail!("unsupported bound lifetime/constant");
                };
                let default = parameter.default(db).ok_or_else(|| {
                    anyhow::anyhow!("unexposed nondefault builtin bound argument")
                })?;
                anyhow::ensure!(
                    default.as_type_param(db) == Some(self_parameter),
                    "unsupported builtin bound default substitution"
                );
                arguments.push(argument.clone());
            }
            if !arguments.is_empty() {
                anyhow::ensure!(
                    argument.impls_trait(db, trait_, &arguments),
                    "unproven actual builtin generic bound arguments"
                );
            }
            let mut canonical_arguments = vec![canonical(db, &argument, &[])?];
            canonical_arguments.extend(
                arguments
                    .iter()
                    .map(|arg| canonical(db, arg, &[]))
                    .collect::<anyhow::Result<Vec<_>>>()?,
            );
            bounds.push(json!({"trait":format!("{}::{}",module(db,trait_.module(db)),trait_.name(db).as_str()),"arguments":canonical_arguments,"polarity":"Positive"}));
        }
        bounds.sort_by_cached_key(|bound| bound.to_string());
        bounds.dedup();
        result.push(json!({"parameter":canonical(db,&argument,&[])?,"traits":bounds}));
    }
    result.sort_by_cached_key(|bound| bound["parameter"].to_string());
    Ok(result)
}

pub fn builtin_trait(origin: &str) -> Option<&'static str> {
    match origin {
        "core::fmt::macros::Debug" => Some("core::fmt::Debug"),
        "core::clone::Clone" => Some("core::clone::Clone"),
        "core::marker::Copy" => Some("core::marker::Copy"),
        "core::cmp::PartialEq" => Some("core::cmp::PartialEq"),
        "core::cmp::Eq" => Some("core::cmp::Eq"),
        "core::cmp::Ord" => Some("core::cmp::Ord"),
        "core::cmp::PartialOrd" => Some("core::cmp::PartialOrd"),
        "core::default::Default" => Some("core::default::Default"),
        "core::hash::macros::Hash" => Some("core::hash::Hash"),
        _ => None,
    }
}
