use super::LowerCtx;
use crate::scope::MovedSnapshot;
use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::lower) enum MustUseObligationKind {
    CloseLike,
    ContextOnly,
    AsyncContextOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::lower) struct MustUseObligation {
    pub(in crate::lower) kind: MustUseObligationKind,
    pub(in crate::lower) label: String,
}

impl std::fmt::Display for MustUseObligation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.label)
    }
}

pub(in crate::lower) fn validate_branch_join(
    ctx: &mut LowerCtx,
    branch_moved_states: &[MovedSnapshot],
    branch_exits: &[bool],
    saved_moved: &MovedSnapshot,
    has_else: bool,
    range: TextRange,
) {
    let mut ownership_branches = branch_moved_states
        .iter()
        .zip(branch_exits)
        .map(|(moved, exits)| (moved, *exits))
        .collect::<Vec<_>>();
    if !has_else {
        ownership_branches.push((saved_moved, false));
    }
    let names = ctx
        .live_must_use_bindings
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    for name in names {
        let continuing = ownership_branches
            .iter()
            .filter(|(_, exits)| !exits)
            .map(|(state, _)| {
                state
                    .iter()
                    .find(|(binding, _)| binding == &name)
                    .is_some_and(|(_, moved)| *moved)
            })
            .collect::<Vec<_>>();
        if continuing.iter().any(|moved| *moved) && continuing.iter().any(|moved| !*moved) {
            ctx.error_with_code_at(
                DiagnosticCode::OWN_USE_AFTER_MOVE,
                format!(
                    "must-use binding '{name}' is consumed on only some continuing control-flow branches"
                ),
                range,
            );
        }
    }
}
