use crate::SourcePath;
use sha2::{Digest as _, Sha256};

pub(crate) struct FingerprintBuilder {
    hash: Sha256,
}

impl FingerprintBuilder {
    pub(crate) fn new(domain: &str) -> Self {
        let mut builder = Self {
            hash: Sha256::new(),
        };
        builder.field("domain", domain);
        builder
    }

    pub(crate) fn field(&mut self, name: &str, value: impl AsRef<str>) {
        let value = value.as_ref();
        self.write(name.as_bytes());
        self.write(&[0]);
        self.write(value.len().to_string().as_bytes());
        self.write(&[0]);
        self.write(value.as_bytes());
        self.write(&[0xff]);
    }

    pub(crate) fn path_field(&mut self, name: &str, path: &SourcePath) {
        self.field(name, path.as_path().display().to_string());
    }

    pub(crate) fn optional_path_field(&mut self, name: &str, path: Option<&SourcePath>) {
        if let Some(path) = path {
            self.path_field(name, path);
        } else {
            self.field(name, "<none>");
        }
    }

    pub(crate) fn finish_hex(self) -> String {
        const DIGITS: &[u8; 16] = b"0123456789abcdef";
        let bytes = self.hash.finalize();
        let mut encoded = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            encoded.push(char::from(DIGITS[usize::from(byte >> 4)]));
            encoded.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
        }
        encoded
    }

    fn write(&mut self, bytes: &[u8]) {
        self.hash.update(bytes);
    }
}
