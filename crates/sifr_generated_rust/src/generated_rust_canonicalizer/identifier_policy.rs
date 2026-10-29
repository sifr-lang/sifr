use std::collections::{BTreeMap, BTreeSet};

const VALUE_PREFIX: &str = "sifr_generated_";
const TYPE_PREFIX: &str = "SifrGenerated";
const STATIC_PREFIX: &str = "SIFR_GENERATED_";

pub(super) fn canonical_name_map(identifiers: &BTreeSet<String>) -> BTreeMap<String, String> {
    let candidates = identifiers
        .iter()
        .filter_map(|identifier| {
            canonical_identifier_candidate(identifier)
                .map(|candidate| (identifier.clone(), candidate))
        })
        .collect::<BTreeMap<_, _>>();
    let reserved = candidates.values().cloned().collect::<BTreeSet<_>>();
    let mut names = BTreeMap::new();
    let mut assigned = BTreeSet::new();

    for identifier in identifiers
        .iter()
        .filter(|identifier| !candidates.contains_key(*identifier))
    {
        if reserved.contains(identifier) {
            let escaped = format!("{identifier}_user_{}", hex_identifier(identifier));
            assigned.insert(escaped.clone());
            names.insert(identifier.clone(), escaped);
        } else {
            assigned.insert(identifier.clone());
        }
    }

    for (identifier, mut candidate) in candidates {
        if assigned.contains(&candidate) {
            let base = candidate;
            candidate = format!("{base}_{}", hex_identifier(&identifier));
            while assigned.contains(&candidate) {
                candidate.push_str("_generated");
            }
        }
        assigned.insert(candidate.clone());
        names.insert(identifier, candidate);
    }
    names
}

pub(super) fn canonical_identifier_candidate(identifier: &str) -> Option<String> {
    if identifier == "_" {
        return None;
    }
    let identifier = identifier.strip_prefix("r#").unwrap_or(identifier);
    if !identifier.starts_with('_') {
        return None;
    }

    let significant = identifier.trim_start_matches('_');
    if significant.is_empty() {
        return Some(format!("{VALUE_PREFIX}underscore"));
    }
    if significant.chars().all(|character| {
        character == '_' || character.is_ascii_uppercase() || character.is_ascii_digit()
    }) {
        return Some(format!("{STATIC_PREFIX}{significant}"));
    }
    let type_like = significant.chars().next().is_some_and(char::is_uppercase);
    let base = if type_like {
        significant.strip_prefix("Sifr").unwrap_or(significant)
    } else {
        significant.strip_prefix("sifr_").unwrap_or(significant)
    };
    Some(if type_like {
        format!("{TYPE_PREFIX}{}", upper_camel_identifier(base))
    } else {
        format!("{VALUE_PREFIX}{base}")
    })
}

fn upper_camel_identifier(identifier: &str) -> String {
    let mut canonical = String::with_capacity(identifier.len());
    let mut capitalize = true;
    for character in identifier.chars() {
        if character == '_' {
            capitalize = true;
        } else if capitalize {
            canonical.extend(character.to_uppercase());
            capitalize = false;
        } else {
            canonical.push(character);
        }
    }
    canonical
}

fn hex_identifier(identifier: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(identifier.len() * 2);
    for byte in identifier.bytes() {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}
