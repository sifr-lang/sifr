#![allow(clippy::format_push_string)]

#[path = "gen_error_docs/family_docs.rs"]
mod family_docs;

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use family_docs::family_docs;
use serde_json::{Map, Value};
use sifr_diagnostics::codes::{
    active_registry_entries, DiagnosticRegistryEntry, DiagnosticState, DIAGNOSTIC_FAMILIES,
    DIAGNOSTIC_REGISTRY,
};

struct GeneratedDocument {
    path: String,
    contents: String,
}

fn main() -> io::Result<()> {
    let check = env::args().skip(1).any(|arg| arg == "--check");
    let repo_root = repo_root()?;
    let documents = generated_documents(&repo_root);

    if check {
        check_documents(&repo_root, &documents)?;
        check_docs_json_reference_nav(&repo_root)?;
    } else {
        write_documents(&repo_root, &documents);
        remove_obsolete_markdown_stubs(&repo_root);
        sync_docs_json_reference_nav(&repo_root)?;
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

fn generated_documents(repo_root: &Path) -> Vec<GeneratedDocument> {
    let mut documents = vec![
        GeneratedDocument {
            path: "docs/errors/diagnostic-codes.md".to_owned(),
            contents: public_index(PublicIndexLinkStyle::RelativeMarkdown),
        },
        GeneratedDocument {
            path: "docs/errors/diagnostic-codes.mdx".to_owned(),
            contents: public_index_mdx(),
        },
        GeneratedDocument {
            path: "internal_docs/diagnostic_codes.md".to_owned(),
            contents: internal_reference(),
        },
    ];

    for entry in active_registry_entries() {
        let example = read_example_fragment(repo_root, entry.id);
        documents.push(GeneratedDocument {
            path: entry.docs_path.to_owned(),
            contents: active_code_page(entry, example.as_deref()),
        });
    }

    documents
}

fn read_example_fragment(repo_root: &Path, code: &str) -> Option<String> {
    let path = repo_root.join(format!(
        "crates/sifr_diagnostics/error_page_examples/{code}.md"
    ));
    match fs::read_to_string(&path) {
        Ok(contents) => {
            let trimmed = contents.trim_end().to_owned();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        }
        Err(_) => None,
    }
}

fn write_documents(repo_root: &Path, documents: &[GeneratedDocument]) {
    for document in documents {
        let path = repo_root.join(&document.path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .unwrap_or_else(|err| panic!("failed to create {}: {err}", parent.display()));
        }
        fs::write(&path, document.contents.as_bytes())
            .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
    }
}

fn remove_obsolete_markdown_stubs(repo_root: &Path) {
    let errors_dir = repo_root.join("docs/errors");
    let Ok(entries) = fs::read_dir(&errors_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !(name.starts_with("SIFR-") && name.ends_with(".md")) {
            continue;
        }
        let _ = fs::remove_file(entry.path());
    }
}

fn check_documents(repo_root: &Path, documents: &[GeneratedDocument]) -> io::Result<()> {
    let mut drift = Vec::new();
    for document in documents {
        let path = repo_root.join(&document.path);
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

    let mut expected_names = std::collections::BTreeSet::from([
        "diagnostic-codes.md".to_owned(),
        "diagnostic-codes.mdx".to_owned(),
    ]);
    for entry in active_registry_entries() {
        let expected = format!("{}.mdx", entry.id);
        expected_names.insert(expected.clone());
        if !actual_names.contains(&expected) {
            drift.push(format!(
                "docs/errors is missing exact active-code page casing for {expected}"
            ));
        }
        let obsolete_md = format!("{}.md", entry.id);
        if actual_names.contains(&obsolete_md) {
            drift.push(format!(
                "docs/errors contains obsolete markdown stub {obsolete_md}; run gen-error-docs"
            ));
        }
    }
    for actual_name in actual_names {
        let path = Path::new(&actual_name);
        let is_mdx = path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("mdx"));
        let is_active_mdx = is_mdx
            && actual_name.starts_with("SIFR-")
            && !expected_names.contains(&actual_name);
        if is_active_mdx {
            drift.push(format!(
                "docs/errors contains orphan generated diagnostic page {actual_name}"
            ));
        }
    }
}

#[derive(Copy, Clone)]
enum PublicIndexLinkStyle {
    RelativeMarkdown,
    MintlifyRoute,
}

fn public_index(link_style: PublicIndexLinkStyle) -> String {
    let mut out = match link_style {
        PublicIndexLinkStyle::RelativeMarkdown => generated_header("Diagnostic Codes"),
        PublicIndexLinkStyle::MintlifyRoute => String::new(),
    };
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
                "| [`{}`]({}) | {} | {} |\n",
                entry.id,
                public_index_code_href(entry.id, link_style),
                severity(entry),
                linkify_diagnostic_codes(entry.summary, link_style)
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
            "| [`{}`]({}) | `{}` | {} |\n",
            entry.id,
            diagnostic_code_href(entry.id, link_style),
            entry.family,
            linkify_diagnostic_codes(entry.summary, link_style)
        ));
    }

    out
}

fn public_index_mdx() -> String {
    let mut out = String::from(
        "---\ntitle: \"Diagnostic Codes\"\nsidebarTitle: \"Error Codes\"\ndescription: \"Complete list of active and reserved Sifr diagnostic codes with links to per-code reference pages.\"\n---\n\n",
    );
    out.push_str(&public_index(PublicIndexLinkStyle::MintlifyRoute));
    out
}

fn public_index_code_href(code: &str, link_style: PublicIndexLinkStyle) -> String {
    match link_style {
        PublicIndexLinkStyle::RelativeMarkdown => format!("{code}.mdx"),
        PublicIndexLinkStyle::MintlifyRoute => format!("/errors/{code}"),
    }
}

fn diagnostic_code_href(code: &str, link_style: PublicIndexLinkStyle) -> String {
    let is_active = DIAGNOSTIC_REGISTRY
        .iter()
        .any(|entry| entry.id == code && entry.state == DiagnosticState::Active);
    if is_active {
        public_index_code_href(code, link_style)
    } else {
        match link_style {
            PublicIndexLinkStyle::RelativeMarkdown => "diagnostic-codes.md".to_owned(),
            PublicIndexLinkStyle::MintlifyRoute => "/errors/diagnostic-codes".to_owned(),
        }
    }
}

fn linkify_diagnostic_codes(text: &str, link_style: PublicIndexLinkStyle) -> String {
    let mut codes = DIAGNOSTIC_REGISTRY
        .iter()
        .map(|entry| entry.id)
        .collect::<Vec<_>>();
    codes.sort_by_key(|code| std::cmp::Reverse(code.len()));

    let mut linked = text.to_owned();
    for code in codes {
        let replacement = format!("[`{code}`]({})", diagnostic_code_href(code, link_style));
        linked = replace_unlinked_diagnostic_code(&linked, code, &replacement);
    }
    linked
}

fn replace_unlinked_diagnostic_code(text: &str, code: &str, link: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(index) = rest.find(code) {
        let (before, after_start) = rest.split_at(index);
        let after = &after_start[code.len()..];
        if is_linked_diagnostic_reference(before, after) {
            out.push_str(before);
            out.push_str(code);
            rest = after;
            continue;
        }
        if before.ends_with('`') && after.starts_with('`') {
            out.push_str(&before[..before.len() - 1]);
            out.push_str(link);
            rest = &after[1..];
            continue;
        }
        out.push_str(before);
        out.push_str(link);
        rest = after;
    }
    out.push_str(rest);
    out
}

fn is_linked_diagnostic_reference(before: &str, after: &str) -> bool {
    if before.ends_with("[`") && after.starts_with("`](") {
        return true;
    }
    if let Some(open) = before.rfind("](") {
        return !before[open..].contains(')');
    }
    false
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

fn active_code_page(entry: &DiagnosticRegistryEntry, example: Option<&str>) -> String {
    let docs = family_docs(entry.family);
    let summary = entry.summary.trim();
    let means = lowercase_sentence_start(summary);
    let title = format!("{}: {summary}", entry.id);
    let mut out = String::new();
    out.push_str("---\n");
    out.push_str(&format!("title: \"{}\"\n", escape_yaml(title.as_str())));
    out.push_str(&format!("sidebarTitle: \"{}\"\n", escape_yaml(entry.id)));
    out.push_str(&format!("description: \"{}\"\n", escape_yaml(summary)));
    out.push_str("---\n\n");
    // MDX rejects HTML comments (`<!-- -->`); use a JSX comment instead.
    out.push_str("{/* Generated by cargo run -p sifr_diagnostics --bin gen-error-docs. Do not edit by hand. */}\n\n");
    let means_linked = linkify_diagnostic_codes(&means, PublicIndexLinkStyle::MintlifyRoute);
    let means_clause = means_linked.trim_end_matches('.').to_owned();
    out.push_str(&format!(
        "`{}` belongs to the **{}** diagnostic family. It means: {means_clause}.\n\n",
        entry.id, docs.display_name
    ));
    out.push_str("## Why It Happens\n\n");
    out.push_str(docs.why_it_happens);
    out.push_str("\n\n");
    out.push_str("<Info>\n");
    out.push_str(&format!(
        "Use `sifr --explain {}` locally to see the renderer's exact message template and any machine-applicable suggestions for the compiler version you are running.\n",
        entry.id
    ));
    out.push_str("</Info>\n\n");
    if let Some(example) = example {
        out.push_str(&escape_mdx_outside_code_fences(example));
        out.push_str("\n\n");
    }
    out.push_str("## Details\n\n");
    out.push_str("| Field | Value |\n");
    out.push_str("| --- | --- |\n");
    out.push_str(&format!("| Code | `{}` |\n", entry.id));
    out.push_str(&format!("| Family | `{}` |\n", entry.family));
    out.push_str(&format!("| Severity | {} |\n", severity(entry)));
    out.push_str("| Stability | stable |\n");
    out.push_str(&format!(
        "| Owner | {} |\n",
        optional_code_mdx(entry.owner_module)
    ));
    out.push_str(&format!(
        "| Representative fixture | {} |\n",
        optional_code_mdx(entry.representative_fixture_path)
    ));
    out.push_str(
        "\nSee the [Error Codes index](/diagnostics/error-codes) for the complete catalog.\n",
    );
    out
}

/// Escape MDX-significant characters in prose while leaving fenced code blocks untouched.
fn escape_mdx_outside_code_fences(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_fence = false;
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            out.push_str(line);
            continue;
        }
        if in_fence {
            out.push_str(line);
        } else {
            out.push_str(&escape_mdx_text(line));
        }
    }
    out
}

fn escape_mdx_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('{', "&#123;")
        .replace('}', "&#125;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn optional_code_mdx(value: Option<&str>) -> String {
    value.map_or_else(
        || "n/a".to_owned(),
        |value| format!("`{}`", escape_mdx_text(&escape_table(value))),
    )
}

fn lowercase_sentence_start(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_lowercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn escape_yaml(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
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

fn reference_error_nav_pages() -> Value {
    let mut pages = vec![Value::String("errors/diagnostic-codes".to_owned())];

    for family in DIAGNOSTIC_FAMILIES {
        let family_pages = active_registry_entries()
            .filter(|entry| entry.family == family.name)
            .map(|entry| Value::String(format!("errors/{}", entry.id)))
            .collect::<Vec<_>>();
        if family_pages.is_empty() {
            continue;
        }
        pages.push(Value::Object(Map::from_iter([
            ("group".to_owned(), Value::String(family.name.to_owned())),
            ("pages".to_owned(), Value::Array(family_pages)),
        ])));
    }

    Value::Array(pages)
}

fn sync_docs_json_reference_nav(repo_root: &Path) -> io::Result<()> {
    let docs_json_path = repo_root.join("docs/docs.json");
    let contents = fs::read_to_string(&docs_json_path)?;
    let mut root: Value = serde_json::from_str(&contents).map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("failed to parse {}: {err}", docs_json_path.display()),
        )
    })?;

    let error_codes_group = reference_error_codes_group(&mut root)?;
    error_codes_group.insert("pages".to_owned(), reference_error_nav_pages());

    let updated = serde_json::to_string_pretty(&root).map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("failed to serialize {}: {err}", docs_json_path.display()),
        )
    })?;
    fs::write(&docs_json_path, format!("{updated}\n"))?;
    Ok(())
}

fn check_docs_json_reference_nav(repo_root: &Path) -> io::Result<()> {
    let docs_json_path = repo_root.join("docs/docs.json");
    let contents = fs::read_to_string(&docs_json_path)?;
    let mut root: Value = serde_json::from_str(&contents).map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("failed to parse {}: {err}", docs_json_path.display()),
        )
    })?;

    let error_codes_group = reference_error_codes_group(&mut root)?;
    let actual_pages = error_codes_group.get("pages").ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "Reference > Error Codes group is missing pages",
        )
    })?;
    let expected_pages = reference_error_nav_pages();
    if actual_pages == &expected_pages {
        return Ok(());
    }

    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        format!(
            "{} Reference > Error Codes navigation is out of date; run cargo run -p sifr_diagnostics --bin gen-error-docs",
            docs_json_path.display()
        ),
    ))
}

fn reference_error_codes_group(root: &mut Value) -> io::Result<&mut Map<String, Value>> {
    let tabs = root
        .get_mut("navigation")
        .and_then(Value::as_object_mut)
        .and_then(|navigation| navigation.get_mut("tabs"))
        .and_then(Value::as_array_mut)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "docs.json missing navigation.tabs",
            )
        })?;

    let reference_tab = tabs
        .iter_mut()
        .find(|tab| tab.get("tab").and_then(Value::as_str) == Some("Reference"))
        .ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "docs.json missing Reference tab")
        })?;

    let groups = reference_tab
        .get_mut("groups")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "docs.json missing Reference tab groups",
            )
        })?;

    groups
        .iter_mut()
        .find(|group| group.get("group").and_then(Value::as_str) == Some("Error Codes"))
        .and_then(Value::as_object_mut)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "docs.json missing Reference > Error Codes group",
            )
        })
}
