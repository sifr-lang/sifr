use super::LowerCtx;
use sifr_python_ast::{Expr, Number, Operator, UnaryOp};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SequenceGuard {
    MinLength {
        sequence: String,
        min_len: usize,
    },
    IndexVarInRange {
        sequence: String,
        index_var: String,
        max_offset: usize,
    },
    DictContains {
        dict: String,
        key_expr_debug: String,
    },
}

impl LowerCtx {
    pub(super) fn add_sequence_guard(&mut self, guard: SequenceGuard) {
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
                self.sequence_guards
                    .push(SequenceGuard::DictContains {
                        dict,
                        key_expr_debug,
                    });
            }
        }
    }

    pub(super) fn save_sequence_guards(&self) -> Vec<SequenceGuard> {
        self.sequence_guards.clone()
    }

    pub(super) fn restore_sequence_guards(&mut self, snapshot: &[SequenceGuard]) {
        self.sequence_guards = snapshot.to_vec();
    }

    pub(super) fn min_length_guard(&self, sequence: &str) -> usize {
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

    pub(super) fn has_index_var_guard(&self, sequence: &str, index_var: &str) -> bool {
        self.has_index_var_offset_guard(sequence, index_var, 0)
    }

    pub(super) fn has_index_var_offset_guard(
        &self,
        sequence: &str,
        index_var: &str,
        offset: usize,
    ) -> bool {
        self.sequence_guards.iter().any(|guard| {
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
        })
    }

    pub(super) fn has_dict_key_guard(&self, dict: &str, key_expr: &Expr) -> bool {
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
}

pub(super) fn key_guard_token(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Name(name) => Some(format!("name:{}", name.id)),
        Expr::NumberLiteral(num) => match &num.value {
            Number::Int(value) => Some(format!("int:{}", value)),
            Number::Float(value) => Some(format!("float:{value}")),
            Number::Complex { .. } => None,
        },
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
