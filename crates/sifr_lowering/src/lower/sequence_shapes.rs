use super::LowerCtx;
use sifr_python_ast::{Expr, ExprBinOp, ExprCall, ExprList, ExprListComp, Operator};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::lower) enum SequenceShapeFact {
    SizedByAnchor {
        sequence_var: String,
        anchor_sequence: String,
        extra_len: usize,
    },
    MatrixSizedByAnchors {
        matrix_var: String,
        outer_anchor: String,
        outer_extra_len: usize,
        inner_anchor: String,
        inner_extra_len: usize,
    },
}

impl LowerCtx {
    pub(in crate::lower) fn clear_sequence_shape_fact(&mut self, name: &str) {
        self.sequence_shapes.retain(|fact| fact.var_name() != name);
    }

    pub(in crate::lower) fn record_sequence_shape_fact(&mut self, fact: SequenceShapeFact) {
        self.clear_sequence_shape_fact(fact.var_name());
        self.sequence_shapes.push(fact);
    }

    pub(in crate::lower) fn sized_sequence_fact(&self, name: &str) -> Option<(String, usize)> {
        self.sequence_shapes.iter().find_map(|fact| match fact {
            SequenceShapeFact::SizedByAnchor {
                sequence_var,
                anchor_sequence,
                extra_len,
            } if sequence_var == name => Some((anchor_sequence.clone(), *extra_len)),
            SequenceShapeFact::SizedByAnchor { .. }
            | SequenceShapeFact::MatrixSizedByAnchors { .. } => None,
        })
    }

    pub(in crate::lower) fn matrix_sequence_fact(
        &self,
        name: &str,
    ) -> Option<(String, usize, String, usize)> {
        self.sequence_shapes.iter().find_map(|fact| match fact {
            SequenceShapeFact::MatrixSizedByAnchors {
                matrix_var,
                outer_anchor,
                outer_extra_len,
                inner_anchor,
                inner_extra_len,
            } if matrix_var == name => Some((
                outer_anchor.clone(),
                *outer_extra_len,
                inner_anchor.clone(),
                *inner_extra_len,
            )),
            SequenceShapeFact::SizedByAnchor { .. }
            | SequenceShapeFact::MatrixSizedByAnchors { .. } => None,
        })
    }
}

impl SequenceShapeFact {
    fn var_name(&self) -> &str {
        match self {
            Self::SizedByAnchor { sequence_var, .. } => sequence_var,
            Self::MatrixSizedByAnchors { matrix_var, .. } => matrix_var,
        }
    }
}

pub(in crate::lower) fn sequence_shape_fact(name: &str, expr: &Expr) -> Option<SequenceShapeFact> {
    if let Some(matrix_fact) = matrix_list_comp_fact(name, expr) {
        return Some(matrix_fact);
    }
    if let Some((anchor_sequence, extra_len)) = sized_list_comp_fact(expr) {
        return Some(SequenceShapeFact::SizedByAnchor {
            sequence_var: name.to_string(),
            anchor_sequence,
            extra_len,
        });
    }
    None
}

fn matrix_list_comp_fact(name: &str, expr: &Expr) -> Option<SequenceShapeFact> {
    let Expr::ListComp(outer_comp) = expr else {
        return None;
    };
    let (outer_anchor, outer_extra_len) = list_comp_range_shape(outer_comp)?;
    let (inner_anchor, inner_extra_len) = matrix_inner_shape(outer_comp.elt.as_ref())?;
    Some(SequenceShapeFact::MatrixSizedByAnchors {
        matrix_var: name.to_string(),
        outer_anchor,
        outer_extra_len,
        inner_anchor,
        inner_extra_len,
    })
}

fn matrix_inner_shape(expr: &Expr) -> Option<(String, usize)> {
    match expr {
        Expr::ListComp(inner_comp) => list_comp_range_shape(inner_comp),
        _ => singleton_list_repeat_shape(expr),
    }
}

fn singleton_list_repeat_shape(expr: &Expr) -> Option<(String, usize)> {
    let Expr::BinOp(ExprBinOp {
        left, op, right, ..
    }) = expr
    else {
        return None;
    };
    if !matches!(op, Operator::Mult) {
        return None;
    }
    singleton_list_repeat_pair(left.as_ref(), right.as_ref())
        .or_else(|| singleton_list_repeat_pair(right.as_ref(), left.as_ref()))
}

fn singleton_list_repeat_pair(list_expr: &Expr, len_expr: &Expr) -> Option<(String, usize)> {
    let Expr::List(ExprList { elts, .. }) = list_expr else {
        return None;
    };
    if elts.len() != 1 {
        return None;
    }
    len_plus_literal(len_expr)
}

fn sized_list_comp_fact(expr: &Expr) -> Option<(String, usize)> {
    let Expr::ListComp(comp) = expr else {
        return None;
    };
    if matches!(comp.elt.as_ref(), Expr::ListComp(_)) {
        return None;
    }
    list_comp_range_shape(comp)
}

fn list_comp_range_shape(comp: &ExprListComp) -> Option<(String, usize)> {
    if comp.generators.len() != 1 {
        return None;
    }
    let generator = &comp.generators[0];
    range_len_plus_literal_shape(&generator.iter)
}

fn range_len_plus_literal_shape(expr: &Expr) -> Option<(String, usize)> {
    let Expr::Call(ExprCall {
        func, arguments, ..
    }) = expr
    else {
        return None;
    };
    let Expr::Name(func_name) = func.as_ref() else {
        return None;
    };
    if func_name.id.as_str() != "range" || arguments.args.len() != 1 {
        return None;
    }
    len_plus_literal(&arguments.args[0])
}

fn len_plus_literal(expr: &Expr) -> Option<(String, usize)> {
    if let Some(sequence_name) = len_call_sequence_name(expr) {
        return Some((sequence_name, 0));
    }
    let Expr::BinOp(ExprBinOp {
        left, op, right, ..
    }) = expr
    else {
        return None;
    };
    if !matches!(op, Operator::Add) {
        return None;
    }
    let sequence_name = len_call_sequence_name(left.as_ref())?;
    let extra_len = literal_usize(right.as_ref())?;
    Some((sequence_name, extra_len))
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
    if func_name.id.as_str() != "len" || arguments.args.len() != 1 {
        return None;
    }
    let Expr::Name(sequence_name) = &arguments.args[0] else {
        return None;
    };
    Some(sequence_name.id.to_string())
}

fn literal_usize(expr: &Expr) -> Option<usize> {
    let Expr::NumberLiteral(num) = expr else {
        return None;
    };
    let sifr_python_ast::Number::Int(value) = &num.value else {
        return None;
    };
    value.as_i64().and_then(|value| usize::try_from(value).ok())
}
