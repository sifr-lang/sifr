use crate::{
    GeneratedProfileModule, ProfileAuthority, SchemaContractError, SchemaContractErrorKind,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

#[derive(Clone, Debug)]
pub struct RegisteredProfileModule {
    authority: ProfileAuthority,
    module: GeneratedProfileModule,
}

impl RegisteredProfileModule {
    #[must_use]
    pub fn authority(&self) -> &ProfileAuthority {
        &self.authority
    }

    #[must_use]
    pub fn module(&self) -> &GeneratedProfileModule {
        &self.module
    }
}

/// Canonical production registry for generated schema-profile modules.
///
/// The registry indexes the same immutable entry by source profile name,
/// generated module path, and nominal profile identity. Query compilation must
/// resolve through this registry instead of reconstructing profile metadata
/// from cache-key text.
#[derive(Clone, Debug, Default)]
pub struct ProfileModuleRegistry {
    entries: BTreeMap<String, RegisteredProfileModule>,
    module_paths: BTreeMap<String, String>,
    nominal_identities: BTreeMap<String, String>,
}

impl ProfileModuleRegistry {
    pub fn register(
        &mut self,
        authority: ProfileAuthority,
        module: GeneratedProfileModule,
    ) -> Result<(), SchemaContractError> {
        validate_entry(&authority, &module)?;
        let profile_name = authority.profile.name.clone();
        if self.entries.contains_key(&profile_name)
            || self.module_paths.contains_key(&module.module_path)
            || self
                .nominal_identities
                .contains_key(&authority.nominal_identity)
        {
            return Err(SchemaContractError::new(
                SchemaContractErrorKind::InvalidProfile,
                format!("SQL profile module '{profile_name}' has a duplicate registry identity"),
            ));
        }
        self.module_paths
            .insert(module.module_path.clone(), profile_name.clone());
        self.nominal_identities
            .insert(authority.nominal_identity.clone(), profile_name.clone());
        self.entries
            .insert(profile_name, RegisteredProfileModule { authority, module });
        Ok(())
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn profile(&self, name: &str) -> Result<&RegisteredProfileModule, SchemaContractError> {
        self.entries.get(name).ok_or_else(|| unknown_profile(name))
    }

    pub fn module_path(
        &self,
        module_path: &str,
    ) -> Result<&RegisteredProfileModule, SchemaContractError> {
        let name = self
            .module_paths
            .get(module_path)
            .ok_or_else(|| unknown_profile(module_path))?;
        self.profile(name)
    }

    pub fn nominal_identity(
        &self,
        identity: &str,
    ) -> Result<&RegisteredProfileModule, SchemaContractError> {
        let name = self
            .nominal_identities
            .get(identity)
            .ok_or_else(|| unknown_profile(identity))?;
        self.profile(name)
    }

    pub fn entries(&self) -> impl ExactSizeIterator<Item = (&str, &RegisteredProfileModule)> + '_ {
        self.entries
            .iter()
            .map(|(name, entry)| (name.as_str(), entry))
    }

    #[must_use]
    pub fn cache_fragment(&self) -> String {
        let mut fragment = String::new();
        for (name, entry) in &self.entries {
            let _ = writeln!(
                fragment,
                "{name}\t{}\t{}\t{}",
                entry.authority.profile_fingerprint.as_str(),
                entry.authority.schema_fingerprint.as_str(),
                entry.module.module_path,
            );
        }
        fragment
    }
}

fn validate_entry(
    authority: &ProfileAuthority,
    module: &GeneratedProfileModule,
) -> Result<(), SchemaContractError> {
    let metadata = &module.metadata;
    let expected_path = format!("sifr.sql.schemas.{}", authority.profile.name);
    let expected_symbols = authority
        .profile
        .schema
        .objects
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    if metadata.profile_name != authority.profile.name
        || metadata.nominal_identity != authority.nominal_identity
        || metadata.profile_fingerprint != authority.profile_fingerprint.as_str()
        || metadata.schema_fingerprint != authority.schema_fingerprint.as_str()
        || metadata.schema_symbols != expected_symbols
        || metadata.schema_witness.export_name != "schema"
        || metadata.schema_witness.profile_identity != authority.nominal_identity
        || metadata.schema_witness.type_name != format!("SqlSchema[{}]", authority.nominal_identity)
        || module.module_path != expected_path
    {
        return Err(SchemaContractError::new(
            SchemaContractErrorKind::InvalidProfile,
            format!(
                "generated SQL profile module '{}' does not match its authority",
                authority.profile.name
            ),
        ));
    }
    Ok(())
}

fn unknown_profile(identity: &str) -> SchemaContractError {
    SchemaContractError::new(
        SchemaContractErrorKind::UnknownSymbol,
        format!("SQL profile module '{identity}' is not registered"),
    )
}
