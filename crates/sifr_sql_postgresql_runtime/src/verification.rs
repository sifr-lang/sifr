use crate::config::{PostgresEvidence, PostgresProfile};
use crate::connection::PostgresNativeConnection;
use crate::error::{map_postgres_error, provider_error};
use sifr_sql_runtime::{SchemaDependencySlice, SchemaProperty, SqlError};

pub(crate) async fn observe_schema(
    native: &PostgresNativeConnection,
    profile: &PostgresProfile,
) -> Result<SchemaDependencySlice, SqlError> {
    match &profile.evidence {
        PostgresEvidence::Introspection {
            fingerprint_statement,
            probes,
        } => {
            let fingerprint = native
                .client
                .query_one(fingerprint_statement, &[])
                .await
                .map_err(|error| map_postgres_error(&error))?
                .try_get::<_, String>(0)
                .map_err(|error| map_postgres_error(&error))?;
            let mut properties = Vec::with_capacity(probes.len());
            for probe in probes {
                let row = native
                    .client
                    .query_opt(&probe.statement, &[])
                    .await
                    .map_err(|error| map_postgres_error(&error))?;
                let value = match row {
                    Some(row) => row
                        .try_get::<_, Option<String>>(0)
                        .map_err(|error| map_postgres_error(&error))?,
                    None => None,
                };
                properties.push(SchemaProperty::new(probe.property_identity.clone(), value)?);
            }
            SchemaDependencySlice::new(fingerprint, properties)
        }
        PostgresEvidence::MigrationHead {
            head_statement,
            accepted_states,
        } => {
            let rows = native
                .client
                .query(head_statement, &[])
                .await
                .map_err(|error| map_postgres_error(&error))?;
            let mut heads = rows
                .into_iter()
                .map(|row| {
                    row.try_get::<_, String>(0)
                        .map_err(|error| map_postgres_error(&error))
                })
                .collect::<Result<Vec<_>, _>>()?;
            heads.sort();
            let identity = heads.join(",");
            accepted_states
                .get(&identity)
                .cloned()
                .ok_or_else(provider_error)
        }
        PostgresEvidence::SignedManifest { manifest, verifier } => {
            PostgresEvidence::verify_manifest(verifier, manifest)
        }
    }
}
