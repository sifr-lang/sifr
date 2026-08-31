use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use sifr_python_ast::{Expr, Number, Operator, UnaryOp};
use sifr_type_system::ParamConvention;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::lower) enum SubscriptReferenceStability {
    StableAcrossGrowth,
    MayChangeOnGrowth,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::lower) enum SequenceGuard {
    MinLength {
        sequence: String,
        min_len: usize,
    },
    IndexVarInRange {
        sequence: String,
        index_var: String,
        max_offset: usize,
    },
    IndexVarNonNegative {
        index_var: String,
    },
    DictContains {
        dict: String,
        key_expr_debug: String,
    },
    SubscriptAccessible {
        sequence: String,
        index_expr_debug: String,
    },
    SubscriptPresent {
        sequence: String,
        index_expr_debug: String,
        reference_stability: SubscriptReferenceStability,
    },
}

impl LowerCtx {
    pub(in crate::lower) fn add_sequence_guard(&mut self, guard: SequenceGuard) {
        match guard {
            SequenceGuard::MinLength { sequence, min_len } => {
                for existing in &mut self.sequence_guards {
                    if let SequenceGuard::MinLength {
                        sequence: existing_sequence,
                        min_len: existing_min_len,
                    } = existing
                    {
                        if existing_sequence == &sequence {
                            *existing_min_len = (*existing_min_len).max(min_len);
                            return;
                        }
                    }
                }
                self.sequence_guards
                    .push(SequenceGuard::MinLength { sequence, min_len });
            }
            SequenceGuard::IndexVarInRange {
                sequence,
                index_var,
                max_offset,
            } => {
                for existing in &mut self.sequence_guards {
                    if let SequenceGuard::IndexVarInRange {
                        sequence: existing_sequence,
                        index_var: existing_index_var,
                        max_offset: existing_max_offset,
                    } = existing
                    {
                        if existing_sequence == &sequence && existing_index_var == &index_var {
                            *existing_max_offset = (*existing_max_offset).max(max_offset);
                            return;
                        }
                    }
                }
                self.sequence_guards.push(SequenceGuard::IndexVarInRange {
                    sequence,
                    index_var,
                    max_offset,
                });
            }
            SequenceGuard::IndexVarNonNegative { index_var } => {
                if !self.sequence_guards.iter().any(|existing| {
                    matches!(
                        existing,
                        SequenceGuard::IndexVarNonNegative {
                            index_var: existing_index,
                        } if existing_index == &index_var
                    )
                }) {
                    self.sequence_guards
                        .push(SequenceGuard::IndexVarNonNegative { index_var });
                }
            }
            SequenceGuard::DictContains {
                dict,
                key_expr_debug,
            } => {
                if self.sequence_guards.iter().any(|existing| {
                    matches!(
                        existing,
                        SequenceGuard::DictContains {
                            dict: existing_dict,
                            key_expr_debug: existing_key,
                        } if existing_dict == &dict && existing_key == &key_expr_debug
                    )
                }) {
                    return;
                }
                self.sequence_guards.push(SequenceGuard::DictContains {
                    dict,
                    key_expr_debug,
                });
            }
            SequenceGuard::SubscriptPresent {
                sequence,
                index_expr_debug,
                reference_stability,
            } => {
                if self.sequence_guards.iter().any(|existing| {
                    matches!(
                        existing,
                        SequenceGuard::SubscriptPresent {
                            sequence: existing_sequence,
                            index_expr_debug: existing_index,
                            reference_stability: existing_stability,
                        } if existing_sequence == &sequence
                            && existing_index == &index_expr_debug
                            && existing_stability == &reference_stability
                    )
                }) {
                    return;
                }
                self.sequence_guards.push(SequenceGuard::SubscriptPresent {
                    sequence,
                    index_expr_debug,
                    reference_stability,
                });
            }
            SequenceGuard::SubscriptAccessible {
                sequence,
                index_expr_debug,
            } => {
                if self.sequence_guards.iter().any(|existing| {
                    matches!(
                        existing,
                        SequenceGuard::SubscriptAccessible {
                            sequence: existing_sequence,
                            index_expr_debug: existing_index,
                        } if existing_sequence == &sequence && existing_index == &index_expr_debug
                    )
                }) {
                    return;
                }
                self.sequence_guards
                    .push(SequenceGuard::SubscriptAccessible {
                        sequence,
                        index_expr_debug,
                    });
            }
        }
    }

    pub(in crate::lower) fn save_sequence_guards(&self) -> Vec<SequenceGuard> {
        self.sequence_guards.clone()
    }

    pub(in crate::lower) fn restore_sequence_guards(&mut self, snapshot: &[SequenceGuard]) {
        self.sequence_guards = snapshot.to_vec();
    }

    pub(in crate::lower) fn clear_sequence_guards_for_binding(&mut self, binding: &str) {
        self.sequence_guards
            .retain(|guard| !guard_depends_on_binding(guard, binding));
    }

    pub(in crate::lower) fn clear_sequence_guards_for_target(&mut self, target: &str) {
        self.sequence_guards
            .retain(|guard| !guard_depends_on_target(guard, target));
    }

    pub(in crate::lower) fn clear_subscript_presence_guards_for_target(&mut self, target: &str) {
        self.sequence_guards.retain(|guard| {
            !matches!(
                guard,
                SequenceGuard::SubscriptPresent { sequence, .. }
                    if path_depends_on_target(sequence, target)
            )
        });
    }

    pub(in crate::lower) fn clear_growth_sensitive_subscript_presence_guards_for_target(
        &mut self,
        target: &str,
    ) {
        self.sequence_guards.retain(|guard| {
            !matches!(
                guard,
                SequenceGuard::SubscriptPresent {
                    sequence,
                    reference_stability: SubscriptReferenceStability::MayChangeOnGrowth,
                    ..
                } if path_depends_on_target(sequence, target)
            )
        });
    }

    pub(in crate::lower) fn min_length_guard(&self, sequence: &str) -> usize {
        self.sequence_guards
            .iter()
            .filter_map(|guard| match guard {
                SequenceGuard::MinLength {
                    sequence: guard_sequence,
                    min_len,
                } if guard_sequence == sequence => Some(*min_len),
                _ => None,
            })
            .max()
            .unwrap_or(0)
    }

    pub(in crate::lower) fn has_index_var_guard(&self, sequence: &str, index_var: &str) -> bool {
        self.has_index_var_offset_guard(sequence, index_var, 0)
    }

    pub(in crate::lower) fn has_index_var_offset_guard(
        &self,
        sequence: &str,
        index_var: &str,
        offset: usize,
    ) -> bool {
        let has_upper_bound = self.sequence_guards.iter().any(|guard| {
            matches!(
                guard,
                SequenceGuard::IndexVarInRange {
                    sequence: guard_sequence,
                    index_var: guard_index_var,
                    max_offset,
                } if guard_sequence == sequence
                    && guard_index_var == index_var
                    && *max_offset >= offset
            )
        });
        let has_lower_bound = self.is_zero_based_pointer(index_var)
            || self.sequence_guards.iter().any(|guard| {
                matches!(
                    guard,
                    SequenceGuard::IndexVarNonNegative {
                        index_var: guard_index,
                    } if guard_index == index_var
                )
            });
        has_upper_bound && has_lower_bound
    }

    pub(in crate::lower) fn has_dict_key_guard(&self, dict: &str, key_expr: &Expr) -> bool {
        let Some(key_expr_debug) = key_guard_token(key_expr) else {
            return false;
        };
        self.sequence_guards.iter().any(|guard| {
            matches!(
                guard,
                SequenceGuard::DictContains {
                    dict: guard_dict,
                    key_expr_debug: guard_key,
                } if guard_dict == dict && guard_key == &key_expr_debug
            )
        })
    }

    pub(in crate::lower) fn has_subscript_guard(&self, sequence: &str, index_expr: &Expr) -> bool {
        let Some(index_expr_debug) = key_guard_token(index_expr) else {
            return false;
        };
        self.sequence_guards.iter().any(|guard| {
            matches!(
                guard,
                SequenceGuard::SubscriptPresent {
                    sequence: guard_sequence,
                    index_expr_debug: guard_index,
                    ..
                } if guard_sequence == sequence && guard_index == &index_expr_debug
            )
        })
    }

    pub(in crate::lower) fn has_subscript_access_guard(
        &self,
        sequence: &str,
        index_expr: &Expr,
    ) -> bool {
        let Some(index_expr_debug) = key_guard_token(index_expr) else {
            return false;
        };
        self.sequence_guards.iter().any(|guard| {
            matches!(
                guard,
                SequenceGuard::SubscriptAccessible {
                    sequence: guard_sequence,
                    index_expr_debug: guard_index,
                } if guard_sequence == sequence && guard_index == &index_expr_debug
            )
        })
    }
}

fn guard_depends_on_binding(guard: &SequenceGuard, binding: &str) -> bool {
    match guard {
        SequenceGuard::MinLength { sequence, .. } => path_depends_on_binding(sequence, binding),
        SequenceGuard::IndexVarInRange {
            sequence,
            index_var,
            ..
        } => path_depends_on_binding(sequence, binding) || index_var == binding,
        SequenceGuard::IndexVarNonNegative { index_var } => index_var == binding,
        SequenceGuard::DictContains {
            dict,
            key_expr_debug,
        } => path_depends_on_binding(dict, binding) || token_mentions_name(key_expr_debug, binding),
        SequenceGuard::SubscriptAccessible {
            sequence,
            index_expr_debug,
        }
        | SequenceGuard::SubscriptPresent {
            sequence,
            index_expr_debug,
            ..
        } => {
            path_depends_on_binding(sequence, binding)
                || token_mentions_name(index_expr_debug, binding)
        }
    }
}

fn guard_depends_on_target(guard: &SequenceGuard, target: &str) -> bool {
    match guard {
        SequenceGuard::MinLength { sequence, .. }
        | SequenceGuard::IndexVarInRange { sequence, .. }
        | SequenceGuard::SubscriptAccessible { sequence, .. }
        | SequenceGuard::SubscriptPresent { sequence, .. } => {
            path_depends_on_target(sequence, target)
        }
        SequenceGuard::IndexVarNonNegative { index_var } => index_var == target,
        SequenceGuard::DictContains { dict, .. } => path_depends_on_target(dict, target),
    }
}

fn path_depends_on_binding(path: &str, binding: &str) -> bool {
    path == binding || path.starts_with(&format!("{binding}."))
}

fn path_depends_on_target(path: &str, target: &str) -> bool {
    path == target || path.starts_with(&format!("{target}."))
}

fn token_mentions_name(token: &str, binding: &str) -> bool {
    token == format!("name:{binding}") || token.contains(&format!("name:{binding}"))
}

pub(in crate::lower) fn hir_sequence_guard_target_name(expr: &HirExpr) -> Option<String> {
    match expr {
        HirExpr::Name { name, .. } => Some(name.clone()),
        HirExpr::FieldAccess { object, field, .. } => {
            let base = hir_sequence_guard_target_name(object)?;
            Some(format!("{base}.{field}"))
        }
        _ => None,
    }
}

pub(in crate::lower) fn invalidate_mutable_call_sequence_guards(
    ctx: &mut LowerCtx,
    args: &[HirExpr],
    conventions: impl IntoIterator<Item = ParamConvention>,
) {
    for (arg, convention) in args.iter().zip(conventions) {
        if !convention.is_mut_borrow() {
            continue;
        }
        let Some(target) = hir_sequence_guard_target_name(arg) else {
            continue;
        };
        ctx.clear_sequence_guards_for_binding(&target);
        ctx.clear_sequence_guards_for_target(&target);
    }
}

pub(in crate::lower) fn key_guard_token(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Name(name) => Some(format!("name:{}", name.id)),
        Expr::StringLiteral(value) => Some(format!("str:{:?}", value.value.to_str())),
        Expr::BooleanLiteral(value) => Some(format!("bool:{}", value.value)),
        Expr::NumberLiteral(num) => match &num.value {
            Number::Int(value) => Some(format!("int:{value}")),
            Number::Float(value) => Some(format!("float:{value}")),
            Number::Complex { .. } => None,
        },
        Expr::Tuple(tuple) => {
            let mut parts = Vec::new();
            for element in &tuple.elts {
                parts.push(key_guard_token(element)?);
            }
            Some(format!("tuple:({})", parts.join(", ")))
        }
        Expr::UnaryOp(unary) => {
            let operand = key_guard_token(&unary.operand)?;
            let op = match unary.op {
                UnaryOp::Not => "not",
                UnaryOp::Invert => "~",
                UnaryOp::UAdd => "+",
                UnaryOp::USub => "-",
            };
            Some(format!("({op} {operand})"))
        }
        Expr::BinOp(binop) => {
            let left = key_guard_token(&binop.left)?;
            let right = key_guard_token(&binop.right)?;
            let op = match binop.op {
                Operator::Add => "+",
                Operator::Sub => "-",
                Operator::Mult => "*",
                Operator::Div => "/",
                Operator::Mod => "%",
                Operator::Pow => "**",
                Operator::LShift => "<<",
                Operator::RShift => ">>",
                Operator::BitOr => "|",
                Operator::BitXor => "^",
                Operator::BitAnd => "&",
                Operator::FloorDiv => "//",
                Operator::MatMult => "@",
            };
            Some(format!("({left} {op} {right})"))
        }
        _ => None,
    }
}
