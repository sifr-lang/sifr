//! The required string representation shared by error declarations and constructors.
use super::{HirExpr, HirFunction, HirParam, HirStmt, LowerCtx, MethodKind, Stmt, Type};
use ruff_text_size::Ranged;
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::StmtClassDef;

pub(in crate::lower) fn root_error_type() -> Type {
    Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "Error".to_string(),
        fields: vec![("message".to_string(), Type::Str)],
        methods: Vec::new(),
        parent_class: None,
    }
}

pub(super) fn seed(class: &StmtClassDef, fields: &mut Vec<(String, Type)>) {
    let explicitly_declared = class.body.iter().any(|stmt| matches!(stmt,
        Stmt::AnnAssign(ann) if matches!(ann.target.as_ref(), sifr_python_ast::Expr::Name(name) if name.id.as_str() == "message")));
    if !explicitly_declared && !fields.iter().any(|(name, _)| name == "message") {
        fields.push(("message".to_string(), Type::Str));
    }
}

pub(super) fn collect(
    class: &StmtClassDef,
    fields: &mut Vec<(String, Type)>,
    defaults: &mut Vec<(usize, HirExpr)>,
    ancestry: &mut Option<String>,
    ctx: &mut LowerCtx,
) {
    // An explicit Error marker also applies when the data parent is not an error.
    if let Some(chain) = ancestry {
        if !chain
            .split('|')
            .any(|parent| matches!(parent, "Error" | "sifr.builtin.Error"))
        {
            chain.push_str("|Error");
        }
    }
    if let Some(index) = fields.iter().position(|(name, _)| name == "message") {
        if !matches!(fields[index].1.resolve_alias(), Type::Str) {
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_MISMATCH,
                format!(
                    "error class '{}': message must have type 'str', got '{}'",
                    class.name,
                    fields[index].1.display_name()
                ),
                class.name.range(),
            );
        }
        if defaults.iter().any(|(field, _)| *field == index) {
            ctx.error_with_code_at(
                DiagnosticCode::CLASS_UNSUPPORTED_DECLARATION,
                format!("error class '{}': message must be supplied by the constructor, not a field default", class.name),
                class.name.range(),
            );
            defaults.retain(|(field, _)| *field != index);
        }
    } else {
        // Preserve explicit layouts (especially PythonError); only absent storage
        // gains the inherited root field, before the additional payload fields.
        fields.insert(0, ("message".to_string(), Type::Str));
        for (index, _) in defaults {
            *index += 1;
        }
    }
}

pub(super) fn inherit_constructor(
    class: &StmtClassDef,
    fields: &[(String, Type)],
    is_error: bool,
    ctx: &mut LowerCtx,
) {
    let explicit_constructor = class
        .body
        .iter()
        .any(|stmt| matches!(stmt, Stmt::FunctionDef(f) if f.name.as_str() == "__init__"));
    if !explicit_constructor
        && ctx
            .functions
            .get(class.name.as_str())
            .is_some_and(|function| function.return_type.is_builtin_error_base())
    {
        ctx.functions.remove(class.name.as_str());
        ctx.function_defaults.remove(class.name.as_str());
    }
    if !is_error || explicit_constructor {
        return;
    }
    // Collection repeats after annotations resolve. Do not retain the provisional
    // field-derived signature, or a colliding builtin constructor signature.
    ctx.functions.remove(class.name.as_str());
    ctx.function_defaults.remove(class.name.as_str());
    let Some(parent) =
        super::super::descriptor_declarations::data_parent_name(class.name.as_str(), ctx)
    else {
        return;
    };
    let Some(Type::Class {
        fields: parent_fields,
        ..
    }) = ctx.class_types.get(&parent)
    else {
        return;
    };
    if fields != parent_fields {
        return; // ordinary missing-initializer diagnostic owns additional fields
    }
    if let Some(constructor) = ctx.functions.get(&parent).cloned() {
        let defaults = ctx
            .function_defaults
            .get(&parent)
            .cloned()
            .unwrap_or_default();
        if !constructor
            .params
            .iter()
            .enumerate()
            .any(|(index, (_, ty, _))| {
                matches!(ty.resolve_alias(), Type::Str)
                    && !defaults.iter().any(|(default, _)| *default == index)
            })
        {
            ctx.error_with_code_at(
                DiagnosticCode::CLASS_MISSING_INITIALIZER,
                format!("error class '{}': inherited constructor requires a caller-supplied 'str' parameter for its message", class.name),
                class.name.range(),
            );
        }
        ctx.functions.insert(class.name.to_string(), constructor);
        if !defaults.is_empty() {
            ctx.function_defaults
                .insert(class.name.to_string(), defaults);
        }
    }
}

pub(super) fn lower_root_initialization(body: &mut [HirStmt]) {
    for stmt in body {
        if let HirStmt::Expr {
            expr:
                HirExpr::SuperCall {
                    parent_type,
                    method,
                    args,
                    ..
                },
        } = stmt
        {
            if parent_type.is_builtin_error_base() && method == "new" && args.len() == 1 {
                *stmt = HirStmt::FieldAssign {
                    object: "self".to_string(),
                    field: "message".to_string(),
                    field_ty: Type::Str,
                    value: args[0].clone(),
                };
            }
        }
    }
}

pub(super) fn validate_constructor(
    class: &StmtClassDef,
    function: &sifr_python_ast::StmtFunctionDef,
    body: &[HirStmt],
    own_fields: &[(String, Type)],
    params: &[HirParam],
    requires_parent: bool,
    ctx: &mut LowerCtx,
) {
    if !params
        .iter()
        .any(|param| matches!(param.ty.resolve_alias(), Type::Str) && param.default.is_none())
    {
        ctx.error_with_code_at(
            DiagnosticCode::CLASS_MISSING_INITIALIZER,
            format!("error class '{}': constructor requires a caller-supplied 'str' parameter for its message", class.name),
            function.name.range(),
        );
    }
    // A final materialization uses self even if the source body does not. The
    // same storage proof used for early self access must hold at that boundary.
    let mut materialization = body.to_vec();
    materialization.push(HirStmt::Expr {
        expr: HirExpr::Name {
            name: "self".to_string(),
            binding_id: None,
            ty: Type::None,
        },
    });
    if let Some(gap) = super::constructor_uninitialized_storage_at_first_self_use(
        &materialization,
        own_fields,
        params,
        requires_parent,
    ) {
        if gap.statement_index == body.len() {
            ctx.error_with_code_at(
                DiagnosticCode::CLASS_MISSING_INITIALIZER,
                format!(
                    "error class '{}': constructor must initialize required storage: {}{}",
                    class.name,
                    if gap.missing_parent {
                        "super().__init__(...), "
                    } else {
                        ""
                    },
                    gap.missing_fields.join(", ")
                ),
                function.name.range(),
            );
        }
    }
    if own_fields.iter().any(|(field, _)| field == "message") {
        if let Some(param) = params.iter().find(|param| param.name == "message") {
            if !matches!(param.ty.resolve_alias(), Type::Str) || param.default.is_some() {
                ctx.error_with_code_at(
                    DiagnosticCode::TYPE_MISMATCH,
                    format!(
                        "error class '{}': constructor message parameter must be a required 'str'",
                        class.name
                    ),
                    function.name.range(),
                );
            }
        }
    }
}

pub(super) fn inherited_constructor(
    class: &StmtClassDef,
    parent_name: &str,
    parent_type: &Type,
    ctx: &LowerCtx,
) -> Option<HirFunction> {
    let constructor = ctx.functions.get(class.name.as_str())?;
    let defaults = ctx.function_defaults.get(class.name.as_str());
    let params: Vec<_> = constructor
        .params
        .iter()
        .enumerate()
        .map(|(index, (name, ty, convention))| HirParam {
            name: name.clone(),
            ty: ty.clone(),
            convention: *convention,
            keyword_only: false,
            default: defaults
                .and_then(|defaults| defaults.iter().find(|(i, _)| *i == index))
                .map(|(_, value)| value.clone()),
        })
        .collect();
    Some(HirFunction {
        name: "new".to_string(),
        body: vec![HirStmt::Expr {
            expr: HirExpr::SuperCall {
                parent_class: parent_name.to_string(),
                parent_type: parent_type.clone(),
                method: "new".to_string(),
                args: params
                    .iter()
                    .map(|param| HirExpr::Name {
                        name: param.name.clone(),
                        binding_id: None,
                        ty: param.ty.clone(),
                    })
                    .collect(),
                ty: parent_type.clone(),
            },
        }],
        params,
        return_type: Type::None,
        is_async: false,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: Vec::new(),
        rust_interop: Vec::new(),
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: Vec::new(),
    })
}
