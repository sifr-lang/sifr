//! Canonical identities for compiler-checked callable const values.

use sifr_lowering::{
    CallableIdentity, ExternalDefs, HirClass, HirClassKind, HirFunction, LoweringResult,
    MethodKind, canonicalize_user_export_function_type,
};
use sifr_python_ast::Expr;
use sifr_type_system::{FunctionType, ParamConvention, Type};
use std::collections::HashMap;

pub(crate) fn resolve(
    module_name: &str,
    result: &LoweringResult,
    external_defs: &ExternalDefs,
    expression: &Expr,
) -> Option<CallableIdentity> {
    let resolver = Resolver {
        module_name,
        result,
        external_defs,
    };
    match expression {
        Expr::Name(name) => resolver.free_or_constructor(name.id.as_str(), Vec::new()),
        Expr::Subscript(subscript) => {
            let Expr::Name(name) = subscript.value.as_ref() else {
                return None;
            };
            let arguments = match subscript.slice.as_ref() {
                Expr::Tuple(tuple) => tuple
                    .elts
                    .iter()
                    .map(|argument| resolver.type_identity(argument))
                    .collect(),
                single => vec![resolver.type_identity(single)],
            };
            resolver.free_or_constructor(name.id.as_str(), arguments)
        }
        Expr::Attribute(attribute) => {
            let Expr::Name(owner) = attribute.value.as_ref() else {
                return None;
            };
            resolver.method(owner.id.as_str(), attribute.attr.as_str())
        }
        _ => None,
    }
}

pub(crate) fn contract_for_identity(
    module_name: &str,
    result: &LoweringResult,
    external_defs: &ExternalDefs,
    identity: &CallableIdentity,
) -> Option<FunctionType> {
    Resolver {
        module_name,
        result,
        external_defs,
    }
    .contract(identity)
}

pub(crate) fn method_declaration(
    module_name: &str,
    result: &LoweringResult,
    owner: &str,
    method_name: &str,
) -> Option<CallableIdentity> {
    let class = result
        .module
        .classes
        .iter()
        .find(|class| class.name == owner)?;
    let hir_name = if method_name == "__init__" {
        "new"
    } else {
        method_name
    };
    let method = class
        .methods
        .iter()
        .chain(class.operator_impls.iter().map(|(_, method)| method))
        .find(|method| method.name == hir_name)?;
    let local_classes = result
        .module
        .classes
        .iter()
        .map(|class| (class.name.clone(), format!("{module_name}.{}", class.name)))
        .collect();
    let function = canonicalize_user_export_function_type(&function_type(method), &local_classes);
    Some(CallableIdentity {
        module: module_name.to_string(),
        owner: Some(format!("{module_name}.{owner}")),
        symbol: method_name.to_string(),
        generic_arguments: Vec::new(),
        signature: signature(&function),
    })
}

struct Resolver<'a> {
    module_name: &'a str,
    result: &'a LoweringResult,
    external_defs: &'a ExternalDefs,
}

impl Resolver<'_> {
    fn free_or_constructor(
        &self,
        local: &str,
        generic_arguments: Vec<String>,
    ) -> Option<CallableIdentity> {
        if let Some(function) = self
            .result
            .module
            .functions
            .iter()
            .find(|item| item.name == local)
        {
            return Some(CallableIdentity {
                module: self.module_name.to_string(),
                owner: None,
                symbol: local.to_string(),
                generic_arguments,
                signature: signature(&self.canonical_local_function_type(function)),
            });
        }
        if let Some(class) = self
            .result
            .module
            .classes
            .iter()
            .find(|class| class.name == local)
        {
            return self.local_constructor(class, generic_arguments);
        }
        for import in &self.result.module.imports {
            for name in &import.names {
                if imported_local_name(import, name) != local {
                    continue;
                }
                if let Some(function) = self
                    .external_defs
                    .functions
                    .get(&import.module)
                    .and_then(|functions| functions.get(name))
                {
                    return Some(CallableIdentity {
                        module: import.module.clone(),
                        owner: None,
                        symbol: name.clone(),
                        generic_arguments,
                        signature: signature(function),
                    });
                }
                if let Some(ty) = self
                    .external_defs
                    .classes
                    .get(&import.module)
                    .and_then(|classes| classes.get(name))
                {
                    return self.imported_constructor(&import.module, name, ty, generic_arguments);
                }
            }
        }
        builtin_factory_type(local).map(|function| CallableIdentity {
            module: "sifr.builtins".to_string(),
            owner: None,
            symbol: local.to_string(),
            generic_arguments,
            signature: signature(&function),
        })
    }

    fn contract(&self, identity: &CallableIdentity) -> Option<FunctionType> {
        if identity.module == "sifr.builtins" && identity.owner.is_none() {
            return builtin_factory_type(&identity.symbol)
                .filter(|function| signature(function) == identity.signature);
        }
        if identity.module == self.module_name {
            if let Some(owner) = &identity.owner {
                let owner_name = owner
                    .rsplit_once('.')
                    .map_or(owner.as_str(), |(_, name)| name);
                let class = self
                    .result
                    .module
                    .classes
                    .iter()
                    .find(|class| class.name == owner_name)?;
                let function = if identity.symbol == "__init__" {
                    let return_type = class_type(self.module_name, class);
                    class
                        .methods
                        .iter()
                        .find(|method| method.name == "new")
                        .map_or_else(
                            || constructor_type(&class.fields, return_type.clone()),
                            |method| {
                                let mut function = self.canonical_local_function_type(method);
                                function.return_type = Box::new(return_type.clone());
                                function
                            },
                        )
                } else {
                    let method = class
                        .methods
                        .iter()
                        .find(|method| method.name == identity.symbol)?;
                    if method.method_kind == MethodKind::Regular {
                        return None;
                    }
                    self.canonical_local_function_type(method)
                };
                return (signature(&function) == identity.signature).then_some(function);
            }
            let function = self
                .result
                .module
                .functions
                .iter()
                .find(|function| function.name == identity.symbol)
                .map(|function| self.canonical_local_function_type(function))?;
            return (signature(&function) == identity.signature).then_some(function);
        }
        if let Some(owner) = &identity.owner {
            let owner_name = owner
                .rsplit_once('.')
                .map_or(owner.as_str(), |(_, name)| name);
            if identity.symbol == "__init__" {
                let ty = self
                    .external_defs
                    .classes
                    .get(&identity.module)?
                    .get(owner_name)?;
                let Type::Class { fields, .. } = ty.resolve_alias() else {
                    return None;
                };
                let function = self
                    .external_defs
                    .structural_methods_for(&identity.module)
                    .and_then(|classes| classes.get(owner_name))
                    .and_then(|methods| methods.iter().find(|method| method.name == "__init__"))
                    .map_or_else(
                        || constructor_type(fields, ty.clone()),
                        |method| FunctionType {
                            receiver: method.receiver,
                            params: method
                                .params
                                .iter()
                                .map(|parameter| {
                                    (
                                        parameter.name.clone(),
                                        parameter.ty.clone(),
                                        parameter.convention,
                                    )
                                })
                                .collect(),
                            return_type: Box::new(ty.clone()),
                        },
                    );
                return (signature(&function) == identity.signature).then_some(function);
            }
            let method = self
                .external_defs
                .structural_methods_for(&identity.module)?
                .get(owner_name)?
                .iter()
                .find(|method| method.name == identity.symbol)?;
            if method.method_kind == MethodKind::Regular {
                return None;
            }
            let function = FunctionType {
                receiver: method.receiver,
                params: method
                    .params
                    .iter()
                    .map(|parameter| {
                        (
                            parameter.name.clone(),
                            parameter.ty.clone(),
                            parameter.convention,
                        )
                    })
                    .collect(),
                return_type: Box::new(method.return_type.clone()),
            };
            return (signature(&function) == identity.signature).then_some(function);
        }
        let function = self
            .external_defs
            .functions
            .get(&identity.module)?
            .get(&identity.symbol)?
            .clone();
        (signature(&function) == identity.signature).then_some(function)
    }

    fn method(&self, owner: &str, method_name: &str) -> Option<CallableIdentity> {
        if let Some(class) = self
            .result
            .module
            .classes
            .iter()
            .find(|class| class.name == owner)
        {
            let function = class.methods.iter().find(|item| item.name == method_name)?;
            if function.method_kind == MethodKind::Regular {
                return None;
            }
            return Some(CallableIdentity {
                module: self.module_name.to_string(),
                owner: Some(format!("{}.{}", self.module_name, owner)),
                symbol: method_name.to_string(),
                generic_arguments: Vec::new(),
                signature: signature(&self.canonical_local_function_type(function)),
            });
        }
        for import in &self.result.module.imports {
            for name in &import.names {
                if imported_local_name(import, name) != owner {
                    continue;
                }
                let method = self
                    .external_defs
                    .structural_methods_for(&import.module)?
                    .get(name)?
                    .iter()
                    .find(|candidate| candidate.name == method_name)?;
                if method.method_kind == MethodKind::Regular {
                    return None;
                }
                let function = FunctionType {
                    receiver: method.receiver,
                    params: method
                        .params
                        .iter()
                        .map(|parameter| {
                            (
                                parameter.name.clone(),
                                parameter.ty.clone(),
                                parameter.convention,
                            )
                        })
                        .collect(),
                    return_type: Box::new(method.return_type.clone()),
                };
                return Some(CallableIdentity {
                    module: import.module.clone(),
                    owner: Some(format!("{}.{}", import.module, name)),
                    symbol: method.name.clone(),
                    generic_arguments: Vec::new(),
                    signature: signature(&function),
                });
            }
        }
        None
    }

    fn local_constructor(
        &self,
        class: &HirClass,
        generic_arguments: Vec<String>,
    ) -> Option<CallableIdentity> {
        if !matches!(class.kind, HirClassKind::Regular) {
            return None;
        }
        let return_type = class_type(self.module_name, class);
        let function = class
            .methods
            .iter()
            .find(|method| method.name == "new")
            .map_or_else(
                || constructor_type(&class.fields, return_type.clone()),
                |method| {
                    let mut function = self.canonical_local_function_type(method);
                    function.return_type = Box::new(return_type.clone());
                    function
                },
            );
        Some(CallableIdentity {
            module: self.module_name.to_string(),
            owner: Some(format!("{}.{}", self.module_name, class.name)),
            symbol: "__init__".to_string(),
            generic_arguments,
            signature: signature(&function),
        })
    }

    fn imported_constructor(
        &self,
        module: &str,
        name: &str,
        ty: &Type,
        generic_arguments: Vec<String>,
    ) -> Option<CallableIdentity> {
        let Type::Class { fields, .. } = ty.resolve_alias() else {
            return None;
        };
        let return_type = ty.clone();
        let function = self
            .external_defs
            .structural_methods_for(module)
            .and_then(|classes| classes.get(name))
            .and_then(|methods| methods.iter().find(|method| method.name == "__init__"))
            .map_or_else(
                || constructor_type(fields, return_type.clone()),
                |method| FunctionType {
                    receiver: method.receiver,
                    params: method
                        .params
                        .iter()
                        .map(|parameter| {
                            (
                                parameter.name.clone(),
                                parameter.ty.clone(),
                                parameter.convention,
                            )
                        })
                        .collect(),
                    return_type: Box::new(return_type.clone()),
                },
            );
        Some(CallableIdentity {
            module: module.to_string(),
            owner: Some(format!("{module}.{name}")),
            symbol: "__init__".to_string(),
            generic_arguments,
            signature: signature(&function),
        })
    }

    fn canonical_local_function_type(&self, function: &HirFunction) -> FunctionType {
        canonicalize_user_export_function_type(&function_type(function), &self.local_classes())
    }

    fn local_classes(&self) -> HashMap<String, String> {
        self.result
            .module
            .classes
            .iter()
            .map(|class| {
                (
                    class.name.clone(),
                    format!("{}.{}", self.module_name, class.name),
                )
            })
            .collect()
    }

    fn type_identity(&self, expression: &Expr) -> String {
        match expression {
            Expr::Name(name) => {
                if let Some(class) = self
                    .result
                    .module
                    .classes
                    .iter()
                    .find(|class| class.name == name.id.as_str())
                {
                    return format!("{}.{}", self.module_name, class.name);
                }
                if let Some(alias) = self.result.type_aliases.get(name.id.as_str()) {
                    return crate::canonical_types::type_identity(alias.resolve_alias());
                }
                for import in &self.result.module.imports {
                    for imported in &import.names {
                        if imported_local_name(import, imported) != name.id.as_str() {
                            continue;
                        }
                        if let Some(ty) = self
                            .external_defs
                            .classes
                            .get(&import.module)
                            .and_then(|classes| classes.get(imported))
                        {
                            return crate::canonical_types::type_identity(ty.resolve_alias());
                        }
                    }
                }
                name.id.to_string()
            }
            Expr::Attribute(attribute) => {
                format!(
                    "{}.{}",
                    self.type_identity(&attribute.value),
                    attribute.attr
                )
            }
            Expr::Subscript(subscript) => format!(
                "{}[{}]",
                self.type_identity(&subscript.value),
                self.type_identity(&subscript.slice)
            ),
            Expr::Tuple(tuple) => tuple
                .elts
                .iter()
                .map(|item| self.type_identity(item))
                .collect::<Vec<_>>()
                .join(","),
            Expr::BinOp(binary) if matches!(binary.op, sifr_python_ast::Operator::BitOr) => {
                format!(
                    "{}|{}",
                    self.type_identity(&binary.left),
                    self.type_identity(&binary.right)
                )
            }
            _ => "<type>".to_string(),
        }
    }
}

fn imported_local_name<'a>(import: &'a sifr_lowering::HirImport, name: &'a str) -> &'a str {
    import
        .aliases
        .iter()
        .find(|(original, _)| original == name)
        .map_or(name, |(_, alias)| alias.as_str())
}

fn class_type(module_name: &str, class: &HirClass) -> Type {
    Type::Class {
        identity: Some(
            class
                .identity
                .clone()
                .unwrap_or_else(|| format!("{module_name}.{}", class.name)),
        ),
        type_args: class
            .type_params
            .iter()
            .cloned()
            .map(Type::TypeVar)
            .collect(),
        name: class.name.clone(),
        fields: class.fields.clone(),
        methods: Vec::new(),
        parent_class: class.semantic_parent_chain(),
    }
}

fn constructor_type(fields: &[(String, Type)], return_type: Type) -> FunctionType {
    FunctionType {
        receiver: None,
        params: fields
            .iter()
            .map(|(name, ty)| (name.clone(), ty.clone(), ParamConvention::own()))
            .collect(),
        return_type: Box::new(return_type),
    }
}

fn builtin_factory_type(name: &str) -> Option<FunctionType> {
    let return_type = match name {
        "list" => Type::List(Box::new(Type::Any)),
        "set" => Type::Set(Box::new(Type::Any)),
        "dict" => Type::Dict(Box::new(Type::Any), Box::new(Type::Any)),
        "str" => Type::Str,
        "bytes" => Type::Bytes,
        "int" => Type::Int,
        "float" => Type::Float,
        "bool" => Type::Bool,
        _ => return None,
    };
    Some(FunctionType::new(Vec::new(), return_type))
}

fn function_type(function: &HirFunction) -> FunctionType {
    FunctionType {
        receiver: function.receiver,
        params: function
            .params
            .iter()
            .map(|parameter| {
                (
                    parameter.name.clone(),
                    parameter.ty.clone(),
                    parameter.convention,
                )
            })
            .collect(),
        return_type: Box::new(function.return_type.clone()),
    }
}

fn signature(function: &FunctionType) -> String {
    crate::canonical_types::function_identity(function)
}
