const FNV_128_OFFSET: u128 = 0x6c62_272e_07bb_0142_62b8_2175_6295_c58d;
const FNV_128_PRIME: u128 = 0x0000_0000_0100_0000_0000_0000_0000_013b;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IpcSchemaDescriptor {
    pub protocol_schema_version: u32,
    pub module_path: String,
    pub schema_name: String,
    pub compatible_version_min: u32,
    pub compatible_version_max: u32,
    pub request: IpcSchemaType,
    pub response: IpcSchemaType,
    pub error: IpcSchemaType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IpcSchemaField {
    pub name: String,
    pub ty: IpcSchemaType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IpcSchemaVariant {
    pub name: String,
    pub payload: Option<IpcSchemaType>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IpcSchemaType {
    Bool,
    Int,
    Float,
    Str,
    Bytes,
    None,
    Option(Box<IpcSchemaType>),
    Result(Box<IpcSchemaType>, Box<IpcSchemaType>),
    List(Box<IpcSchemaType>),
    DictStr(Box<IpcSchemaType>),
    Tuple(Vec<IpcSchemaType>),
    Record {
        name: String,
        fields: Vec<IpcSchemaField>,
    },
    Enum {
        name: String,
        variants: Vec<IpcSchemaVariant>,
    },
    Unsupported {
        type_name: String,
    },
}

#[must_use]
pub fn canonical_schema_descriptor(descriptor: &IpcSchemaDescriptor) -> String {
    let mut output = String::new();
    output.push_str("sifr-ipc-schema-v1\nprotocol_schema_version=");
    output.push_str(&descriptor.protocol_schema_version.to_string());
    output.push_str("\nmodule=");
    push_escaped(&mut output, &descriptor.module_path);
    output.push_str("\nschema=");
    push_escaped(&mut output, &descriptor.schema_name);
    output.push_str("\ncompatible=");
    output.push_str(&descriptor.compatible_version_min.to_string());
    output.push_str("..");
    output.push_str(&descriptor.compatible_version_max.to_string());
    output.push_str("\nrequest=");
    push_type(&mut output, &descriptor.request);
    output.push_str("\nresponse=");
    push_type(&mut output, &descriptor.response);
    output.push_str("\nerror=");
    push_type(&mut output, &descriptor.error);
    output
}

#[must_use]
pub fn schema_hash_v1(descriptor: &IpcSchemaDescriptor) -> u128 {
    fnv1a_128(canonical_schema_descriptor(descriptor).as_bytes())
}

#[must_use]
pub fn schema_hash_hex_v1(descriptor: &IpcSchemaDescriptor) -> String {
    format!("{:032x}", schema_hash_v1(descriptor))
}

#[must_use]
pub fn fnv1a_128(bytes: &[u8]) -> u128 {
    let mut hash = FNV_128_OFFSET;
    for byte in bytes {
        hash ^= u128::from(*byte);
        hash = hash.wrapping_mul(FNV_128_PRIME);
    }
    hash
}

fn push_type(output: &mut String, ty: &IpcSchemaType) {
    match ty {
        IpcSchemaType::Bool => output.push_str("bool"),
        IpcSchemaType::Int => output.push_str("int"),
        IpcSchemaType::Float => output.push_str("float"),
        IpcSchemaType::Str => output.push_str("str"),
        IpcSchemaType::Bytes => output.push_str("bytes"),
        IpcSchemaType::None => output.push_str("none"),
        IpcSchemaType::Option(inner) => {
            output.push_str("option(");
            push_type(output, inner);
            output.push(')');
        }
        IpcSchemaType::Result(ok, err) => {
            output.push_str("result(");
            push_type(output, ok);
            output.push(',');
            push_type(output, err);
            output.push(')');
        }
        IpcSchemaType::List(inner) => {
            output.push_str("list(");
            push_type(output, inner);
            output.push(')');
        }
        IpcSchemaType::DictStr(value) => {
            output.push_str("dict(str,");
            push_type(output, value);
            output.push(')');
        }
        IpcSchemaType::Tuple(items) => {
            output.push_str("tuple(");
            push_types(output, items);
            output.push(')');
        }
        IpcSchemaType::Record { name, fields } => {
            output.push_str("record(");
            push_escaped(output, name);
            output.push('{');
            for (index, field) in fields.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                push_escaped(output, &field.name);
                output.push(':');
                push_type(output, &field.ty);
            }
            output.push_str("})");
        }
        IpcSchemaType::Enum { name, variants } => {
            output.push_str("enum(");
            push_escaped(output, name);
            output.push('{');
            for (index, variant) in variants.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                push_escaped(output, &variant.name);
                if let Some(payload) = &variant.payload {
                    output.push('(');
                    push_type(output, payload);
                    output.push(')');
                }
            }
            output.push_str("})");
        }
        IpcSchemaType::Unsupported { type_name } => {
            output.push_str("unsupported(");
            push_escaped(output, type_name);
            output.push(')');
        }
    }
}

fn push_types(output: &mut String, items: &[IpcSchemaType]) {
    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        push_type(output, item);
    }
}

fn push_escaped(output: &mut String, value: &str) {
    for ch in value.chars() {
        match ch {
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '(' | ')' | '{' | '}' | ':' | ',' | '=' => {
                output.push('\\');
                output.push(ch);
            }
            _ => output.push(ch),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        IpcSchemaDescriptor, IpcSchemaField, IpcSchemaType, IpcSchemaVariant,
        canonical_schema_descriptor, schema_hash_hex_v1,
    };

    fn sample_descriptor() -> IpcSchemaDescriptor {
        IpcSchemaDescriptor {
            protocol_schema_version: 1,
            module_path: "demo.worker".to_string(),
            schema_name: "Echo".to_string(),
            compatible_version_min: 1,
            compatible_version_max: 1,
            request: IpcSchemaType::Record {
                name: "EchoRequest".to_string(),
                fields: vec![
                    IpcSchemaField {
                        name: "message".to_string(),
                        ty: IpcSchemaType::Str,
                    },
                    IpcSchemaField {
                        name: "attempt".to_string(),
                        ty: IpcSchemaType::Int,
                    },
                ],
            },
            response: IpcSchemaType::Record {
                name: "EchoResponse".to_string(),
                fields: vec![IpcSchemaField {
                    name: "accepted".to_string(),
                    ty: IpcSchemaType::Bool,
                }],
            },
            error: IpcSchemaType::Enum {
                name: "EchoError".to_string(),
                variants: vec![
                    IpcSchemaVariant {
                        name: "Rejected".to_string(),
                        payload: Some(IpcSchemaType::Str),
                    },
                    IpcSchemaVariant {
                        name: "Closed".to_string(),
                        payload: None,
                    },
                ],
            },
        }
    }

    #[test]
    fn canonical_descriptor_is_stable_and_ordered() {
        let descriptor = sample_descriptor();

        assert_eq!(
            canonical_schema_descriptor(&descriptor),
            "sifr-ipc-schema-v1\nprotocol_schema_version=1\nmodule=demo.worker\nschema=Echo\ncompatible=1..1\nrequest=record(EchoRequest{message:str,attempt:int})\nresponse=record(EchoResponse{accepted:bool})\nerror=enum(EchoError{Rejected(str),Closed})"
        );
    }

    #[test]
    fn schema_hash_v1_is_stable_and_sensitive_to_shape() {
        let descriptor = sample_descriptor();
        let mut changed = sample_descriptor();
        let IpcSchemaType::Record { fields, .. } = &mut changed.request else {
            panic!("sample request should be a record");
        };
        fields[1].ty = IpcSchemaType::Float;

        assert_eq!(
            schema_hash_hex_v1(&descriptor),
            "4733c89fb23a40ecb5f3bcda99fb34da"
        );
        assert_ne!(
            schema_hash_hex_v1(&descriptor),
            schema_hash_hex_v1(&changed)
        );
    }
}
