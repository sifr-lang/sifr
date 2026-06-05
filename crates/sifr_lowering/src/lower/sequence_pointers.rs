use super::LowerCtx;
use sifr_python_ast::{Expr, Number};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::lower) enum SequencePointerFact {
    ZeroBased {
        pointer_var: String,
    },
    EndPointer {
        pointer_var: String,
        sequence: String,
    },
}

impl LowerCtx {
    pub(in crate::lower) fn clear_sequence_pointer(&mut self, pointer_var: &str) {
        self.sequence_pointers
            .retain(|fact| fact.pointer_var() != pointer_var);
    }

    pub(in crate::lower) fn clear_sequence_pointers(&mut self) {
        self.sequence_pointers.clear();
    }

    pub(in crate::lower) fn set_zero_based_pointer(&mut self, pointer_var: String) {
        self.clear_sequence_pointer(&pointer_var);
        self.sequence_pointers
            .push(SequencePointerFact::ZeroBased { pointer_var });
    }

    pub(in crate::lower) fn set_end_pointer(&mut self, pointer_var: String, sequence: String) {
        self.clear_sequence_pointer(&pointer_var);
        self.sequence_pointers
            .push(SequencePointerFact::EndPointer {
                pointer_var,
                sequence,
            });
    }

    pub(in crate::lower) fn same_sequence_two_pointer_loop(
        &self,
        left_var: &str,
        right_var: &str,
    ) -> Option<String> {
        let left_is_zero_based = self.is_zero_based_pointer(left_var);
        if !left_is_zero_based {
            return None;
        }
        self.end_pointer_sequence(right_var)
    }

    pub(in crate::lower) fn is_zero_based_pointer(&self, pointer_var: &str) -> bool {
        self.sequence_pointers.iter().any(|fact| {
            matches!(
                fact,
                SequencePointerFact::ZeroBased { pointer_var: fact_var } if fact_var == pointer_var
            )
        })
    }

    pub(in crate::lower) fn end_pointer_sequence(&self, pointer_var: &str) -> Option<String> {
        self.sequence_pointers.iter().find_map(|fact| match fact {
            SequencePointerFact::EndPointer {
                pointer_var: fact_var,
                sequence,
            } if fact_var == pointer_var => Some(sequence.clone()),
            SequencePointerFact::ZeroBased { .. } | SequencePointerFact::EndPointer { .. } => None,
        })
    }
}

impl SequencePointerFact {
    fn pointer_var(&self) -> &str {
        match self {
            Self::ZeroBased { pointer_var } | Self::EndPointer { pointer_var, .. } => pointer_var,
        }
    }
}

pub(in crate::lower) fn record_sequence_pointer_fact(ctx: &mut LowerCtx, name: &str, value: &Expr) {
    ctx.clear_sequence_pointer(name);
    if expr_is_zero(value) {
        ctx.set_zero_based_pointer(name.to_string());
        return;
    }
    if let Some(sequence) = len_minus_one_sequence_name(value, ctx) {
        ctx.set_end_pointer(name.to_string(), sequence);
    }
}

pub(in crate::lower) fn record_tuple_unpack_pointer_facts(
    ctx: &mut LowerCtx,
    target_names: &[String],
    value: &Expr,
) {
    let Expr::Tuple(value_tuple) = value else {
        for name in target_names {
            ctx.clear_sequence_pointer(name);
        }
        return;
    };
    if value_tuple.elts.len() != target_names.len() {
        for name in target_names {
            ctx.clear_sequence_pointer(name);
        }
        return;
    }
    for (index, name) in target_names.iter().enumerate() {
        record_sequence_pointer_fact(ctx, name, &value_tuple.elts[index]);
    }
}

fn expr_is_zero(expr: &Expr) -> bool {
    let Expr::NumberLiteral(num) = expr else {
        return false;
    };
    let Number::Int(value) = &num.value else {
        return false;
    };
    value.as_i64() == Some(0)
}

fn len_minus_one_sequence_name(expr: &Expr, ctx: &LowerCtx) -> Option<String> {
    let Expr::BinOp(binop) = expr else {
        return None;
    };
    if !matches!(binop.op, sifr_python_ast::Operator::Sub) {
        return None;
    }
    let Some(1) = literal_int(binop.right.as_ref()) else {
        return None;
    };
    len_like_sequence_name(binop.left.as_ref(), ctx)
}

fn len_like_sequence_name(expr: &Expr, ctx: &LowerCtx) -> Option<String> {
    len_call_sequence_name(expr).or_else(|| match expr {
        Expr::Name(alias) => ctx.len_alias_sequence(alias.id.as_str()),
        _ => None,
    })
}

fn len_call_sequence_name(expr: &Expr) -> Option<String> {
    let Expr::Call(call) = expr else {
        return None;
    };
    let Expr::Name(func_name) = call.func.as_ref() else {
        return None;
    };
    if func_name.id.as_str() != "len" || call.arguments.args.len() != 1 {
        return None;
    }
    let Expr::Name(sequence_name) = &call.arguments.args[0] else {
        return None;
    };
    Some(sequence_name.id.to_string())
}

fn literal_int(expr: &Expr) -> Option<i64> {
    let Expr::NumberLiteral(num) = expr else {
        return None;
    };
    let Number::Int(value) = &num.value else {
        return None;
    };
    value.as_i64()
}
