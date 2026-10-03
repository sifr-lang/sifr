//! Lossless original expanded text storage between compiler callbacks.
//! The actual owner/span/site metadata stays in the caller-held declarations.
//! Every record binds that metadata and all text bytes before any selection.
use crate::expanded::Declaration;
use rustc_span::{SourceFileHash, SourceFileHashAlgorithm};
use serde_json::json;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;

struct Record {
    start: u64,
    length: u64,
    hash: SourceFileHash,
}
pub struct OriginalText {
    file: File,
    path: PathBuf,
    records: Vec<Record>,
    sealed_length: Option<u64>,
}
fn identity(decl: &Declaration) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "node_id":decl.node_id.as_u32(),"local_def_id":decl.def.local_def_index.as_u32(),
        "span":format!("{:?}",decl.span),"ast_kind":decl.ast_kind,
        "tokens_available":decl.tokens_available,"body":decl.body,
        "impl_member_count":decl.impl_member_count,"body_tokens_present":decl.body_tokens.is_some(),
        "sites":decl.sites.iter().map(|site|json!({"kind":site.kind,
            "span":format!("{:?}",site.span),"ancestor":site.ancestor})).collect::<Vec<_>>()
    }))
    .expect("serialize actual expanded owner/span/site identity")
}
fn write_bytes(file: &mut File, bytes: &[u8]) {
    file.write_all(&(bytes.len() as u64).to_le_bytes())
        .expect("write original length");
    file.write_all(bytes)
        .expect("write complete original bytes");
}
fn read_bytes(file: &mut File, remaining: &mut u64) -> Vec<u8> {
    assert!(*remaining >= 8, "original text length is truncated");
    let mut length = [0; 8];
    file.read_exact(&mut length).expect("read original length");
    *remaining -= 8;
    let length = u64::from_le_bytes(length);
    assert!(
        length <= *remaining,
        "original text exceeds its authenticated record"
    );
    let mut bytes = vec![0; usize::try_from(length).expect("original text length fits host")];
    file.read_exact(&mut bytes)
        .expect("read complete original bytes");
    *remaining -= length;
    bytes
}
impl OriginalText {
    pub fn create() -> Self {
        let destination = [
            "SIFR_BUILTIN_SOURCE_ENVELOPE",
            "SIFR_BUILTIN_CAPTURE",
            "SIFR_BUILTIN_SOURCE_BINDER",
        ]
        .iter()
        .find_map(std::env::var_os)
        .expect("explicit original capture destination");
        let mut path = PathBuf::from(destination);
        let name = path
            .file_name()
            .expect("capture filename")
            .to_string_lossy();
        path.set_file_name(format!(
            "{name}.expanded-original-{}.tmp",
            std::process::id()
        ));
        Self::open(path)
    }
    fn open(path: PathBuf) -> Self {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)
            .expect("create exclusively owned original text storage");
        Self {
            file,
            path,
            records: vec![],
            sealed_length: None,
        }
    }
    pub fn store(&mut self, decl: &mut Declaration) {
        assert!(self.sealed_length.is_none(), "original text is sealed");
        let start = self.file.stream_position().expect("original record start");
        write_bytes(&mut self.file, &identity(decl));
        write_bytes(&mut self.file, decl.tokens.as_bytes());
        if let Some(body) = &decl.body_tokens {
            write_bytes(&mut self.file, body.as_bytes());
        }
        for site in &decl.sites {
            write_bytes(&mut self.file, site.tokens.as_bytes());
        }
        let end = self.file.stream_position().expect("original record end");
        self.file
            .seek(SeekFrom::Start(start))
            .expect("hash original record");
        let hash = SourceFileHash::new(
            SourceFileHashAlgorithm::Sha256,
            (&mut self.file).take(end - start),
        )
        .expect("hash all original bytes");
        self.file
            .seek(SeekFrom::Start(end))
            .expect("resume original serialization");
        self.records.push(Record {
            start,
            length: end - start,
            hash,
        });
        // Drop capacity as well as content while the compiler performs analysis.
        decl.tokens = String::new();
        if decl.body_tokens.is_some() {
            decl.body_tokens = Some(String::new());
        }
        for site in &mut decl.sites {
            site.tokens = String::new();
        }
    }
    pub fn seal(&mut self) {
        self.file
            .sync_all()
            .expect("durable complete original serialization");
        self.sealed_length = Some(self.file.metadata().expect("original metadata").len());
        let mut final_path = self.path.clone();
        final_path.set_extension("bin");
        assert!(
            !final_path.exists(),
            "original capture destination already exists"
        );
        std::fs::rename(&self.path, &final_path)
            .expect("publish complete original text serialization");
        self.file = File::open(&final_path).expect("read-only original text storage");
        self.path = final_path;
    }
    pub fn restore(mut self, declarations: &mut [Declaration]) {
        assert_eq!(
            self.records.len(),
            declarations.len(),
            "complete original owner inventory"
        );
        assert_eq!(
            self.sealed_length,
            Some(self.file.metadata().expect("original metadata").len()),
            "complete original byte length"
        );
        for (record, decl) in self.records.iter().zip(declarations) {
            self.file
                .seek(SeekFrom::Start(record.start))
                .expect("authenticate original record");
            let hash = SourceFileHash::new(
                SourceFileHashAlgorithm::Sha256,
                (&mut self.file).take(record.length),
            )
            .expect("authenticate complete original bytes");
            assert_eq!(hash, record.hash, "original owner/text record changed");
            self.file
                .seek(SeekFrom::Start(record.start))
                .expect("read authenticated original record");
            let mut remaining = record.length;
            assert_eq!(
                read_bytes(&mut self.file, &mut remaining),
                identity(decl),
                "actual original owner/span/site identity changed"
            );
            decl.tokens = String::from_utf8(read_bytes(&mut self.file, &mut remaining))
                .expect("original UTF-8");
            if decl.body_tokens.is_some() {
                decl.body_tokens = Some(
                    String::from_utf8(read_bytes(&mut self.file, &mut remaining))
                        .expect("original body UTF-8"),
                );
            }
            for site in &mut decl.sites {
                site.tokens = String::from_utf8(read_bytes(&mut self.file, &mut remaining))
                    .expect("original site UTF-8");
            }
            assert_eq!(remaining, 0, "all original bytes restored exhaustively");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expanded::Site;
    use rustc_ast::NodeId;
    use rustc_hir::def_id::{LocalDefId, LocalDefIndex};
    use rustc_span::DUMMY_SP;
    fn original(label: &str) -> (OriginalText, Declaration) {
        let path = std::env::temp_dir().join(format!(
            "sifr-expanded-original-{}-{label}.tmp",
            std::process::id()
        ));
        let declaration = Declaration {
            def: LocalDefId {
                local_def_index: LocalDefIndex::from_u32(7),
            },
            node_id: NodeId::from_u32(19),
            ast_kind: "associated function".into(),
            attrs: Default::default(),
            tokens_available: false,
            span: DUMMY_SP,
            tokens: "fn café() {\r\n println!(\"雪\"); }".into(),
            body: true,
            impl_member_count: None,
            body_tokens: Some("{\n café();\n}".into()),
            sites: vec![
                Site {
                    kind: "call".into(),
                    tokens: "café()".into(),
                    span: DUMMY_SP,
                    ancestor: None,
                },
                Site {
                    kind: "unary:Not".into(),
                    tokens: "!false".into(),
                    span: DUMMY_SP,
                    ancestor: Some(0),
                },
            ],
        };
        (OriginalText::open(path), declaration)
    }
    #[test]
    fn complete_original_text_and_identity_round_trip() {
        let (mut storage, mut decl) = original("round-trip");
        let expected = (
            identity(&decl),
            decl.tokens.clone(),
            decl.body_tokens.clone(),
            decl.sites
                .iter()
                .map(|site| site.tokens.clone())
                .collect::<Vec<_>>(),
        );
        storage.store(&mut decl);
        assert!(decl.tokens.is_empty() && decl.sites.iter().all(|site| site.tokens.is_empty()));
        assert_eq!(identity(&decl), expected.0);
        storage.seal();
        storage.restore(std::slice::from_mut(&mut decl));
        assert_eq!(identity(&decl), expected.0);
        assert_eq!(decl.tokens, expected.1);
        assert_eq!(decl.body_tokens, expected.2);
        assert_eq!(
            decl.sites
                .iter()
                .map(|site| site.tokens.clone())
                .collect::<Vec<_>>(),
            expected.3
        );
    }
    #[test]
    #[should_panic(expected = "original owner/text record changed")]
    fn changed_original_bytes_reject_before_restore() {
        let (mut storage, mut decl) = original("corrupt");
        storage.store(&mut decl);
        storage.seal();
        let mut writer = OpenOptions::new()
            .write(true)
            .open(&storage.path)
            .expect("test owned file");
        writer
            .seek(SeekFrom::Start(storage.records[0].length - 1))
            .expect("test byte");
        writer.write_all(&[0]).expect("mutate actual original byte");
        writer.sync_all().expect("durable mutation");
        storage.restore(std::slice::from_mut(&mut decl));
    }
    #[test]
    #[should_panic(expected = "actual original owner/span/site identity changed")]
    fn changed_site_ancestry_rejects_authenticated_text() {
        let (mut storage, mut decl) = original("ancestry");
        storage.store(&mut decl);
        storage.seal();
        decl.sites[1].ancestor = None;
        storage.restore(std::slice::from_mut(&mut decl));
    }
}
