use super::LowerCtx;

impl LowerCtx<'_> {
    pub(in crate::lower) fn with_pushed_scope<T>(
        &mut self,
        f: impl FnOnce(&mut Self) -> Option<T>,
    ) -> Option<T> {
        self.scope.push();
        let result = f(self);
        self.scope.pop();
        result
    }

    pub(in crate::lower) fn pop_scopes(&mut self, count: usize) {
        for _ in 0..count {
            self.scope.pop();
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/lower/scope_helpers/tests.rs"]
mod tests;
