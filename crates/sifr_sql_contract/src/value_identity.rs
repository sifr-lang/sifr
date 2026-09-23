use crate::SifrType;

/// One source annotation and frontend identity for every nominal SQL value.
/// The same identities are used when a query input is recovered from a Sifr type
/// and when a provider result is lowered into a Sifr type.
#[derive(Clone, Debug)]
pub struct SqlValueIdentity {
    pub sifr_type: SifrType,
    pub import_module: &'static str,
    pub annotation: &'static str,
    pub frontend_identity: &'static str,
}

pub const SQL_VALUE_IDENTITIES: &[SqlValueIdentity] = &[
    SqlValueIdentity {
        sifr_type: SifrType::Numeric,
        import_module: "sifr.sql",
        annotation: "Numeric",
        frontend_identity: "sifr.sql.Numeric",
    },
    SqlValueIdentity {
        sifr_type: SifrType::Date,
        import_module: "sifr.datetime",
        annotation: "date",
        frontend_identity: "sifr.datetime.date",
    },
    SqlValueIdentity {
        sifr_type: SifrType::LocalTime,
        import_module: "sifr.datetime",
        annotation: "time",
        frontend_identity: "sifr.datetime.time",
    },
    SqlValueIdentity {
        sifr_type: SifrType::OffsetTime,
        import_module: "sifr.sql",
        annotation: "OffsetTime",
        frontend_identity: "sifr.sql.OffsetTime",
    },
    SqlValueIdentity {
        sifr_type: SifrType::LocalDateTime,
        import_module: "sifr.datetime",
        annotation: "datetime",
        frontend_identity: "sifr.datetime.datetime",
    },
    SqlValueIdentity {
        sifr_type: SifrType::Instant,
        import_module: "sifr.sql",
        annotation: "Instant",
        frontend_identity: "sifr.sql.Instant",
    },
    SqlValueIdentity {
        sifr_type: SifrType::CalendarInterval,
        import_module: "sifr.sql",
        annotation: "CalendarInterval",
        frontend_identity: "sifr.sql.CalendarInterval",
    },
    SqlValueIdentity {
        sifr_type: SifrType::Uuid,
        import_module: "sifr.uuid",
        annotation: "UUID",
        frontend_identity: "sifr.uuid.UUID",
    },
    SqlValueIdentity {
        sifr_type: SifrType::JsonValue,
        import_module: "sifr.json",
        annotation: "JsonValue",
        frontend_identity: "sifr.json.JsonValue",
    },
    SqlValueIdentity {
        sifr_type: SifrType::IpAddress,
        import_module: "sifr.sql",
        annotation: "IPAddress",
        frontend_identity: "sifr.sql.IPAddress",
    },
    SqlValueIdentity {
        sifr_type: SifrType::IpNetwork,
        import_module: "sifr.sql",
        annotation: "IPNetwork",
        frontend_identity: "sifr.sql.IPNetwork",
    },
    SqlValueIdentity {
        sifr_type: SifrType::MacAddress,
        import_module: "sifr.sql",
        annotation: "MacAddress",
        frontend_identity: "sifr.sql.MacAddress",
    },
];

#[must_use]
pub fn sql_value_identity(ty: &SifrType) -> Option<&'static SqlValueIdentity> {
    SQL_VALUE_IDENTITIES
        .iter()
        .find(|entry| &entry.sifr_type == ty)
}

#[must_use]
pub fn sql_value_type_for_frontend_identity(identity: &str) -> Option<SifrType> {
    SQL_VALUE_IDENTITIES
        .iter()
        .find(|entry| entry.frontend_identity == identity)
        .map(|entry| entry.sifr_type.clone())
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DatabaseType, canonical_read_type, generated_sifr_type_name};

    #[test]
    fn sql_value_identities_are_closed_and_reversible() {
        let mut source_names = std::collections::BTreeSet::new();
        let mut frontend_names = std::collections::BTreeSet::new();
        for identity in SQL_VALUE_IDENTITIES {
            assert!(source_names.insert((identity.import_module, identity.annotation)));
            assert!(frontend_names.insert(identity.frontend_identity));
            assert_eq!(
                sql_value_identity(&identity.sifr_type).map(|found| found.frontend_identity),
                Some(identity.frontend_identity)
            );
            assert_eq!(
                sql_value_type_for_frontend_identity(identity.frontend_identity),
                Some(identity.sifr_type.clone())
            );
            assert_eq!(
                generated_sifr_type_name(&identity.sifr_type)
                    .ok()
                    .as_deref(),
                Some(identity.annotation)
            );
        }
        for database in [
            DatabaseType::Date,
            DatabaseType::LocalTime { precision: 6 },
            DatabaseType::OffsetTime { precision: 6 },
            DatabaseType::LocalDateTime { precision: 6 },
            DatabaseType::Instant { precision: 6 },
            DatabaseType::CalendarInterval,
            DatabaseType::Uuid,
            DatabaseType::Json { binary: true },
            DatabaseType::IpAddress,
            DatabaseType::IpNetwork,
            DatabaseType::MacAddress,
        ] {
            let sifr = canonical_read_type(&database).expect("standard SQL value");
            assert!(sql_value_identity(&sifr).is_some(), "{database:?}");
        }
    }
}
