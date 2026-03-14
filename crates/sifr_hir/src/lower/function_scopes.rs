use super::LowerCtx;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub(super) struct FunctionScopeState {
    pub(super) frame_start: usize,
    pub(super) declared_nonlocals: HashSet<String>,
}

impl LowerCtx {
    pub(super) fn enter_function_scope(&mut self, declared_nonlocals: HashSet<String>) {
        self.scope.push();
        self.function_scopes.push(FunctionScopeState {
            frame_start: self.scope.frame_count() - 1,
            declared_nonlocals,
        });
    }

    pub(super) fn exit_function_scope(&mut self) {
        self.function_scopes.pop();
        self.scope.pop();
    }

    pub(super) fn current_function_frame_start(&self) -> Option<usize> {
        self.function_scopes.last().map(|state| state.frame_start)
    }

    pub(super) fn is_declared_nonlocal(&self, name: &str) -> bool {
        self.function_scopes
            .last()
            .is_some_and(|state| state.declared_nonlocals.contains(name))
    }

    pub(super) fn lookup_current_function_binding(
        &self,
        name: &str,
    ) -> Option<&crate::scope::VarInfo> {
        let frame_start = self.current_function_frame_start()?;
        self.scope
            .lookup_in_frame_range(name, frame_start, self.scope.frame_count() - 1)
    }

    pub(super) fn lookup_outer_function_binding(
        &self,
        name: &str,
    ) -> Option<&crate::scope::VarInfo> {
        let current = self.function_scopes.last()?;
        let outermost_function_frame_start = self.function_scopes.first()?.frame_start;
        current.frame_start.checked_sub(1).and_then(|end| {
            self.scope
                .lookup_in_frame_range(name, outermost_function_frame_start, end)
        })
    }
}
