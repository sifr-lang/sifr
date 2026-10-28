//! Stateless compatibility identities. No compiler-wide constants live here.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]
use sha2::{Digest, Sha256};

mod stream;
#[cfg(test)]
#[path = "../tests/unit/stream_tests.rs"]
mod stream_tests;

#[cfg(any(feature = "build", test))]
pub mod build;

#[derive(Clone)]
pub struct IdentityEncoder(Sha256);
impl IdentityEncoder {
    pub fn new(domain: &str) -> Self {
        let mut result = Self(Sha256::new());
        result.field("domain", domain.as_bytes());
        result
    }
    pub fn field(&mut self, name: &str, bytes: &[u8]) {
        for value in [name.as_bytes(), bytes] {
            self.0.update((value.len() as u64).to_le_bytes());
            self.0.update(value);
        }
    }
    pub fn finish(self) -> String {
        self.0
            .finalize()
            .iter()
            .fold(String::with_capacity(64), |mut result, byte| {
                use std::fmt::Write;
                let _ = write!(result, "{byte:02x}");
                result
            })
    }
}

macro_rules! identity {
    ($($name:ident => $domain:literal),+ $(,)?) => {$(
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);
        impl $name {
            pub fn from_records<'a>(records: impl IntoIterator<Item = (&'a str, &'a [u8])>) -> Self {
                let mut encoder = IdentityEncoder::new($domain);
                for (key, value) in records { encoder.field(key, value); }
                Self(encoder.finish())
            }
            pub fn as_str(&self) -> &str { &self.0 }
        }
    )+};
}
identity! {
    CompilerBuildId => "compiler-build-v1",
    TargetSemanticId => "target-semantic-v1",
    StdlibInputsId => "stdlib-inputs-v1",
    MetadataId => "metadata-bytes-v1",
    NativeBuildId => "native-build-v1",
    ValidationInputsId => "validation-inputs-v1",
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CompilerIdentity {
    id: String,
    test: bool,
}
impl CompilerIdentity {
    pub fn product(embedded: &str) -> Result<Self, &'static str> {
        if embedded.len() != 64 || !embedded.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("invalid embedded compiler identity");
        }
        Ok(Self {
            id: embedded.to_owned(),
            test: false,
        })
    }
    pub fn for_test(mut tokens: Vec<(&str, &str)>, configuration: &str) -> Self {
        tokens.sort_unstable();
        tokens.dedup();
        let mut encoder = IdentityEncoder::new("test-compiler-v1");
        for (name, token) in tokens {
            encoder.field(name, token.as_bytes());
        }
        encoder.field("configuration", configuration.as_bytes());
        Self {
            id: encoder.finish(),
            test: true,
        }
    }
    pub fn as_str(&self) -> &str {
        &self.id
    }
    pub fn is_test(&self) -> bool {
        self.test
    }
    pub fn validate_override(&self, claimed: &Self) -> Result<(), &'static str> {
        if self == claimed {
            Ok(())
        } else {
            Err("compiler identity mismatch; rebuild the supplied artifact for this context")
        }
    }
}

#[cfg(test)]
#[path = "../tests/unit/lib_tests.rs"]
mod tests;
