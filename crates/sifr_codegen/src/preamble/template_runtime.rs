use crate::{RustEnumVariant, RustItem, RustTrait, RustType, Visibility};

pub(crate) fn build_template_runtime_items() -> Vec<RustItem> {
    let range_fields = [
        "source_start",
        "source_end",
        "expression_start",
        "expression_end",
        "virtual_start",
        "virtual_end",
    ]
    .into_iter()
    .map(|name| (name.to_string(), RustType::I64));
    let mut interpolation_fields = vec![
        (
            "value".to_string(),
            RustType::Boxed(Box::new(RustType::DynTrait {
                trait_: RustTrait::Named {
                    name: "std::any::Any".to_string(),
                    params: Vec::new(),
                    associated_types: Vec::new(),
                },
                auto_traits: Vec::new(),
            })),
        ),
        ("value_type".to_string(), RustType::String_),
        ("expression".to_string(), RustType::String_),
        (
            "conversion".to_string(),
            RustType::Option(Box::new(RustType::Named("char".to_string()))),
        ),
        (
            "format_spec".to_string(),
            RustType::Option(Box::new(RustType::Named(
                "__SifrTemplateFormatSpec".to_string(),
            ))),
        ),
    ];
    interpolation_fields.extend(range_fields);

    vec![
        RustItem::Enum {
            name: "__SifrTemplateFormatSpecPart".to_string(),
            visibility: Visibility::Private,
            derives: Vec::new(),
            repr: None,
            variants: vec![
                RustEnumVariant {
                    name: "Literal".to_string(),
                    tuple_fields: Vec::new(),
                    fields: vec![("value".to_string(), RustType::String_)],
                    value: None,
                },
                RustEnumVariant {
                    name: "Interpolation".to_string(),
                    tuple_fields: Vec::new(),
                    fields: vec![
                        (
                            "value".to_string(),
                            RustType::Boxed(Box::new(RustType::DynTrait {
                                trait_: RustTrait::Named {
                                    name: "std::any::Any".to_string(),
                                    params: Vec::new(),
                                    associated_types: Vec::new(),
                                },
                                auto_traits: Vec::new(),
                            })),
                        ),
                        ("value_type".to_string(), RustType::String_),
                        (
                            "conversion".to_string(),
                            RustType::Option(Box::new(RustType::Named("char".to_string()))),
                        ),
                        (
                            "format_spec".to_string(),
                            RustType::Option(Box::new(RustType::Boxed(Box::new(RustType::Named(
                                "__SifrTemplateFormatSpec".to_string(),
                            ))))),
                        ),
                        ("source_start".to_string(), RustType::I64),
                        ("source_end".to_string(), RustType::I64),
                    ],
                    value: None,
                },
            ],
        },
        RustItem::Struct {
            name: "__SifrTemplateFormatSpec".to_string(),
            visibility: Visibility::Private,
            derives: Vec::new(),
            fields: vec![(
                "parts".to_string(),
                RustType::Vec(Box::new(RustType::Named(
                    "__SifrTemplateFormatSpecPart".to_string(),
                ))),
            )],
        },
        RustItem::Struct {
            name: "__SifrTemplateInterpolation".to_string(),
            visibility: Visibility::Private,
            derives: Vec::new(),
            fields: interpolation_fields,
        },
        RustItem::Struct {
            name: "__SifrTemplate".to_string(),
            visibility: Visibility::Private,
            derives: Vec::new(),
            fields: vec![
                (
                    "strings".to_string(),
                    RustType::Vec(Box::new(RustType::String_)),
                ),
                (
                    "interpolations".to_string(),
                    RustType::Vec(Box::new(RustType::Named(
                        "__SifrTemplateInterpolation".to_string(),
                    ))),
                ),
            ],
        },
    ]
}
