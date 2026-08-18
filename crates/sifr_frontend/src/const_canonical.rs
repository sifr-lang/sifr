//! Canonical identities for closed compiler-owned const values.

use crate::ConstValue;

pub(crate) fn canonical_value(value: &ConstValue) -> String {
    match value {
        ConstValue::None => "none".to_string(),
        ConstValue::Bool(value) => format!("bool:{value}"),
        ConstValue::Integer(value) => format!("int:{value}"),
        ConstValue::FloatBits(value) => format!("float:{value:016x}"),
        ConstValue::String(value) => format!("str:{}:{value}", value.len()),
        ConstValue::Bytes(value) => format!("bytes:{}:{}", value.len(), canonical_bytes(value)),
        ConstValue::Tuple(values) => canonical_values("tuple", values),
        ConstValue::List(values) => canonical_values("list", values),
        ConstValue::Record(values) => format!(
            "record[{}]",
            values
                .iter()
                .map(|(key, value)| { format!("{}:{key}={}", key.len(), canonical_value(value)) })
                .collect::<Vec<_>>()
                .join(",")
        ),
        ConstValue::CallableIdentity(identity) => format!(
            "callable[module={}:{};owner={}:{};symbol={}:{};generics=[{}];signature={}:{}]",
            identity.module.len(),
            identity.module,
            identity.owner.as_deref().unwrap_or("").len(),
            identity.owner.as_deref().unwrap_or(""),
            identity.symbol.len(),
            identity.symbol,
            identity
                .generic_arguments
                .iter()
                .map(|argument| format!("{}:{argument}", argument.len()))
                .collect::<Vec<_>>()
                .join(","),
            identity.signature.len(),
            identity.signature,
        ),
        // Origin identity is diagnostic-only. A package result containing an
        // origin is rejected before static-program canonicalization.
        ConstValue::SourceOrigin(_) => "source-origin:opaque".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sifr_lowering::{CallableIdentity, StaticProgramValue};

    fn callable(symbol: &str, generic: &str) -> ConstValue {
        ConstValue::CallableIdentity(CallableIdentity {
            module: "fixture.callbacks".to_string(),
            owner: Some("fixture.callbacks.Handler".to_string()),
            symbol: symbol.to_string(),
            generic_arguments: vec![generic.to_string()],
            signature: "FunctionType([str], bool)".to_string(),
        })
    }

    #[test]
    fn callable_identity_round_trips_and_changes_canonical_target_identity() {
        let first = callable("accept", "str");
        let changed_symbol = callable("reject", "str");
        let changed_generic = callable("accept", "bytes");
        assert_ne!(canonical_value(&first), canonical_value(&changed_symbol));
        assert_ne!(canonical_value(&first), canonical_value(&changed_generic));

        let static_value = crate::specialization_support::static_program_value(&first)
            .expect("callable identity should be a retained static value");
        let StaticProgramValue::CallableIdentity(identity) = static_value else {
            panic!("callable identity should round-trip without scalar erasure");
        };
        assert_eq!(identity.symbol, "accept");
        assert_eq!(identity.generic_arguments, vec!["str"]);
    }
}

fn canonical_bytes(value: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(value.len() * 2);
    for byte in value {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

fn canonical_values(kind: &str, values: &[ConstValue]) -> String {
    format!(
        "{kind}[{}]",
        values
            .iter()
            .map(canonical_value)
            .collect::<Vec<_>>()
            .join(",")
    )
}
