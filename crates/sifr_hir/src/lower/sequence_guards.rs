use super::LowerCtx;

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
}
