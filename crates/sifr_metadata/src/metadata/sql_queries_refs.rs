use super::{
    HirSqlBoundQuery, HirSqlCardinality, HirSqlEffectContract, HirSqlEffectKind, HirSqlExecution,
    HirSqlExecutionMethod, HirSqlParameterSlot, HirSqlQueryAdapter, HirSqlQueryTemplate, Record,
    RecordId, References, sealed,
};
impl References for HirSqlQueryAdapter {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::ExpectAtMostOne => {}
            Self::First => {}
        }
    }
}
impl sealed::Sealed for HirSqlQueryAdapter {}
impl Record for HirSqlQueryAdapter {
    const KIND: u16 = 52;
}
impl References for HirSqlEffectKind {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Read => {}
            Self::Write => {}
            Self::ReadWrite => {}
            Self::SchemaChange => {}
            Self::SessionChange => {}
            Self::TransactionControl => {}
        }
    }
}
impl sealed::Sealed for HirSqlEffectKind {}
impl Record for HirSqlEffectKind {
    const KIND: u16 = 45;
}
impl References for HirSqlCardinality {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.empty.references(out);
        self.minimum.references(out);
        self.maximum.references(out);
    }
}
impl sealed::Sealed for HirSqlCardinality {}
impl Record for HirSqlCardinality {
    const KIND: u16 = 43;
}
impl References for HirSqlEffectContract {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.effect.references(out);
        self.referenced_objects.references(out);
        self.affected_objects.references(out);
    }
}
impl sealed::Sealed for HirSqlEffectContract {}
impl Record for HirSqlEffectContract {
    const KIND: u16 = 44;
}
impl References for HirSqlParameterSlot {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.slot.references(out);
        self.ty.references(out);
    }
}
impl sealed::Sealed for HirSqlParameterSlot {}
impl Record for HirSqlParameterSlot {
    const KIND: u16 = 51;
}
impl References for HirSqlQueryTemplate {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.identity.references(out);
        self.module.references(out);
        self.symbol.references(out);
        self.profile_identity.references(out);
        self.profile_fingerprint.references(out);
        self.schema_fingerprint.references(out);
        self.normalized_statement.references(out);
        self.parameters.references(out);
        self.row_type.references(out);
        self.cardinality.references(out);
        self.effects.references(out);
        self.deterministic_order.references(out);
        self.fragment_identities.references(out);
        self.adapters.references(out);
        self.ty.references(out);
    }
}
impl sealed::Sealed for HirSqlQueryTemplate {}
impl Record for HirSqlQueryTemplate {
    const KIND: u16 = 53;
}
impl References for HirSqlBoundQuery {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.template_identity.references(out);
        self.profile_identity.references(out);
        self.profile_fingerprint.references(out);
        self.schema_fingerprint.references(out);
        self.captures.references(out);
        self.cardinality.references(out);
        self.effects.references(out);
        self.ty.references(out);
    }
}
impl sealed::Sealed for HirSqlBoundQuery {}
impl Record for HirSqlBoundQuery {
    const KIND: u16 = 42;
}
impl References for HirSqlExecutionMethod {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        match self {
            Self::Execute => {}
            Self::FetchOne => {}
            Self::FetchOptional => {}
            Self::FetchAll { maximum_rows } => {
                maximum_rows.references(out);
            }
            Self::Stream => {}
        }
    }
}
impl sealed::Sealed for HirSqlExecutionMethod {}
impl Record for HirSqlExecutionMethod {
    const KIND: u16 = 47;
}
impl References for HirSqlExecution {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.query.references(out);
        self.method.references(out);
        self.runtime_cardinality.references(out);
        self.runtime_effects.references(out);
    }
}
impl sealed::Sealed for HirSqlExecution {}
impl Record for HirSqlExecution {
    const KIND: u16 = 46;
}
