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
    hash: sifr_identity::IdentityEncoder,
}

impl FingerprintBuilder {
    pub(crate) fn new(domain: &str) -> Self {
        let mut builder = Self {
            hash: sifr_identity::IdentityEncoder::new("frontend-cache-v2"),
        };
        builder.field("domain", domain);
        builder
    }

    pub(crate) fn field(&mut self, name: &str, value: impl AsRef<str>) {
        self.hash.field(name, value.as_ref().as_bytes());
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
        self.hash.finish()
    }
}
