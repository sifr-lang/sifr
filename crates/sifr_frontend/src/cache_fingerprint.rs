use crate::SourcePath;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CacheKeyFingerprint(pub(crate) String);

impl CacheKeyFingerprint {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub(crate) fn stable_cache_fingerprint(
    domain: &str,
    fields: impl IntoIterator<Item = (&'static str, String)>,
) -> CacheKeyFingerprint {
    let mut builder = FingerprintBuilder::new(domain);
    for (name, value) in fields {
        builder.field(name, value);
    }
    CacheKeyFingerprint(builder.finish_hex())
}

pub(crate) struct FingerprintBuilder {
    hash: u64,
}

impl FingerprintBuilder {
    pub(crate) fn new(domain: &str) -> Self {
        let mut builder = Self {
            hash: 0xcbf2_9ce4_8422_2325_u64,
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
        format!("{:016x}", self.hash)
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.hash ^= u64::from(*byte);
            self.hash = self.hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
}
