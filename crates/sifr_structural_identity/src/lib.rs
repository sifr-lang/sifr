//! Single-owner structural shape identity encoding for compiler and runtime.

use sha2::{Digest, Sha256};

/// The current structural identity algorithm. This is independent of Sifr's
/// release number and changes only when the canonical shape contract changes.
pub const ALGORITHM_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ShapeIdentity([u8; 32]);

impl ShapeIdentity {
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NominalField<'a> {
    pub name: &'a str,
    pub identity: ShapeIdentity,
    pub required: bool,
    pub default_identity: Option<ShapeIdentity>,
}

#[must_use]
pub fn primitive(name: &str) -> ShapeIdentity {
    compose("primitive", [name.as_bytes()])
}

#[must_use]
pub fn unary_container(tag: &str, value: ShapeIdentity) -> ShapeIdentity {
    compose(tag, [value.as_bytes().as_slice()])
}

#[must_use]
pub fn binary_container(tag: &str, left: ShapeIdentity, right: ShapeIdentity) -> ShapeIdentity {
    compose(
        tag,
        [left.as_bytes().as_slice(), right.as_bytes().as_slice()],
    )
}

#[must_use]
pub fn tuple(members: &[ShapeIdentity]) -> ShapeIdentity {
    compose_identities("tuple", members)
}

#[must_use]
pub fn union(members: &[ShapeIdentity]) -> ShapeIdentity {
    compose_identities("union", members)
}

#[must_use]
pub fn recursive_reference(index: u32) -> ShapeIdentity {
    compose("recursive-reference", [index.to_be_bytes().as_slice()])
}

#[must_use]
pub fn refined(
    nominal_identity: &str,
    base: ShapeIdentity,
    metadata_identity: ShapeIdentity,
) -> ShapeIdentity {
    compose(
        "refined",
        [
            nominal_identity.as_bytes(),
            base.as_bytes().as_slice(),
            metadata_identity.as_bytes().as_slice(),
        ],
    )
}

#[must_use]
pub fn nominal_record(
    nominal_identity: &str,
    type_arguments: &[ShapeIdentity],
    fields: &[NominalField<'_>],
    metadata_identity: ShapeIdentity,
) -> ShapeIdentity {
    let mut hash = encoder("nominal-record");
    push_bytes(&mut hash, nominal_identity.as_bytes());
    push_usize(&mut hash, type_arguments.len());
    for argument in type_arguments {
        push_bytes(&mut hash, argument.as_bytes());
    }
    push_usize(&mut hash, fields.len());
    for field in fields {
        push_bytes(&mut hash, field.name.as_bytes());
        push_bytes(&mut hash, field.identity.as_bytes());
        push_bytes(&mut hash, &[u8::from(field.required)]);
        match field.default_identity {
            Some(identity) => {
                push_bytes(&mut hash, &[1]);
                push_bytes(&mut hash, identity.as_bytes());
            }
            None => push_bytes(&mut hash, &[0]),
        }
    }
    push_bytes(&mut hash, metadata_identity.as_bytes());
    finish(hash)
}

#[must_use]
pub fn enum_shape(
    nominal_identity: &str,
    members: &[(&str, Option<i64>)],
    metadata_identity: ShapeIdentity,
) -> ShapeIdentity {
    let mut hash = encoder("enum");
    push_bytes(&mut hash, nominal_identity.as_bytes());
    push_usize(&mut hash, members.len());
    for (name, discriminant) in members {
        push_bytes(&mut hash, name.as_bytes());
        match discriminant {
            Some(value) => {
                push_bytes(&mut hash, &[1]);
                push_bytes(&mut hash, &value.to_be_bytes());
            }
            None => push_bytes(&mut hash, &[0]),
        }
    }
    push_bytes(&mut hash, metadata_identity.as_bytes());
    finish(hash)
}

#[must_use]
pub fn metadata(canonical_entries: &[&str]) -> ShapeIdentity {
    compose(
        "metadata",
        canonical_entries.iter().map(|entry| entry.as_bytes()),
    )
}

/// Identity of a compiler-canonicalized field default value.
#[must_use]
pub fn default_value(canonical_value: &str) -> ShapeIdentity {
    compose("default-value", [canonical_value.as_bytes()])
}

fn compose<'a>(tag: &str, parts: impl IntoIterator<Item = &'a [u8]>) -> ShapeIdentity {
    let mut hash = encoder(tag);
    for part in parts {
        push_bytes(&mut hash, part);
    }
    finish(hash)
}

fn compose_identities(tag: &str, identities: &[ShapeIdentity]) -> ShapeIdentity {
    let mut hash = encoder(tag);
    push_usize(&mut hash, identities.len());
    for identity in identities {
        push_bytes(&mut hash, identity.as_bytes());
    }
    finish(hash)
}

fn encoder(tag: &str) -> Sha256 {
    let mut hash = Sha256::new();
    hash.update(b"sifr-structural-identity");
    hash.update(ALGORITHM_VERSION.to_be_bytes());
    push_bytes(&mut hash, tag.as_bytes());
    hash
}

fn push_usize(hash: &mut Sha256, value: usize) {
    hash.update(u64::try_from(value).unwrap_or(u64::MAX).to_be_bytes());
}

fn push_bytes(hash: &mut Sha256, value: &[u8]) {
    push_usize(hash, value.len());
    hash.update(value);
}

fn finish(hash: Sha256) -> ShapeIdentity {
    ShapeIdentity::from_bytes(hash.finalize().into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identities_are_ordered_length_delimited_and_deterministic() {
        let string = primitive("str");
        assert_eq!(string, primitive("str"));
        assert_ne!(string, primitive("bytes"));
        assert_ne!(
            tuple(&[primitive("a"), primitive("bc")]),
            tuple(&[primitive("ab"), primitive("c")])
        );
        assert_ne!(
            union(&[primitive("a"), primitive("b")]),
            union(&[primitive("b"), primitive("a")])
        );
    }

    #[test]
    fn nominal_identity_tracks_required_and_default_state() {
        let field = NominalField {
            name: "value",
            identity: primitive("str"),
            required: true,
            default_identity: None,
        };
        let optional = NominalField {
            required: false,
            ..field
        };
        assert_ne!(
            nominal_record("pkg.Item", &[], &[field], metadata(&[])),
            nominal_record("pkg.Item", &[], &[optional], metadata(&[]))
        );
    }

    #[test]
    fn default_value_identity_is_typed_and_deterministic() {
        assert_eq!(default_value("int:1"), default_value("int:1"));
        assert_ne!(default_value("int:1"), default_value("str:1"));
    }
}
