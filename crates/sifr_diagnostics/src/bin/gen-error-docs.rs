#![allow(clippy::format_push_string)]

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use sifr_diagnostics::codes::{
    active_registry_entries, DiagnosticRegistryEntry, DiagnosticState, DIAGNOSTIC_FAMILIES,
    DIAGNOSTIC_REGISTRY,
};

struct GeneratedDocument {
    path: &'static str,
    contents: String,
}

fn main() -> io::Result<()> {
    let check = env::args().skip(1).any(|arg| arg == "--check");
    let repo_root = repo_root()?;
    let documents = generated_documents();

    if check {
        check_documents(&repo_root, &documents)?;
    } else {
        write_documents(&repo_root, &documents);
    }

    Ok(())
}

fn repo_root() -> io::Result<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "sifr_diagnostics must live under workspace crates directory",
            )
        })
}

fn generated_documents() -> Vec<GeneratedDocument> {
    let mut documents = vec![
        GeneratedDocument {
            path: "docs/errors/diagnostic-codes.md",
            contents: public_index(),
        },
        GeneratedDocument {
            path: "internal_docs/diagnostic_codes.md",
            contents: internal_reference(),
        },
    ];

    for entry in active_registry_entries() {
        documents.push(GeneratedDocument {
            path: entry.docs_path,
            contents: active_code_page(entry),
        });
    }

    documents
}

fn write_documents(repo_root: &Path, documents: &[GeneratedDocument]) {
    for document in documents {
        let path = repo_root.join(document.path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .unwrap_or_else(|err| panic!("failed to create {}: {err}", parent.display()));
        }
        fs::write(&path, document.contents.as_bytes())
            .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
    }
}

fn check_documents(repo_root: &Path, documents: &[GeneratedDocument]) -> io::Result<()> {
    let mut drift = Vec::new();
    for document in documents {
        let path = repo_root.join(document.path);
        match fs::read_to_string(&path) {
            Ok(existing) if existing == document.contents => {}
            Ok(_) => drift.push(format!("{} is out of date", document.path)),
            Err(err) => drift.push(format!("{} is missing or unreadable: {err}", document.path)),
        }
    }

    check_active_doc_casing(repo_root, &mut drift);

    if !drift.is_empty() {
        let mut stderr = io::stderr().lock();
        for message in drift {
            writeln!(stderr, "{message}")?;
        }
        std::process::exit(1);
    }
    Ok(())
}

fn check_active_doc_casing(repo_root: &Path, drift: &mut Vec<String>) {
    let errors_dir = repo_root.join("docs/errors");
    let Ok(entries) = fs::read_dir(&errors_dir) else {
        drift.push(format!("{} is missing", errors_dir.display()));
        return;
    };
    let mut actual_names = std::collections::BTreeSet::new();
    for entry in entries {
        match entry {
            Ok(entry) => {
                actual_names.insert(entry.file_name().to_string_lossy().into_owned());
            }
            Err(err) => drift.push(format!("failed to read docs/errors directory entry: {err}")),
        }
    }

    let mut expected_names = std::collections::BTreeSet::from(["diagnostic-codes.md".to_owned()]);
    for entry in active_registry_entries() {
        let expected = format!("{}.md", entry.id);
        expected_names.insert(expected.clone());
        if !actual_names.contains(&expected) {
            drift.push(format!(
                "docs/errors is missing exact active-code page casing for {expected}"
            ));
        }
    }
    for actual_name in actual_names {
        if Path::new(&actual_name)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
            && !expected_names.contains(&actual_name)
        {
            drift.push(format!(
                "docs/errors contains orphan generated diagnostic page {actual_name}"
            ));
        }
    }
}

fn public_index() -> String {
    let mut out = generated_header("Diagnostic Codes");
    out.push_str(
        "Sifr diagnostic codes use `SIFR-<FAMILY>-dddd`, with local numbers scoped to each family. Some interop families use hyphenated names such as `RUST-CONFIG`.\n\n",
    );
    out.push_str("Tooling metadata defaults: `tool_actions` is empty, `fix_all_eligible` is `false`, and machine-applicable suggestion availability is derived from emitted suggestions rather than authored registry metadata.\n\n");

    out.push_str("## Families\n\n");
    out.push_str("| Family | Reserved base | Summary |\n");
    out.push_str("| --- | --- | --- |\n");
    for family in DIAGNOSTIC_FAMILIES {
        out.push_str(&format!(
            "| `{}` | `{}` | {} |\n",
            family.name, family.reserved_base, family.summary
        ));
    }

    out.push_str("\n## Active Codes\n\n");
    let active_entries = active_registry_entries().collect::<Vec<_>>();
    if active_entries.is_empty() {
        out.push_str("No active public diagnostic codes are registered yet.\n");
    } else {
        out.push_str("| Code | Severity | Summary |\n");
        out.push_str("| --- | --- | --- |\n");
        for entry in active_entries {
            out.push_str(&format!(
                "| [`{}`]({}.md) | {} | {} |\n",
                entry.id,
                entry.id,
                severity(entry),
                entry.summary
            ));
        }
    }

    out.push_str("\n## Reserved Codes\n\n");
    out.push_str("| Code | Family | Summary |\n");
    out.push_str("| --- | --- | --- |\n");
    for entry in DIAGNOSTIC_REGISTRY
        .iter()
        .filter(|entry| entry.state == DiagnosticState::Reserved)
    {
        out.push_str(&format!(
            "| `{}` | `{}` | {} |\n",
            entry.id, entry.family, entry.summary
        ));
    }

    out
}

fn internal_reference() -> String {
    let mut out = generated_header("Diagnostic Code Registry");
    out.push_str("This file is generated from `crates/sifr_diagnostics/src/codes.rs`.\n\n");
    out.push_str("Codes use per-family local numbering. `SIFR-<FAMILY>-0000` is reserved as the family base and must never be emitted. Family names may be hyphenated for scoped interop domains such as `RUST-CONFIG`.\n\n");
    out.push_str("Registry states:\n\n");
    out.push_str("- `Active`: may have a `DiagnosticCode` constant and can be emitted.\n");
    out.push_str(
        "- `Reserved`: allocated for a future or structural purpose and cannot be emitted.\n",
    );
    out.push('\n');
    out.push_str("Tooling metadata defaults: `tool_actions` is empty, `fix_all_eligible` is `false`, and machine-applicable suggestion availability is derived from emitted suggestions rather than authored manually.\n\n");

    out.push_str("## Families\n\n");
    out.push_str("| Family | Reserved base | Summary |\n");
    out.push_str("| --- | --- | --- |\n");
    for family in DIAGNOSTIC_FAMILIES {
        out.push_str(&format!(
            "| `{}` | `{}` | {} |\n",
            family.name, family.reserved_base, family.summary
        ));
    }

    out.push_str("\n## Registry\n\n");
    out.push_str("| ID | Family | State | Severity | Docs path | Fixture | Owner | Template | Declared args | Dedupe args | Tool actions | Fix all |\n");
    out.push_str("| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |\n");
    for entry in DIAGNOSTIC_REGISTRY.iter() {
        out.push_str(&format!(
            "| `{}` | `{}` | {} | {} | `{}` | {} | {} | {} | {} | {} | {} | {} |\n",
            entry.id,
            entry.family,
            entry.state.as_str(),
            severity(entry),
            entry.docs_path,
            optional_code(entry.representative_fixture_path),
            optional_code(entry.owner_module),
            optional_code(entry.message_template),
            declared_args(entry),
            string_list(entry.dedupe_args),
            string_list(entry.tooling.tool_actions),
            entry.tooling.fix_all_eligible
        ));
    }

    out
}

fn active_code_page(entry: &DiagnosticRegistryEntry) -> String {
    let mut out = generated_header(entry.id);
    out.push_str(&format!("{}\n\n", entry.summary));
    out.push_str("| Field | Value |\n");
    out.push_str("| --- | --- |\n");
    out.push_str(&format!("| Code | `{}` |\n", entry.id));
    out.push_str(&format!("| Family | `{}` |\n", entry.family));
    out.push_str(&format!("| Severity | {} |\n", severity(entry)));
    out.push_str(&format!(
        "| Owner | {} |\n",
        optional_code(entry.owner_module)
    ));
    out.push_str(&format!(
        "| Message template | {} |\n",
        optional_code(entry.message_template)
    ));
    out.push_str(&format!(
        "| Representative fixture | {} |\n",
        optional_code(entry.representative_fixture_path)
    ));
    out.push_str(&format!("| Declared args | {} |\n", declared_args(entry)));
    out.push_str(&format!(
        "| Dedupe args | {} |\n",
        string_list(entry.dedupe_args)
    ));
    out
}

fn generated_header(title: &str) -> String {
    format!(
        "# {title}\n\n<!-- Generated by cargo run -p sifr_diagnostics --bin gen-error-docs. Do not edit by hand. -->\n\n"
    )
}

fn severity(entry: &DiagnosticRegistryEntry) -> String {
    entry
        .declared_severity
        .map_or_else(|| "n/a".to_owned(), |severity| format!("{severity:?}"))
}

fn optional_code(value: Option<&str>) -> String {
    value.map_or_else(
        || "n/a".to_owned(),
        |value| format!("`{}`", escape_table(value)),
    )
}

fn declared_args(entry: &DiagnosticRegistryEntry) -> String {
    if entry.declared_args.is_empty() {
        return "n/a".to_owned();
    }
    entry
        .declared_args
        .iter()
        .map(|arg| format!("`{} ({})`", escape_table(arg.name), arg.format.as_str()))
        .collect::<Vec<_>>()
        .join(", ")
}

fn string_list(values: &[&str]) -> String {
    if values.is_empty() {
        return "n/a".to_owned();
    }
    values
        .iter()
        .map(|value| format!("`{}`", escape_table(value)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn escape_table(value: &str) -> String {
    value.replace('|', "\\|")
}
