use super::LowerCtx;
use sifr_python_ast::{Expr, ExprCall};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::lower) struct LenAliasFact {
    alias_var: String,
    sequence: String,
}

impl LowerCtx {
    pub(in crate::lower) fn clear_len_alias(&mut self, alias_var: &str) {
        self.len_aliases
            .retain(|fact| fact.alias_var.as_str() != alias_var);
    }

    pub(in crate::lower) fn set_len_alias(&mut self, alias_var: String, sequence: String) {
        self.clear_len_alias(&alias_var);
        self.len_aliases.push(LenAliasFact {
            alias_var,
            sequence,
        });
    }

    pub(in crate::lower) fn len_alias_sequence(&self, alias_var: &str) -> Option<String> {
        self.len_aliases
            .iter()
            .find_map(|fact| (fact.alias_var.as_str() == alias_var).then(|| fact.sequence.clone()))
    }
}

pub(in crate::lower) fn record_len_alias_fact(ctx: &mut LowerCtx, name: &str, value: &Expr) {
    ctx.clear_len_alias(name);
    if let Some(sequence) = len_alias_target_sequence(value, ctx) {
        ctx.set_len_alias(name.to_string(), sequence);
    }
}

pub(in crate::lower) fn record_tuple_unpack_len_alias_facts(
    ctx: &mut LowerCtx,
    target_names: &[String],
    value: &Expr,
) {
    let Expr::Tuple(value_tuple) = value else {
        for name in target_names {
            ctx.clear_len_alias(name);
        }
        return;
    };
    if value_tuple.elts.len() != target_names.len() {
        for name in target_names {
            ctx.clear_len_alias(name);
        }
        return;
    }
    for (index, name) in target_names.iter().enumerate() {
        record_len_alias_fact(ctx, name, &value_tuple.elts[index]);
    }
}

fn len_alias_target_sequence(value: &Expr, ctx: &LowerCtx) -> Option<String> {
    len_call_sequence_name(value).or_else(|| match value {
        Expr::Name(name) => ctx.len_alias_sequence(name.id.as_str()),
        _ => None,
    })
}

fn len_call_sequence_name(expr: &Expr) -> Option<String> {
    let Expr::Call(ExprCall {
        func, arguments, ..
    }) = expr
    else {
        return None;
    };
    let Expr::Name(func_name) = func.as_ref() else {
        return None;
    };
    if func_name.id.as_str() != "len" || arguments.args.len() != 1 || !arguments.keywords.is_empty()
    {
        return None;
    }
    let Expr::Name(sequence_name) = &arguments.args[0] else {
        return None;
    };
    Some(sequence_name.id.to_string())
}
