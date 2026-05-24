use super::LowerCtx;

impl LowerCtx {
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
mod tests {
    use super::LowerCtx;
    use sifr_type_system::Type;

    #[test]
    fn with_pushed_scope_restores_scope_after_none() {
        let mut ctx = LowerCtx::new();
        assert_eq!(ctx.scope.frame_count(), 1);

        let result: Option<()> = ctx.with_pushed_scope(|ctx| {
            ctx.scope.define("temp".to_string(), Type::Int);
            None
        });

        assert!(result.is_none());
        assert_eq!(ctx.scope.frame_count(), 1);
        assert!(ctx.scope.lookup("temp").is_none());
    }
}
