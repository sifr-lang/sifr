use std::collections::BTreeSet;

use crate::RustExpr;

pub(super) fn checked_place_expr_token(expr: &crate::HirExpr) -> Option<String> {
    match expr {
        crate::HirExpr::Name { name, .. } => Some(format!("name:{name}")),
        crate::HirExpr::IntLiteral(value) => Some(format!("int:{value}")),
        crate::HirExpr::LargeIntLiteral(value) => Some(format!("int:{value}")),
        crate::HirExpr::StringLiteral(value) => Some(format!("str:{value:?}")),
        crate::HirExpr::BoolLiteral(value) => Some(format!("bool:{value}")),
        crate::HirExpr::FieldAccess { object, field, .. } => Some(format!(
            "field:{}.{field}",
            checked_place_expr_token(object)?
        )),
        crate::HirExpr::Index { object, index, .. } => Some(format!(
            "index:{}[{}]",
            checked_place_expr_token(object)?,
            checked_place_expr_token(index)?
        )),
        crate::HirExpr::TupleLiteral { elements, .. } => {
            let elements = elements
                .iter()
                .map(checked_place_expr_token)
                .collect::<Option<Vec<_>>>()?;
            Some(format!("tuple:({})", elements.join(",")))
        }
        crate::HirExpr::BinOp {
            left, op, right, ..
        } => Some(format!(
            "binop:{op}({},{})",
            checked_place_expr_token(left)?,
            checked_place_expr_token(right)?
        )),
        crate::HirExpr::UnaryOp { op, operand, .. } => Some(format!(
            "unary:{op}({})",
            checked_place_expr_token(operand)?
        )),
        _ => None,
    }
}

pub(super) fn checked_place_read_key(
    object: &crate::HirExpr,
    index: &crate::HirExpr,
) -> Option<String> {
    Some(format!(
        "{}[{}]",
        checked_place_expr_token(object)?,
        checked_place_expr_token(index)?
    ))
}

pub(super) fn checked_place_dependencies(
    object: &crate::HirExpr,
    index: &crate::HirExpr,
) -> Vec<String> {
    let mut dependencies = BTreeSet::new();
    for root in [object, index] {
        crate::hir_analysis::traversal::walk_expr(root, &mut |expr| {
            if let crate::HirExpr::Name { name, .. } = expr {
                dependencies.insert(name.clone());
            }
        });
    }
    dependencies.into_iter().collect()
}

#[derive(Clone)]
pub(crate) struct CheckedDictReadGuard {
    pub(crate) key: String,
    pub(crate) binding: String,
    pub(crate) option: RustExpr,
    pub(crate) negated: bool,
    pub(crate) borrowed: bool,
    pub(crate) dependencies: Vec<String>,
    pub(crate) order: usize,
}

#[derive(Clone)]
pub(crate) struct CheckedPlaceReadWitness {
    pub(super) binding: String,
    pub(super) borrowed: bool,
    pub(super) option: RustExpr,
    pub(super) dependencies: Vec<String>,
    pub(super) order: usize,
}

impl CheckedDictReadGuard {
    pub(super) fn witness(&self) -> CheckedPlaceReadWitness {
        CheckedPlaceReadWitness {
            binding: self.binding.clone(),
            borrowed: self.borrowed,
            option: self.option.clone(),
            dependencies: self.dependencies.clone(),
            order: self.order,
        }
    }
}
