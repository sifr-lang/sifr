#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::fs;
    use std::path::PathBuf;

    use super::{
        active_registry_entries, DiagnosticCode, DiagnosticState, ACTIVE_DIAGNOSTIC_CODES,
        DIAGNOSTIC_FAMILIES, DIAGNOSTIC_REGISTRY,
    };

    #[test]
    fn docs_url_is_derived_from_code() {
        assert_eq!(
            DiagnosticCode::TEST_SOURCE_ERROR.docs_url(),
            "https://sifr.sh/docs/errors/SIFR-NAME-9999"
        );
    }

    #[test]
    fn registry_skeleton_is_internally_consistent() {
        let families_by_name = families_by_name();
        let mut ids = BTreeSet::new();

        for entry in DIAGNOSTIC_REGISTRY {
            assert!(ids.insert(entry.id), "duplicate diagnostic id {}", entry.id);
            assert_canonical_code(entry.id);
            assert_eq!(entry.family, parse_family(entry.id));
            assert!(
                families_by_name.contains_key(entry.family),
                "unknown diagnostic family {} for {}",
                entry.family,
                entry.id
            );
            assert_dedupe_args_are_declared(entry);
            assert_template_placeholders_are_declared(entry);
            assert_registry_strings_are_markdown_safe(entry);

            match entry.state {
                DiagnosticState::Active => {
                    assert!(
                        entry.declared_severity.is_some(),
                        "active diagnostic {} must declare severity",
                        entry.id
                    );
                    assert!(
                        entry.owner_module.is_some(),
                        "active diagnostic {} must declare owner module",
                        entry.id
                    );
                    assert!(
                        entry.message_template.is_some(),
                        "active diagnostic {} must declare message template",
                        entry.id
                    );
                    assert!(
                        entry.representative_fixture_path.is_some(),
                        "active diagnostic {} must declare representative fixture path",
                        entry.id
                    );
                    assert!(
                        entry.docs_path == format!("docs/errors/{}.md", entry.id),
                        "active diagnostic {} must use its canonical docs page",
                        entry.id
                    );
                }
                DiagnosticState::Reserved => {
                    assert!(
                        entry.representative_fixture_path.is_none(),
                        "reserved diagnostic {} must not claim a fixture",
                        entry.id
                    );
                }
            }
        }

        for family in DIAGNOSTIC_FAMILIES {
            assert_family_name(family.name);
            assert_eq!(family.reserved_base, format!("SIFR-{}-0000", family.name));
            let base = registry_entry_for(family.reserved_base);
            assert_eq!(base.state, DiagnosticState::Reserved);
            assert_eq!(
                base.declared_severity, None,
                "reserved family base {} must not declare severity",
                base.id
            );
        }

        let active_ids: BTreeSet<_> = active_registry_entries().map(|entry| entry.id).collect();
        let constant_ids: BTreeSet<_> = ACTIVE_DIAGNOSTIC_CODES
            .iter()
            .map(|code| code.code())
            .collect();
        assert_eq!(
            active_ids, constant_ids,
            "active registry entries and DiagnosticCode constants must stay in sync"
        );

        for code in ACTIVE_DIAGNOSTIC_CODES {
            let entry = registry_entry_for(code.code());
            assert_eq!(
                entry.declared_severity,
                Some(code.declared_severity()),
                "DiagnosticCode severity must match registry severity for {}",
                code.code()
            );
        }
    }

    #[test]
    fn active_diagnostic_docs_pages_exist_with_exact_casing() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .expect("crate must live under workspace crates directory")
            .to_path_buf();
        let errors_dir = repo_root.join("docs/errors");
        let directory_entries = fs::read_dir(&errors_dir)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", errors_dir.display()))
            .map(|entry| {
                entry
                    .expect("failed to read docs/errors directory entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect::<BTreeSet<_>>();

        for entry in active_registry_entries() {
            let expected_file = format!("{}.md", entry.id);
            assert!(
                directory_entries.contains(&expected_file),
                "active diagnostic {} is missing exact docs page {}",
                entry.id,
                expected_file
            );
        }
    }

    fn registry_entry_for(id: &str) -> &'static super::DiagnosticRegistryEntry {
        super::registry_entry(id).unwrap_or_else(|| panic!("missing registry entry for {id}"))
    }

    fn families_by_name() -> BTreeMap<&'static str, &'static super::DiagnosticFamily> {
        DIAGNOSTIC_FAMILIES
            .iter()
            .map(|family| (family.name, family))
            .collect()
    }

    fn assert_family_name(name: &str) {
        assert!(
            (3..=12).contains(&name.len()),
            "family name {name} must be 3-12 ASCII letters"
        );
        assert!(
            name.bytes().all(|byte| byte.is_ascii_uppercase()),
            "family name {name} must contain uppercase ASCII letters only"
        );
    }

    fn assert_canonical_code(id: &str) {
        let mut parts = id.split('-');
        assert_eq!(parts.next(), Some("SIFR"));
        let family = parts.next().expect("diagnostic id must include family");
        let local = parts.next().expect("diagnostic id must include local code");
        assert!(parts.next().is_none(), "diagnostic id has too many parts");
        assert_family_name(family);
        assert_eq!(local.len(), 4, "diagnostic local code must be four digits");
        assert!(
            local.bytes().all(|byte| byte.is_ascii_digit()),
            "diagnostic local code must contain digits only"
        );
    }

    fn parse_family(id: &str) -> &str {
        id.split('-')
            .nth(1)
            .expect("canonical diagnostic id must include family")
    }

    fn assert_dedupe_args_are_declared(entry: &super::DiagnosticRegistryEntry) {
        let declared_args = entry
            .declared_args
            .iter()
            .map(|arg| arg.name)
            .collect::<BTreeSet<_>>();
        for dedupe_arg in entry.dedupe_args {
            assert!(
                declared_args.contains(dedupe_arg),
                "dedupe arg {dedupe_arg} is not declared for {}",
                entry.id
            );
        }
    }

    fn assert_template_placeholders_are_declared(entry: &super::DiagnosticRegistryEntry) {
        let Some(template) = entry.message_template else {
            return;
        };
        for placeholder in placeholders(template) {
            let declaration = entry
                .declared_args
                .iter()
                .find(|arg| arg.name == placeholder)
                .unwrap_or_else(|| {
                    panic!(
                        "template placeholder {{{placeholder}}} is not declared for {}",
                        entry.id
                    )
                });
            assert_eq!(
                declaration.format,
                super::DiagnosticArgFormat::MessageAndJson,
                "json-only arg {placeholder} must not appear in the message template for {}",
                entry.id
            );
        }
    }

    fn assert_registry_strings_are_markdown_safe(entry: &super::DiagnosticRegistryEntry) {
        for value in [
            entry.id,
            entry.family,
            entry.docs_path,
            entry.summary,
            entry.owner_module.unwrap_or_default(),
            entry.message_template.unwrap_or_default(),
            entry.representative_fixture_path.unwrap_or_default(),
        ] {
            assert!(
                !value.contains('`'),
                "registry string for {} must not contain backticks: {value}",
                entry.id
            );
        }
        for arg in entry.declared_args {
            assert!(
                !arg.name.contains('`'),
                "declared arg for {} must not contain backticks: {}",
                entry.id,
                arg.name
            );
        }
        for value in entry
            .dedupe_args
            .iter()
            .chain(entry.tooling.tool_actions.iter())
        {
            assert!(
                !value.contains('`'),
                "registry metadata for {} must not contain backticks: {value}",
                entry.id
            );
        }
    }

    fn placeholders(template: &str) -> Vec<String> {
        let mut placeholders = Vec::new();
        let mut chars = template.char_indices().peekable();

        while let Some((_, ch)) = chars.next() {
            if ch != '{' {
                continue;
            }
            if matches!(chars.peek(), Some((_, '{'))) {
                chars.next();
                continue;
            }

            let mut placeholder = String::new();
            let mut closed = false;
            for (_, next) in chars.by_ref() {
                if next == '}' {
                    closed = true;
                    break;
                }
                placeholder.push(next);
            }
            assert!(closed, "unclosed template placeholder in {template}");
            assert!(
                !placeholder.is_empty(),
                "empty template placeholder in {template}"
            );
            placeholders.push(placeholder);
        }

        placeholders
    }
}
