use crate::{
    HashMap, RustExpr, RustItem, RustLiteral, RustParam, RustStmt, RustType, RustTypeParam,
    Visibility,
};
use sifr_type_system::StructuralRecordType;

pub(crate) fn structural_record_rust_type(record: &StructuralRecordType) -> RustType {
    // Structural layouts are emitted in the current standalone module. Project
    // modules import the canonical crate-root layout explicitly, which keeps
    // the generated module relocatable (including when test harnesses nest it).
    let base = crate::structural_identity_codegen::structural_record_layout_rust_name(record);
    let arguments = record
        .fields()
        .iter()
        .map(|field| crate::sifr_type_to_rust_type(field.ty()))
        .collect::<Vec<_>>();
    if arguments.is_empty() {
        RustType::Named(base)
    } else {
        RustType::Generic {
            base,
            params: arguments,
        }
    }
}

fn field_type_params(record: &StructuralRecordType) -> Vec<String> {
    (0..record.fields().len())
        .map(|index| format!("__SifrField{index}"))
        .collect()
}

fn generic_name(name: &str, arguments: &[String]) -> String {
    if arguments.is_empty() {
        name.to_string()
    } else {
        format!("{name}<{}>", arguments.join(", "))
    }
}

fn rust_type_params(names: &[String], bounds: &[String]) -> Vec<RustTypeParam> {
    names
        .iter()
        .map(|name| RustTypeParam {
            name: name.clone(),
            bounds: bounds.to_vec(),
        })
        .collect()
}

impl crate::RustEmitter {
    pub(crate) fn generate_structural_record_definitions(&mut self) {
        let mut records = self
            .structural_record_types
            .values()
            .cloned()
            .collect::<Vec<_>>();
        records.sort_by_key(crate::structural_identity_codegen::structural_record_rust_name);

        let mut layouts = HashMap::new();
        for record in &records {
            layouts
                .entry(
                    crate::structural_identity_codegen::structural_record_layout_rust_name(record),
                )
                .or_insert_with(|| record.clone());
        }
        let mut layouts = layouts.into_iter().collect::<Vec<_>>();
        layouts.sort_by(|left, right| left.0.cmp(&right.0));

        for (layout_name, record) in &layouts {
            let field_params = field_type_params(record);
            let declared_name = generic_name(layout_name, &field_params);
            let fields = record
                .fields()
                .iter()
                .zip(&field_params)
                .map(|(field, parameter)| {
                    (field.name().to_string(), RustType::Named(parameter.clone()))
                })
                .collect::<Vec<_>>();
            self.body_items.push(RustItem::Struct {
                name: declared_name.clone(),
                visibility: Visibility::Pub,
                derives: vec![
                    "Debug".to_string(),
                    "Clone".to_string(),
                    "Copy".to_string(),
                    "PartialEq".to_string(),
                    "Eq".to_string(),
                    "Hash".to_string(),
                    "PartialOrd".to_string(),
                    "Ord".to_string(),
                ],
                fields: fields.clone(),
            });
            self.body_items.push(RustItem::Impl {
                target: declared_name.clone(),
                type_params: rust_type_params(&field_params, &[]),
                trait_: None,
                items: vec![RustItem::Fn {
                    name: "new".to_string(),
                    visibility: Visibility::Pub,
                    type_params: Vec::new(),
                    params: fields
                        .iter()
                        .map(|(field, ty)| RustParam::Named {
                            name: field.clone(),
                            ty: ty.clone(),
                        })
                        .collect(),
                    ret: Some(RustType::Named("Self".to_string())),
                    body: vec![RustStmt::Return(Some(RustExpr::StructInit {
                        name: "Self".to_string(),
                        fields: fields
                            .iter()
                            .map(|(field, _)| (field.clone(), RustExpr::Ident(field.clone())))
                            .collect(),
                    }))],
                    is_async: false,
                }],
            });

            let format_string = format!(
                "{{{{{}}}}}",
                record
                    .fields()
                    .iter()
                    .map(|field| format!("{}: {{}}", field.name()))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            let mut args = vec![
                RustExpr::Ident("f".to_string()),
                RustExpr::Literal(RustLiteral::Str(format_string)),
            ];
            args.extend(record.fields().iter().map(|field| RustExpr::Ref {
                mutable: false,
                expr: Box::new(RustExpr::Field {
                    expr: Box::new(RustExpr::Ident("self".to_string())),
                    field: field.name().to_string(),
                }),
            }));
            self.body_items.push(RustItem::Impl {
                target: declared_name,
                type_params: rust_type_params(&field_params, &["std::fmt::Display".to_string()]),
                trait_: Some("std::fmt::Display".to_string()),
                items: vec![RustItem::Fn {
                    name: "fmt".to_string(),
                    visibility: Visibility::Private,
                    type_params: Vec::new(),
                    params: vec![
                        RustParam::SelfParam { mutable: false },
                        RustParam::Named {
                            name: "f".to_string(),
                            ty: RustType::Ref {
                                mutable: true,
                                inner: Box::new(RustType::Named(
                                    "std::fmt::Formatter<'_>".to_string(),
                                )),
                            },
                        },
                    ],
                    ret: Some(RustType::Named("std::fmt::Result".to_string())),
                    body: vec![RustStmt::Return(Some(RustExpr::MacroCall {
                        name: "write".to_string(),
                        args,
                    }))],
                    is_async: false,
                }],
            });
        }

        for (target_layout, target) in &layouts {
            let trait_name =
                crate::structural_identity_codegen::structural_record_view_trait_name(target);
            debug_assert_eq!(
                trait_name,
                target_layout.replacen("__SifrRecord_", "__SifrRecordView_", 1)
            );
            let target_field_params = field_type_params(target);
            self.body_items.push(RustItem::Trait {
                name: generic_name(&trait_name, &target_field_params),
                visibility: Visibility::Pub,
                supertraits: Vec::new(),
                methods: target
                    .fields()
                    .iter()
                    .zip(&target_field_params)
                    .map(|(field, parameter)| RustItem::TraitMethodSig {
                        name: format!("__sifr_record_field_{}", field.name()),
                        params: vec![RustParam::SelfParam { mutable: false }],
                        ret: Some(RustType::Ref {
                            mutable: false,
                            inner: Box::new(RustType::Named(parameter.clone())),
                        }),
                        is_async: false,
                    })
                    .collect(),
            });

            for (source_layout, source) in &layouts {
                if !target
                    .fields()
                    .iter()
                    .all(|field| source.field(field.name()).is_some())
                {
                    continue;
                }
                let source_field_params = field_type_params(source);
                let source_field_indices = source
                    .fields()
                    .iter()
                    .enumerate()
                    .map(|(index, field)| (field.name(), index))
                    .collect::<HashMap<_, _>>();
                let trait_arguments = target
                    .fields()
                    .iter()
                    .map(|target_field| {
                        let index = source_field_indices[target_field.name()];
                        source_field_params[index].clone()
                    })
                    .collect::<Vec<_>>();
                self.body_items.push(RustItem::Impl {
                    target: generic_name(source_layout, &source_field_params),
                    type_params: rust_type_params(&source_field_params, &[]),
                    trait_: Some(generic_name(&trait_name, &trait_arguments)),
                    items: target
                        .fields()
                        .iter()
                        .map(|field| {
                            let index = source_field_indices[field.name()];
                            RustItem::Fn {
                                name: format!("__sifr_record_field_{}", field.name()),
                                visibility: Visibility::Private,
                                type_params: Vec::new(),
                                params: vec![RustParam::SelfParam { mutable: false }],
                                ret: Some(RustType::Ref {
                                    mutable: false,
                                    inner: Box::new(RustType::Named(
                                        source_field_params[index].clone(),
                                    )),
                                }),
                                body: vec![RustStmt::Return(Some(RustExpr::Ref {
                                    mutable: false,
                                    expr: Box::new(RustExpr::Field {
                                        expr: Box::new(RustExpr::Ident("self".to_string())),
                                        field: field.name().to_string(),
                                    }),
                                }))],
                                is_async: false,
                            }
                        })
                        .collect(),
                });
            }
        }
    }
}
