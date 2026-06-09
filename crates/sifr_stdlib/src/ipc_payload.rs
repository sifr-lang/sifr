use crate::IpcSchemaType;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IpcPayloadEligibilityError {
    UnsupportedPayload { type_name: String },
}

impl Display for IpcPayloadEligibilityError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedPayload { type_name } => {
                write!(formatter, "unsupported IPC payload type {type_name}")
            }
        }
    }
}

impl std::error::Error for IpcPayloadEligibilityError {}

pub fn validate_ipc_payload_type(ty: &IpcSchemaType) -> Result<(), IpcPayloadEligibilityError> {
    match ty {
        IpcSchemaType::Bool
        | IpcSchemaType::Int
        | IpcSchemaType::Float
        | IpcSchemaType::Str
        | IpcSchemaType::Bytes
        | IpcSchemaType::None => Ok(()),
        IpcSchemaType::Option(inner)
        | IpcSchemaType::List(inner)
        | IpcSchemaType::DictStr(inner) => validate_ipc_payload_type(inner),
        IpcSchemaType::Result(ok, err) => {
            validate_ipc_payload_type(ok)?;
            validate_ipc_payload_type(err)
        }
        IpcSchemaType::Tuple(items) => validate_all(items),
        IpcSchemaType::Record { fields, .. } => {
            for field in fields {
                validate_ipc_payload_type(&field.ty)?;
            }
            Ok(())
        }
        IpcSchemaType::Enum { variants, .. } => {
            for variant in variants {
                if let Some(payload) = &variant.payload {
                    validate_ipc_payload_type(payload)?;
                }
            }
            Ok(())
        }
        IpcSchemaType::Unsupported { type_name } => {
            Err(IpcPayloadEligibilityError::UnsupportedPayload {
                type_name: type_name.clone(),
            })
        }
    }
}

fn validate_all(items: &[IpcSchemaType]) -> Result<(), IpcPayloadEligibilityError> {
    for item in items {
        validate_ipc_payload_type(item)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{validate_ipc_payload_type, IpcPayloadEligibilityError};
    use crate::{IpcSchemaField, IpcSchemaType, IpcSchemaVariant};

    #[test]
    fn accepts_initial_ipc_serializable_payload_families() {
        let payload = IpcSchemaType::Record {
            name: "Request".to_string(),
            fields: vec![
                IpcSchemaField {
                    name: "message".to_string(),
                    ty: IpcSchemaType::Str,
                },
                IpcSchemaField {
                    name: "attempts".to_string(),
                    ty: IpcSchemaType::List(Box::new(IpcSchemaType::Int)),
                },
                IpcSchemaField {
                    name: "metadata".to_string(),
                    ty: IpcSchemaType::DictStr(Box::new(IpcSchemaType::Option(Box::new(
                        IpcSchemaType::Bytes,
                    )))),
                },
                IpcSchemaField {
                    name: "result".to_string(),
                    ty: IpcSchemaType::Result(
                        Box::new(IpcSchemaType::Tuple(vec![
                            IpcSchemaType::Bool,
                            IpcSchemaType::Float,
                        ])),
                        Box::new(IpcSchemaType::None),
                    ),
                },
            ],
        };

        assert_eq!(validate_ipc_payload_type(&payload), Ok(()));
    }

    #[test]
    fn rejects_unsupported_process_resource_payloads_inside_records() {
        let payload = IpcSchemaType::Record {
            name: "Request".to_string(),
            fields: vec![IpcSchemaField {
                name: "child".to_string(),
                ty: IpcSchemaType::Unsupported {
                    type_name: "sifr.process.Child".to_string(),
                },
            }],
        };

        assert_eq!(
            validate_ipc_payload_type(&payload),
            Err(IpcPayloadEligibilityError::UnsupportedPayload {
                type_name: "sifr.process.Child".to_string()
            })
        );
    }

    #[test]
    fn rejects_unsupported_task_payloads_inside_enum_variants() {
        let payload = IpcSchemaType::Enum {
            name: "Message".to_string(),
            variants: vec![
                IpcSchemaVariant {
                    name: "Ping".to_string(),
                    payload: None,
                },
                IpcSchemaVariant {
                    name: "Task".to_string(),
                    payload: Some(IpcSchemaType::Unsupported {
                        type_name: "sifr.task.Task".to_string(),
                    }),
                },
            ],
        };

        assert_eq!(
            validate_ipc_payload_type(&payload),
            Err(IpcPayloadEligibilityError::UnsupportedPayload {
                type_name: "sifr.task.Task".to_string()
            })
        );
    }

    #[test]
    fn rejects_unsupported_payloads_through_recursive_containers() {
        let cases = [
            IpcSchemaType::Unsupported {
                type_name: "sifr.process.Child".to_string(),
            },
            IpcSchemaType::Option(Box::new(IpcSchemaType::Unsupported {
                type_name: "sifr.process.OptionChild".to_string(),
            })),
            IpcSchemaType::List(Box::new(IpcSchemaType::Unsupported {
                type_name: "sifr.process.ListChild".to_string(),
            })),
            IpcSchemaType::DictStr(Box::new(IpcSchemaType::Unsupported {
                type_name: "sifr.process.DictChild".to_string(),
            })),
            IpcSchemaType::Result(
                Box::new(IpcSchemaType::Unsupported {
                    type_name: "sifr.process.ResultOkChild".to_string(),
                }),
                Box::new(IpcSchemaType::None),
            ),
            IpcSchemaType::Result(
                Box::new(IpcSchemaType::None),
                Box::new(IpcSchemaType::Unsupported {
                    type_name: "sifr.process.ResultErrChild".to_string(),
                }),
            ),
            IpcSchemaType::Tuple(vec![IpcSchemaType::Unsupported {
                type_name: "sifr.process.TupleChild".to_string(),
            }]),
        ];

        for payload in cases {
            assert!(matches!(
                validate_ipc_payload_type(&payload),
                Err(IpcPayloadEligibilityError::UnsupportedPayload { .. })
            ));
        }
    }

    #[test]
    fn eligibility_errors_do_not_render_payload_values() {
        let err = IpcPayloadEligibilityError::UnsupportedPayload {
            type_name: "sifr.process.PipeReader".to_string(),
        };

        assert_eq!(
            err.to_string(),
            "unsupported IPC payload type sifr.process.PipeReader"
        );
    }
}
