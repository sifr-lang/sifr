use crate::errors::{LspError, LspResult, required_string};
use serde_json::Value;

pub(crate) enum DocumentContentChange {
    Full { text: String },
    Incremental { range: Value, text: String },
}

pub(crate) struct CompactedDocumentChange {
    pub(crate) changes: Vec<DocumentContentChange>,
    pub(crate) raw_change_count: usize,
}

pub(crate) fn compact_content_changes(changes: &[Value]) -> LspResult<CompactedDocumentChange> {
    if changes.is_empty() {
        return Err(LspError::invalid_params(
            "textDocument/didChange requires at least one content change",
        ));
    }
    let mut compacted = Vec::new();
    for change in changes {
        let text = required_string(change, "/text")?;
        if let Some(range) = change.get("range") {
            compacted.push(DocumentContentChange::Incremental {
                range: range.clone(),
                text,
            });
        } else {
            compacted.clear();
            compacted.push(DocumentContentChange::Full { text });
        }
    }
    Ok(CompactedDocumentChange {
        changes: compacted,
        raw_change_count: changes.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::{DocumentContentChange, compact_content_changes};
    use serde_json::json;

    #[test]
    fn compact_content_changes_keeps_only_latest_full_prefix() {
        let compacted = compact_content_changes(&[
            json!({"text": "old"}),
            json!({"text": "new"}),
            json!({
                "range": {
                    "start": {"line": 0, "character": 0},
                    "end": {"line": 0, "character": 0}
                },
                "text": "# "
            }),
        ])
        .expect("changes compact");

        assert_eq!(compacted.raw_change_count, 3);
        assert_eq!(compacted.changes.len(), 2);
        assert!(matches!(
            compacted.changes.first(),
            Some(DocumentContentChange::Full { text }) if text == "new"
        ));
        assert!(matches!(
            compacted.changes.get(1),
            Some(DocumentContentChange::Incremental { text, .. }) if text == "# "
        ));
    }
}
