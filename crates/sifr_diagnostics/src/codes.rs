use crate::model::Severity;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DiagnosticCode {
    code: &'static str,
    declared_severity: Severity,
}

impl DiagnosticCode {
    #[cfg(test)]
    pub(crate) const TEST_INTERNAL_ERROR: Self = Self::new("SIFR-INTERNAL-9998", Severity::Error);
    #[cfg(test)]
    pub(crate) const TEST_NOTE: Self = Self::new("SIFR-INTERNAL-9999", Severity::Note);
    #[cfg(test)]
    pub(crate) const TEST_SOURCE_ERROR: Self = Self::new("SIFR-NAME-9999", Severity::Error);

    #[cfg(test)]
    const fn new(code: &'static str, declared_severity: Severity) -> Self {
        Self {
            code,
            declared_severity,
        }
    }

    #[must_use]
    pub const fn code(self) -> &'static str {
        self.code
    }

    #[must_use]
    pub const fn declared_severity(self) -> Severity {
        self.declared_severity
    }

    #[must_use]
    pub fn docs_url(self) -> String {
        format!("https://sifr.sh/docs/errors/{}", self.code())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DiagnosticState {
    Active,
    Reserved,
    Retired,
}

impl DiagnosticState {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "Active",
            Self::Reserved => "Reserved",
            Self::Retired => "Retired",
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DiagnosticArgFormat {
    MessageAndJson,
    JsonOnly,
}

impl DiagnosticArgFormat {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MessageAndJson => "message+json",
            Self::JsonOnly => "json-only",
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DiagnosticArgDeclaration {
    pub name: &'static str,
    pub format: DiagnosticArgFormat,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DiagnosticTooling {
    pub tool_actions: &'static [&'static str],
    pub fix_all_eligible: bool,
}

impl DiagnosticTooling {
    pub const DEFAULT: Self = Self {
        tool_actions: &[],
        fix_all_eligible: false,
    };
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DiagnosticFamily {
    pub name: &'static str,
    pub summary: &'static str,
    pub reserved_base: &'static str,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DiagnosticRegistryEntry {
    pub id: &'static str,
    pub family: &'static str,
    pub summary: &'static str,
    pub state: DiagnosticState,
    pub docs_path: &'static str,
    pub representative_fixture_path: Option<&'static str>,
    pub message_template: Option<&'static str>,
    pub owner_module: Option<&'static str>,
    pub declared_args: &'static [DiagnosticArgDeclaration],
    pub dedupe_args: &'static [&'static str],
    pub declared_severity: Option<Severity>,
    pub tooling: DiagnosticTooling,
}

pub const DIAGNOSTIC_FAMILIES: &[DiagnosticFamily] = &[
    DiagnosticFamily {
        name: "PARSE",
        summary: "Parsing and source syntax diagnostics.",
        reserved_base: "SIFR-PARSE-0000",
    },
    DiagnosticFamily {
        name: "NAME",
        summary: "Name binding and resolution diagnostics.",
        reserved_base: "SIFR-NAME-0000",
    },
    DiagnosticFamily {
        name: "IMPORT",
        summary: "Module import and path resolution diagnostics.",
        reserved_base: "SIFR-IMPORT-0000",
    },
    DiagnosticFamily {
        name: "TYPE",
        summary: "Static type compatibility and inference diagnostics.",
        reserved_base: "SIFR-TYPE-0000",
    },
    DiagnosticFamily {
        name: "DECIMAL",
        summary: "Decimal literal and fixed-point arithmetic diagnostics.",
        reserved_base: "SIFR-DECIMAL-0000",
    },
    DiagnosticFamily {
        name: "CALL",
        summary: "Function, method, constructor, and overload call diagnostics.",
        reserved_base: "SIFR-CALL-0000",
    },
    DiagnosticFamily {
        name: "OWN",
        summary: "Ownership, borrow, move, and lifetime diagnostics.",
        reserved_base: "SIFR-OWN-0000",
    },
    DiagnosticFamily {
        name: "FLOW",
        summary: "Control-flow, reachability, and narrowing diagnostics.",
        reserved_base: "SIFR-FLOW-0000",
    },
    DiagnosticFamily {
        name: "MATCH",
        summary: "Pattern matching and exhaustiveness diagnostics.",
        reserved_base: "SIFR-MATCH-0000",
    },
    DiagnosticFamily {
        name: "PROTO",
        summary: "Protocol and structural conformance diagnostics.",
        reserved_base: "SIFR-PROTO-0000",
    },
    DiagnosticFamily {
        name: "CLASS",
        summary: "Class declaration, constructor, field, and method diagnostics.",
        reserved_base: "SIFR-CLASS-0000",
    },
    DiagnosticFamily {
        name: "RESULT",
        summary: "Result, Option, and checked error-flow diagnostics.",
        reserved_base: "SIFR-RESULT-0000",
    },
    DiagnosticFamily {
        name: "STDLIB",
        summary: "Standard-library surface and intrinsic contract diagnostics.",
        reserved_base: "SIFR-STDLIB-0000",
    },
    DiagnosticFamily {
        name: "WORKSPACE",
        summary: "Workspace, package, manifest, and project discovery diagnostics.",
        reserved_base: "SIFR-WORKSPACE-0000",
    },
    DiagnosticFamily {
        name: "CODEGEN",
        summary: "Rust lowering and backend code-generation diagnostics.",
        reserved_base: "SIFR-CODEGEN-0000",
    },
    DiagnosticFamily {
        name: "BUILD",
        summary: "Build orchestration, rustc, linker, and artifact diagnostics.",
        reserved_base: "SIFR-BUILD-0000",
    },
    DiagnosticFamily {
        name: "INTERNAL",
        summary: "Compiler invariant and internal failure diagnostics.",
        reserved_base: "SIFR-INTERNAL-0000",
    },
];

pub const DIAGNOSTIC_REGISTRY: &[DiagnosticRegistryEntry] = &[
    reserved_family_base("SIFR-PARSE-0000", "PARSE"),
    reserved_family_base("SIFR-NAME-0000", "NAME"),
    reserved_family_base("SIFR-IMPORT-0000", "IMPORT"),
    reserved_family_base("SIFR-TYPE-0000", "TYPE"),
    reserved_family_base("SIFR-DECIMAL-0000", "DECIMAL"),
    reserved_family_base("SIFR-CALL-0000", "CALL"),
    reserved_family_base("SIFR-OWN-0000", "OWN"),
    reserved_family_base("SIFR-FLOW-0000", "FLOW"),
    reserved_family_base("SIFR-MATCH-0000", "MATCH"),
    reserved_family_base("SIFR-PROTO-0000", "PROTO"),
    reserved_family_base("SIFR-CLASS-0000", "CLASS"),
    reserved_family_base("SIFR-RESULT-0000", "RESULT"),
    reserved_family_base("SIFR-STDLIB-0000", "STDLIB"),
    reserved_family_base("SIFR-WORKSPACE-0000", "WORKSPACE"),
    reserved_family_base("SIFR-CODEGEN-0000", "CODEGEN"),
    reserved_family_base("SIFR-BUILD-0000", "BUILD"),
    reserved_family_base("SIFR-INTERNAL-0000", "INTERNAL"),
    DiagnosticRegistryEntry {
        id: "SIFR-INTERNAL-0001",
        family: "INTERNAL",
        summary: "Reserved for unclassified compiler panics after a panic boundary.",
        state: DiagnosticState::Reserved,
        docs_path: "docs/errors/diagnostic-codes.md#sifr-internal-0001",
        representative_fixture_path: None,
        message_template: None,
        owner_module: Some("compiler panic boundary"),
        declared_args: &[],
        dedupe_args: &[],
        declared_severity: Some(Severity::Error),
        tooling: DiagnosticTooling::DEFAULT,
    },
    DiagnosticRegistryEntry {
        id: "SIFR-INTERNAL-0002",
        family: "INTERNAL",
        summary: "Reserved for structured recovery-cap omission summaries.",
        state: DiagnosticState::Reserved,
        docs_path: "docs/errors/diagnostic-codes.md#sifr-internal-0002",
        representative_fixture_path: None,
        message_template: None,
        owner_module: Some("diagnostic recovery cap"),
        declared_args: &[],
        dedupe_args: &[],
        declared_severity: Some(Severity::Note),
        tooling: DiagnosticTooling::DEFAULT,
    },
];

pub const ACTIVE_DIAGNOSTIC_CODES: &[DiagnosticCode] = &[];

#[must_use]
pub fn registry_entry(id: &str) -> Option<&'static DiagnosticRegistryEntry> {
    DIAGNOSTIC_REGISTRY.iter().find(|entry| entry.id == id)
}

pub fn active_registry_entries() -> impl Iterator<Item = &'static DiagnosticRegistryEntry> {
    DIAGNOSTIC_REGISTRY
        .iter()
        .filter(|entry| entry.state == DiagnosticState::Active)
}

const fn reserved_family_base(id: &'static str, family: &'static str) -> DiagnosticRegistryEntry {
    DiagnosticRegistryEntry {
        id,
        family,
        summary: "Reserved family base; not emitted as a diagnostic.",
        state: DiagnosticState::Reserved,
        docs_path: "docs/errors/diagnostic-codes.md",
        representative_fixture_path: None,
        message_template: None,
        owner_module: None,
        declared_args: &[],
        dedupe_args: &[],
        declared_severity: None,
        tooling: DiagnosticTooling::DEFAULT,
    }
}

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
                DiagnosticState::Retired => {}
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
