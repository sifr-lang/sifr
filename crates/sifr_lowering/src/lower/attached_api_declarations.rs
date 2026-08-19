use super::{descriptor_declarations::malformed, Expr, LowerCtx, Ranged, Stmt, Type};
use sifr_ir::{
    AttachedApiDeclaration, AttachedApiReceiver, AttachedApiSetDeclaration, AttachedApiSetIdentity,
};
use sifr_python_ast::{Decorator, StmtClassDef, StmtFunctionDef};
use std::collections::HashSet;

pub(super) fn owner_type_param(function: &StmtFunctionDef) -> Option<String> {
    function.decorator_list.iter().find_map(|decorator| {
        let Expr::Call(call) = &decorator.expression else {
            return None;
        };
        if !matches!(call.func.as_ref(), Expr::Name(name) if name.id.as_str() == "attached_api") {
            return None;
        }
        keyword_string(call, "owner")
    })
}

pub(super) fn type_receiver_owner_type_param(function: &StmtFunctionDef) -> Option<String> {
    function.decorator_list.iter().find_map(|decorator| {
        let Expr::Call(call) = &decorator.expression else {
            return None;
        };
        if !matches!(call.func.as_ref(), Expr::Name(name) if name.id.as_str() == "attached_api")
            || keyword_string(call, "receiver").as_deref() != Some("type")
        {
            return None;
        }
        keyword_string(call, "owner")
    })
}

pub(super) fn collect_set(class: &StmtClassDef, ctx: &mut LowerCtx) {
    let Some(range) = class.decorator_list.iter().find_map(|decorator| {
        matches!(&decorator.expression, Expr::Name(name) if name.id.as_str() == "attached_api_set")
            .then(|| decorator.expression.range())
    }) else {
        return;
    };
    let valid_body = class
        .body
        .iter()
        .all(|statement| matches!(statement, Stmt::Pass(_)));
    if !class.bases().is_empty() || !valid_body {
        malformed(
            ctx,
            "attached_api_set_shape",
            "@attached_api_set requires a field-less class containing only pass",
            class.range(),
        );
    }
    ctx.attached_api_sets.push(AttachedApiSetDeclaration {
        identity: AttachedApiSetIdentity {
            module: ctx.current_module_name.clone().unwrap_or_default(),
            symbol: class.name.to_string(),
        },
        range,
    });
    ctx.attached_api_set_bindings.insert(class.name.to_string());
}

pub(super) fn declaration(
    function: &StmtFunctionDef,
    decorator: &Decorator,
    ctx: &mut LowerCtx,
) -> Option<AttachedApiDeclaration> {
    let Expr::Call(call) = &decorator.expression else {
        return None;
    };
    if !matches!(call.func.as_ref(), Expr::Name(name) if name.id.as_str() == "attached_api") {
        return None;
    }
    if call.arguments.args.len() != 2 || call.arguments.keywords.len() != 3 {
        malformed(
            ctx,
            "attached_api_declaration",
            "@attached_api requires module and set string literals plus public_name, receiver, and owner keywords",
            call.range(),
        );
        return None;
    }
    let Some(set_module) = string_literal(&call.arguments.args[0]) else {
        malformed(
            ctx,
            "attached_api_declaration",
            "attached API module must be a string literal",
            call.arguments.args[0].range(),
        );
        return None;
    };
    let Some(set_symbol) = string_literal(&call.arguments.args[1]) else {
        malformed(
            ctx,
            "attached_api_declaration",
            "attached API set must be a string literal",
            call.arguments.args[1].range(),
        );
        return None;
    };
    let (Some(public_name), Some(receiver), Some(owner)) = (
        keyword_string(call, "public_name"),
        keyword_string(call, "receiver"),
        keyword_string(call, "owner"),
    ) else {
        malformed(
            ctx,
            "attached_api_declaration",
            "attached API keywords public_name, receiver, and owner must each be unique string literals",
            call.range(),
        );
        return None;
    };
    let receiver = match receiver.as_str() {
        "type" => AttachedApiReceiver::Type,
        "immutable" => AttachedApiReceiver::Immutable,
        "mutable" => AttachedApiReceiver::Mutable,
        "owned" => AttachedApiReceiver::Owned,
        _ => {
            malformed(
                ctx,
                "attached_api_receiver",
                "attached API receiver must be type, immutable, mutable, or owned",
                call.range(),
            );
            return None;
        }
    };
    Some(AttachedApiDeclaration {
        module: ctx.current_module_name.clone().unwrap_or_default(),
        function: function.name.to_string(),
        set: AttachedApiSetIdentity {
            module: set_module,
            symbol: set_symbol,
        },
        public_name,
        receiver,
        owner_type_param: owner,
        type_params: Vec::new(),
        type_param_bounds: std::collections::BTreeMap::new(),
        function_type: sifr_type_system::FunctionType::new(Vec::new(), Type::Any),
        defaults: Vec::new(),
        range: decorator.expression.range(),
    })
}

pub(super) fn finalize(ctx: &mut LowerCtx, module: &str) {
    let mut seen = HashSet::new();
    for index in 0..ctx.attached_apis.len() {
        let mut declaration = ctx.attached_apis[index].clone();
        let set_exists = if declaration.set.module == module {
            ctx.attached_api_sets
                .iter()
                .any(|set| set.identity == declaration.set)
        } else {
            ctx.externals
                .attached_api_sets
                .get(&declaration.set.module)
                .is_some_and(|sets| sets.contains_key(&declaration.set.symbol))
        };
        if !set_exists {
            malformed(
                ctx,
                "attached_api_set",
                "@attached_api references an unknown canonical attached-API set",
                declaration.range,
            );
            continue;
        }
        if !seen.insert((declaration.set.clone(), declaration.public_name.clone())) {
            malformed(
                ctx,
                "attached_api_name",
                "an attached-API set may expose each public name exactly once",
                declaration.range,
            );
            continue;
        }
        let Some(function_type) = ctx.functions.get(&declaration.function).cloned() else {
            malformed(
                ctx,
                "attached_api_signature",
                "attached API function signature is unavailable",
                declaration.range,
            );
            continue;
        };
        let type_params = ctx
            .generic_functions
            .get(&declaration.function)
            .cloned()
            .unwrap_or_default();
        if !type_params.contains(&declaration.owner_type_param) {
            malformed(
                ctx,
                "attached_api_owner",
                "attached API owner must name a declared function type parameter",
                declaration.range,
            );
            continue;
        }
        let bounds = ctx
            .type_param_bounds
            .get(&declaration.function)
            .cloned()
            .unwrap_or_default();
        if !bounds
            .get(&declaration.owner_type_param)
            .is_some_and(|items| {
                items
                    .iter()
                    .any(|item| item == "StaticProgram" || item == "MethodSlots")
            })
        {
            malformed(
                ctx,
                "attached_api_owner_bound",
                "attached API owner type parameter must be bounded by StaticProgram or MethodSlots",
                declaration.range,
            );
            continue;
        }
        if !receiver_signature_matches(&declaration, &function_type) {
            malformed(
                ctx,
                "attached_api_signature",
                "instance attached APIs require the owner type as their first parameter with the receiver's borrow or ownership convention",
                declaration.range,
            );
            continue;
        }
        declaration.type_params = type_params;
        declaration.type_param_bounds = bounds.into_iter().collect();
        declaration.function_type = function_type;
        declaration.defaults = ctx
            .function_defaults
            .get(&declaration.function)
            .cloned()
            .unwrap_or_default();
        ctx.attached_apis[index] = declaration;
    }
}

fn receiver_signature_matches(
    declaration: &AttachedApiDeclaration,
    function_type: &sifr_type_system::FunctionType,
) -> bool {
    if declaration.receiver == AttachedApiReceiver::Type {
        return true;
    }
    let Some((_, ty, convention)) = function_type.params.first() else {
        return false;
    };
    if ty != &Type::TypeVar(declaration.owner_type_param.clone()) {
        return false;
    }
    match declaration.receiver {
        AttachedApiReceiver::Type => true,
        AttachedApiReceiver::Immutable => convention.is_shared_borrow(),
        AttachedApiReceiver::Mutable => convention.is_mut_borrow(),
        AttachedApiReceiver::Owned => convention.is_owned() && !convention.is_mut_borrow(),
    }
}

fn keyword_string(call: &sifr_python_ast::ExprCall, expected: &str) -> Option<String> {
    let mut matches = call.arguments.keywords.iter().filter(|keyword| {
        keyword
            .arg
            .as_ref()
            .is_some_and(|name| name.as_str() == expected)
    });
    let value = string_literal(&matches.next()?.value)?;
    matches.next().is_none().then_some(value)
}

fn string_literal(expression: &Expr) -> Option<String> {
    match expression {
        Expr::StringLiteral(value) => Some(value.value.to_str().to_string()),
        _ => None,
    }
}
