use super::{HirExpr, RustEmitter};

impl RustEmitter {
    pub(crate) fn materialize_explicit_borrowed_clone(
        &self,
        object: &HirExpr,
        method: &str,
        args: &[HirExpr],
        lowered_object: crate::RustExpr,
    ) -> Option<crate::RustExpr> {
        if !matches!(method, "clone" | "copy" | "cloned") || !args.is_empty() {
            return None;
        }
        let HirExpr::Name { name, ty, .. } = object else {
            return None;
        };
        if !(self.borrowed_params.contains(name) || self.mut_borrowed_params.contains(name)) {
            return None;
        }
        matches!(
            ty.resolve_alias(),
            sifr_type_system::Type::Str
                | sifr_type_system::Type::LiteralStr(_)
                | sifr_type_system::Type::Bytes
                | sifr_type_system::Type::List(_)
                | sifr_type_system::Type::Iterable(_)
        )
        .then(|| crate::ownership_plan::materialize_owned_value(ty, lowered_object))
    }

    pub(crate) fn clone_moved_names_in_borrowed_aggregate(
        arg: &HirExpr,
        lowered: crate::RustExpr,
    ) -> crate::RustExpr {
        Self::clone_moved_names_in_borrowed_aggregate_inner(arg, lowered, false)
    }

    fn clone_moved_names_in_borrowed_aggregate_inner(
        arg: &HirExpr,
        lowered: crate::RustExpr,
        in_aggregate: bool,
    ) -> crate::RustExpr {
        match (arg, lowered) {
            (HirExpr::ListLiteral { elements, .. }, crate::RustExpr::Vec(items)) => {
                crate::RustExpr::Vec(
                    elements
                        .iter()
                        .zip(items)
                        .map(|(element, item)| {
                            Self::clone_moved_names_in_borrowed_aggregate_inner(element, item, true)
                        })
                        .collect(),
                )
            }
            (HirExpr::TupleLiteral { elements, .. }, crate::RustExpr::Tuple(items)) => {
                crate::RustExpr::Tuple(
                    elements
                        .iter()
                        .zip(items)
                        .map(|(element, item)| {
                            Self::clone_moved_names_in_borrowed_aggregate_inner(element, item, true)
                        })
                        .collect(),
                )
            }
            (HirExpr::Name { ty, .. }, lowered_expr)
                if in_aggregate
                    && !crate::helpers::is_copy_type_for_codegen(ty)
                    && !matches!(&lowered_expr, crate::RustExpr::Clone(_))
                    && !matches!(
                        &lowered_expr,
                        crate::RustExpr::MethodCall { method, args, .. }
                            if method == "clone" && args.is_empty()
                    ) =>
            {
                crate::ownership_plan::materialize_owned_value(ty, lowered_expr)
            }
            (_, lowered_expr) => lowered_expr,
        }
    }

    pub(crate) fn arg_is_already_borrowed_for_registry_call(
        &self,
        arg: &HirExpr,
        lowered: &crate::RustExpr,
    ) -> bool {
        if matches!(lowered, crate::RustExpr::Ref { .. }) {
            return true;
        }
        if let (HirExpr::Name { name, .. }, crate::RustExpr::Ident(lowered_name)) = (arg, lowered) {
            if lowered_name != name {
                return false;
            }
            return self.borrowed_params.contains(name) || self.mut_borrowed_params.contains(name);
        }
        false
    }

    pub(crate) fn arg_is_already_mut_borrowed_for_registry_call(
        &self,
        arg: &HirExpr,
        lowered: &crate::RustExpr,
    ) -> bool {
        if let crate::RustExpr::Ref { mutable, .. } = lowered {
            return *mutable;
        }
        if let (HirExpr::Name { name, .. }, crate::RustExpr::Ident(lowered_name)) = (arg, lowered) {
            if lowered_name != name {
                return false;
            }
            return self.mut_borrowed_params.contains(name);
        }
        false
    }
}
