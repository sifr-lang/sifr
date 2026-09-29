use crate::{HirExpr, RustEmitter, RustExpr, Type};
use sifr_ir::{MutableArgumentTarget, MutableReceiverTarget, Place, PlaceProjection};
use sifr_type_system::{ParamConvention, ReceiverConvention};

#[derive(Clone, Copy)]
pub(crate) struct MethodCallPlaces<'a> {
    pub(crate) receiver_convention: Option<ReceiverConvention>,
    pub(crate) receiver_target: Option<&'a MutableReceiverTarget>,
    pub(crate) mutable_arg_places: &'a [Option<MutableArgumentTarget>],
}

impl<'a> MethodCallPlaces<'a> {
    pub(crate) fn new(
        receiver_convention: Option<ReceiverConvention>,
        receiver_target: Option<&'a MutableReceiverTarget>,
        mutable_arg_places: &'a [Option<MutableArgumentTarget>],
    ) -> Self {
        Self {
            receiver_convention,
            receiver_target,
            mutable_arg_places,
        }
    }
}

impl RustEmitter {
    /// Emit a field path used as an explicitly resolved shared method receiver.
    ///
    /// Shared receiver calls borrow their original storage, while an ordinary
    /// field value expression still follows Sifr's clone/value semantics.
    pub(crate) fn emit_shared_receiver_path(&mut self, expression: &HirExpr) -> Option<RustExpr> {
        match expression {
            HirExpr::Name { name, .. } => Some(RustExpr::Ident(name.clone())),
            HirExpr::FieldAccess { object, field, .. } => {
                let lowered_object = self.emit_shared_receiver_path(object)?;
                Some(self.lower_field_storage_access(object, field, lowered_object))
            }
            _ => None,
        }
    }

    /// Emit an assignment/delete storage path whose mutability was proven by statement lowering.
    pub(crate) fn emit_storage_path(&mut self, expression: &HirExpr) -> Option<RustExpr> {
        match expression {
            HirExpr::Name { name, .. } => {
                if name != "self" {
                    self.protected_mutable_place_roots.insert(name.clone());
                }
                Some(RustExpr::Ident(name.clone()))
            }
            HirExpr::FieldAccess { object, field, .. } => {
                let lowered_object = self.emit_storage_path(object)?;
                Some(self.lower_field_storage_access(object, field, lowered_object))
            }
            _ => None,
        }
    }

    /// Emit storage proven by lowering without entering ordinary field-value lowering.
    pub(crate) fn emit_checked_place(
        &mut self,
        expression: &HirExpr,
        place: &Place,
    ) -> Option<RustExpr> {
        self.emit_checked_place_projection(expression, place, place.projections.len())
    }

    fn emit_checked_place_projection(
        &mut self,
        expression: &HirExpr,
        place: &Place,
        projection_count: usize,
    ) -> Option<RustExpr> {
        match expression {
            HirExpr::Name {
                name,
                binding_id: Some(binding_id),
                ..
            } if *binding_id == place.root && projection_count == 0 => {
                if name != "self" {
                    self.protected_mutable_place_roots.insert(name.clone());
                }
                Some(RustExpr::Ident(name.clone()))
            }
            HirExpr::FieldAccess { object, field, .. } if projection_count > 0 => {
                let PlaceProjection::Field(identity) = &place.projections[projection_count - 1];
                if identity.field != *field {
                    return None;
                }
                let lowered_object =
                    self.emit_checked_place_projection(object, place, projection_count - 1)?;
                Some(self.lower_field_storage_access(object, field, lowered_object))
            }
            _ => None,
        }
    }

    pub(crate) fn lower_method_receiver_place_for_stmt(
        &mut self,
        object: &HirExpr,
        convention: Option<ReceiverConvention>,
        target: Option<&MutableReceiverTarget>,
    ) -> Result<Option<RustExpr>, crate::CodegenError> {
        match convention {
            Some(ReceiverConvention::MutableBorrow) => match target {
                Some(MutableReceiverTarget::Place(place)) => Ok(self
                    .emit_checked_place(object, place)
                    .map(|lowered| self.explicit_class_receiver_borrow(object, lowered))),
                Some(MutableReceiverTarget::OwnedTemporary) => self.lower_stmt_expr_for_ir(object),
                Some(MutableReceiverTarget::SpecializedIndexedStorage(_)) => Ok(None),
                None => Ok(None),
            },
            Some(ReceiverConvention::SharedBorrow) => {
                if let HirExpr::Index {
                    object: collection,
                    index,
                    ..
                } = object
                    && let Some(witness) = self.checked_place_read_borrow_witness(collection, index)
                {
                    return Ok(Some(witness));
                }
                if let Some(path) = self.emit_shared_receiver_path(object) {
                    return Ok(Some(path));
                }
                self.lower_stmt_expr_for_ir(object)
            }
            Some(ReceiverConvention::Owned | ReceiverConvention::OwnedMutable) => {
                self.lower_stmt_expr_for_ir(object)
            }
            None => Ok(None),
        }
    }

    pub(crate) fn lower_method_argument_place_for_stmt(
        &mut self,
        argument: &HirExpr,
        convention: ParamConvention,
        target: Option<&MutableArgumentTarget>,
    ) -> Result<Option<RustExpr>, crate::CodegenError> {
        if convention.is_mut_borrow() {
            return match target {
                Some(MutableArgumentTarget::Place(place)) => {
                    Ok(self.emit_checked_place(argument, place))
                }
                Some(MutableArgumentTarget::OwnedTemporary) => {
                    self.lower_stmt_expr_for_ir(argument)
                }
                None => Ok(None),
            };
        }
        let lowered = self.lower_stmt_expr_for_ir(argument)?;
        Ok(lowered.map(|expr| {
            self.clone_borrowed_argument_for_owned_convention(argument, convention, expr)
        }))
    }

    pub(crate) fn lower_call_argument_for_stmt(
        &mut self,
        argument: &HirExpr,
        target: Option<&MutableArgumentTarget>,
    ) -> Result<Option<RustExpr>, crate::CodegenError> {
        match target {
            Some(MutableArgumentTarget::Place(place)) => {
                Ok(self.emit_checked_place(argument, place))
            }
            Some(MutableArgumentTarget::OwnedTemporary) | None => {
                self.lower_stmt_expr_for_ir(argument)
            }
        }
    }

    pub(crate) fn lower_method_receiver_place_for_registry(
        &mut self,
        object: &HirExpr,
        convention: Option<ReceiverConvention>,
        target: Option<&MutableReceiverTarget>,
    ) -> Option<RustExpr> {
        match convention {
            Some(ReceiverConvention::MutableBorrow) => match target {
                Some(MutableReceiverTarget::Place(place)) => self
                    .emit_checked_place(object, place)
                    .map(|lowered| self.explicit_class_receiver_borrow(object, lowered)),
                Some(MutableReceiverTarget::OwnedTemporary) => {
                    self.try_lower_registry_expr_strict(object)
                }
                Some(MutableReceiverTarget::SpecializedIndexedStorage(_)) => None,
                None => None,
            },
            Some(ReceiverConvention::SharedBorrow) => {
                if let HirExpr::Index {
                    object: collection,
                    index,
                    ..
                } = object
                    && let Some(witness) = self.checked_place_read_borrow_witness(collection, index)
                {
                    return Some(witness);
                }
                self.emit_shared_receiver_path(object)
                    .or_else(|| self.try_lower_registry_expr_strict(object))
            }
            Some(ReceiverConvention::Owned | ReceiverConvention::OwnedMutable) => {
                self.try_lower_registry_expr_strict(object)
            }
            None => None,
        }
    }

    pub(crate) fn lower_method_argument_place_for_registry(
        &mut self,
        argument: &HirExpr,
        convention: ParamConvention,
        target: Option<&MutableArgumentTarget>,
    ) -> Option<RustExpr> {
        if convention.is_mut_borrow() {
            return match target {
                Some(MutableArgumentTarget::Place(place)) => {
                    self.emit_checked_place(argument, place)
                }
                Some(MutableArgumentTarget::OwnedTemporary) => {
                    self.try_lower_registry_expr_strict(argument)
                }
                None => None,
            };
        }
        let lowered = self.try_lower_registry_expr_strict(argument)?;
        Some(self.clone_borrowed_argument_for_owned_convention(argument, convention, lowered))
    }

    fn clone_borrowed_argument_for_owned_convention(
        &self,
        argument: &HirExpr,
        convention: ParamConvention,
        lowered: RustExpr,
    ) -> RustExpr {
        if convention.is_owned()
            && matches!(argument, HirExpr::Name { name, ty, .. }
                if (self.borrowed_params.contains(name) || self.mut_borrowed_params.contains(name))
                    && !crate::helpers::is_copy_type_for_codegen(ty))
        {
            crate::ownership_plan::materialize_owned_value(argument.ty(), lowered)
        } else {
            lowered
        }
    }

    // Imported methods have no declaration in the consumer's Rust file. Keep
    // the resolved mutable receiver contract explicit across that boundary.
    fn explicit_class_receiver_borrow(&self, source: &HirExpr, lowered: RustExpr) -> RustExpr {
        let Type::Class {
            identity: Some(identity),
            ..
        } = source.ty().resolve_alias()
        else {
            return lowered;
        };
        let Some((owner, _)) = identity.rsplit_once('.') else {
            return lowered;
        };
        if self
            .current_module_name
            .as_deref()
            .is_none_or(|module| module == owner)
        {
            return lowered;
        }
        let borrowed = matches!(source, HirExpr::Name { name, .. }
            if name == "self" || self.mut_borrowed_params.contains(name));
        RustExpr::Ref {
            mutable: true,
            expr: Box::new(if borrowed {
                RustExpr::Deref(Box::new(lowered))
            } else {
                lowered
            }),
        }
    }

    pub(crate) fn lower_field_storage_access(
        &self,
        object: &HirExpr,
        field: &str,
        lowered_object: RustExpr,
    ) -> RustExpr {
        let class_name = if matches!(object, HirExpr::Name { name, .. } if name == "self") {
            self.current_class_name.clone()
        } else {
            match crate::resolve_alias_type_for_plain_call(object.ty()) {
                Type::Class { name, .. } => Some(name.clone()),
                _ => None,
            }
        };
        self.lower_field_storage_access_for_class(class_name.as_deref(), field, lowered_object)
    }

    pub(crate) fn lower_field_storage_access_for_class(
        &self,
        class_name: Option<&str>,
        field: &str,
        lowered_object: RustExpr,
    ) -> RustExpr {
        let parent_name = class_name
            .and_then(|class_name| self.parent_fields.get(class_name))
            .filter(|(_, parent_fields)| parent_fields.contains(field))
            .map(|(parent_name, _)| parent_name);
        let lowered_base = if let Some(parent_name) = parent_name {
            RustExpr::Field {
                expr: Box::new(lowered_object),
                field: parent_name.to_lowercase(),
            }
        } else {
            lowered_object
        };
        RustExpr::Field {
            expr: Box::new(lowered_base),
            field: field.to_string(),
        }
    }
}
