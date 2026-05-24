use crate::{
    sifr_type_to_rust_type, RustEmitter, RustEnumVariant, RustExpr, RustItem, RustLiteral,
    RustMatchArm, RustParam, RustStmt, RustType, Visibility,
};
use sifr_hir::HirModule;
use sifr_type_system::ParamConvention;
use sifr_type_system::Type;

impl RustEmitter {
    /// Collect all union types from the module that need enum definitions,
    /// and build a map of function signatures for call-site wrapping.
    pub(crate) fn collect_union_types(&mut self, module: &HirModule) {
        for func in &module.functions {
            // Record function signature with conventions
            let param_info: Vec<(Type, ParamConvention)> = func
                .params
                .iter()
                .map(|p| (p.ty.clone(), p.convention))
                .collect();
            self.func_signatures
                .insert(func.name.clone(), (param_info, func.return_type.clone()));

            // Track generator functions (contain yield statements)
            if crate::body_contains_yield(&func.body) {
                self.generator_functions.insert(func.name.clone());
            }

            // Check params
            for param in &func.params {
                self.register_union_type(&param.ty);
            }
            // Check return type
            self.register_union_type(&func.return_type);
            for ty in crate::hir_analysis::queries::collect_let_declared_types(&func.body) {
                self.register_union_type(&ty);
            }
        }
        // Also scan class method bodies and register their signatures
        for class in &module.classes {
            let mut has_constructor = false;
            for method in &class.methods {
                // Register method signature under ClassName::method_name
                let param_info: Vec<(Type, ParamConvention)> = method
                    .params
                    .iter()
                    .map(|p| {
                        let conv = if method.name == "new" {
                            ParamConvention::own()
                        } else {
                            p.convention
                        };
                        (p.ty.clone(), conv)
                    })
                    .collect();
                self.func_signatures.insert(
                    format!("{}::{}", class.name, method.name),
                    (param_info, method.return_type.clone()),
                );
                if method.name == "new" {
                    has_constructor = true;
                }

                for param in &method.params {
                    self.register_union_type(&param.ty);
                }
                self.register_union_type(&method.return_type);
                for ty in crate::hir_analysis::queries::collect_let_declared_types(&method.body) {
                    self.register_union_type(&ty);
                }
            }
            if !has_constructor {
                // Classes without an explicit `new` still get an auto-generated constructor.
                // Register it so call sites can apply ownership conventions correctly.
                let ctor_params = class
                    .fields
                    .iter()
                    .map(|(_, ty)| (ty.clone(), ParamConvention::own()))
                    .collect::<Vec<_>>();
                self.func_signatures.insert(
                    format!("{}::new", class.name),
                    (
                        ctor_params,
                        Type::Class {
                            name: class.name.clone(),
                            fields: class.fields.clone(),
                            methods: Vec::new(),
                            parent_class: class.parent_class.clone(),
                        },
                    ),
                );
            }
        }
    }

    pub(crate) fn register_union_type(&mut self, ty: &Type) {
        let resolved = crate::resolve_alias_type_for_plain_call(ty);
        if let Type::Union(members) = resolved {
            // Skip Option<T> pattern (T | None with exactly 2 members)
            let non_none: Vec<&Type> = members
                .iter()
                .filter(|m| !matches!(m, Type::None))
                .collect();
            let has_none = members.iter().any(|m| matches!(m, Type::None));
            if has_none && non_none.len() == 1 {
                return; // This maps to Option<T>, no enum needed
            }
            // Register the enum name and its member types
            let enum_name = resolved.union_enum_name();
            self.union_enums
                .entry(enum_name)
                .or_insert_with(|| members.clone());
        }
    }

    /// Generate Rust enum definitions for all collected union types.
    pub(crate) fn generate_enum_definitions(&mut self) {
        // Sort enum names for deterministic output
        let mut enums: Vec<(String, Vec<Type>)> = self.union_enums.clone().into_iter().collect();
        enums.sort_by(|a, b| a.0.cmp(&b.0));

        self.enum_items.clear();
        for (enum_name, members) in enums {
            let variants = members
                .iter()
                .map(|member| RustEnumVariant {
                    name: member.union_variant_name(),
                    tuple_fields: vec![sifr_type_to_rust_type(member)],
                    fields: Vec::new(),
                    value: None,
                })
                .collect();
            self.enum_items.push(RustItem::Enum {
                name: enum_name.clone(),
                visibility: Visibility::Private,
                derives: vec!["Debug".to_string(), "Clone".to_string()],
                repr: None,
                variants,
            });

            let arms: Vec<RustMatchArm> = members
                .iter()
                .map(|member| {
                    let variant = member.union_variant_name();
                    let fmt_spec = if matches!(member, Type::Class { .. }) {
                        "{:?}"
                    } else {
                        "{}"
                    };
                    RustMatchArm {
                        pattern: format!("{enum_name}::{variant}(v)"),
                        bindings: Vec::new(),
                        guard: None,
                        body: vec![RustStmt::Return(Some(RustExpr::MacroCall {
                            name: "write".to_string(),
                            args: vec![
                                RustExpr::Ident("f".to_string()),
                                RustExpr::Literal(RustLiteral::Str(fmt_spec.to_string())),
                                RustExpr::Ident("v".to_string()),
                            ],
                        }))],
                    }
                })
                .collect();

            self.enum_items.push(RustItem::Impl {
                target: enum_name,
                type_params: Vec::new(),
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
                    body: vec![RustStmt::Match {
                        expr: RustExpr::Ident("self".to_string()),
                        arms,
                    }],
                    is_async: false,
                }],
            });
        }
    }
}
