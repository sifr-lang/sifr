use super::{HirStmt, RustEmitter, RustStmt};

impl RustEmitter {
    pub(crate) fn try_lower_tuple_unpack_stmt_for_block(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<Option<Vec<RustStmt>>, crate::CodegenError> {
        let HirStmt::TupleUnpack { targets, value } = stmt else {
            return Ok(None);
        };
        let Some(lowered_value) = self.lower_stmt_expr_for_ir(value)? else {
            return Ok(None);
        };
        let source_is_borrowed = crate::tuple_unpack_source_is_borrowed(
            value,
            &self.borrowed_params,
            &self.mut_borrowed_params,
        );
        let mut lowered = crate::lower_tuple_unpack_targets(
            targets,
            value,
            lowered_value,
            &self.mutated_vars,
            source_is_borrowed,
        );
        for target in targets {
            let sifr_ir::HirTupleTargetBinding::Name(name) = &target.binding else {
                continue;
            };
            let cache_stmt = if target.rebind_existing {
                self.string_char_cache_rebuild_stmt_for_local(name)
            } else {
                self.force_string_char_cache_init_stmt_for_local(name, &target.ty)
            };
            if let Some(cache_stmt) = cache_stmt {
                lowered.push(cache_stmt);
            }
        }
        Ok(Some(lowered))
    }
}
