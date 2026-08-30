use crate::analysis::ScopeFrame;
use crate::ast::{Expression, ExpressionKind};
use crate::catalog::PostgresCatalog;

pub(crate) fn refine_for_null_test(
    catalog: &PostgresCatalog,
    frames: &[ScopeFrame],
    condition: &Expression,
    truth: bool,
) -> Vec<ScopeFrame> {
    let ExpressionKind::NullTest { expression, is_not } = &condition.kind else {
        return frames.to_vec();
    };
    if *is_not != truth {
        return frames.to_vec();
    }
    let ExpressionKind::Column { path } = &expression.kind else {
        return frames.to_vec();
    };
    let Some(column_name) = path.last() else {
        return frames.to_vec();
    };
    let qualifier = (path.len() > 1).then(|| &path[path.len() - 2]);
    let mut refined = frames.to_vec();
    for binding in refined
        .iter_mut()
        .rev()
        .flat_map(|frame| &mut frame.bindings)
        .filter(|binding| qualifier.is_none_or(|value| &binding.alias == value))
    {
        if !binding.columns.contains_key(column_name) {
            continue;
        }
        if let Some(relation) = binding
            .relation
            .as_ref()
            .and_then(|identity| catalog.relation_by_id(identity))
        {
            for (name, column) in &relation.columns {
                if let Some(current) = binding.columns.get_mut(name) {
                    current.nullable = column.nullable;
                }
            }
        } else if let Some(column) = binding.columns.get_mut(column_name) {
            column.nullable = false;
        }
        break;
    }
    refined
}
