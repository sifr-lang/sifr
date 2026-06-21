use sifr_ir::{
    HirClass, HirFunction, HirModule, RustInteropDeclaration, RustInteropValue, RustTargetPath,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InteropBuildPlan {
    pub rust: RustInteropPlan,
}

impl InteropBuildPlan {
    #[must_use]
    pub fn cache_key_fragment(&self) -> String {
        let mut out = String::new();
        out.push_str("rust.declarations=");
        out.push_str(&self.rust.declarations.len().to_string());
        out.push('\n');
        for declaration in &self.rust.declarations {
            out.push_str("module=");
            out.push_str(declaration.module_name.as_deref().unwrap_or("<single>"));
            out.push('\n');
            out.push_str("owner=");
            push_owner(&mut out, &declaration.owner);
            out.push('\n');
            push_declaration(&mut out, &declaration.declaration);
            out.push('\n');
        }
        out
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RustInteropPlan {
    pub declarations: Vec<RustInteropPlanDeclaration>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustInteropPlanDeclaration {
    pub module_name: Option<String>,
    pub owner: RustInteropOwner,
    pub declaration: RustInteropDeclaration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RustInteropOwner {
    Function { name: String },
    Class { name: String },
    Method { class_name: String, name: String },
}

pub(crate) fn interop_build_plan_for_module(module: &HirModule) -> InteropBuildPlan {
    interop_build_plan_for_named_modules([(None, module)])
}

pub(crate) fn interop_build_plan_for_named_modules<'a>(
    modules: impl IntoIterator<Item = (Option<&'a str>, &'a HirModule)>,
) -> InteropBuildPlan {
    let mut rust = RustInteropPlan::default();
    for (module_name, module) in modules {
        collect_module_declarations(module_name, module, &mut rust.declarations);
    }
    InteropBuildPlan { rust }
}

fn collect_module_declarations(
    module_name: Option<&str>,
    module: &HirModule,
    declarations: &mut Vec<RustInteropPlanDeclaration>,
) {
    for function in &module.functions {
        collect_function_declarations(module_name, function, declarations);
    }
    for class in &module.classes {
        collect_class_declarations(module_name, class, declarations);
    }
}

fn collect_function_declarations(
    module_name: Option<&str>,
    function: &HirFunction,
    declarations: &mut Vec<RustInteropPlanDeclaration>,
) {
    extend_declarations(
        module_name,
        &RustInteropOwner::Function {
            name: function.name.clone(),
        },
        &function.rust_interop,
        declarations,
    );
}

fn collect_class_declarations(
    module_name: Option<&str>,
    class: &HirClass,
    declarations: &mut Vec<RustInteropPlanDeclaration>,
) {
    extend_declarations(
        module_name,
        &RustInteropOwner::Class {
            name: class.name.clone(),
        },
        &class.rust_interop,
        declarations,
    );
    for method in &class.methods {
        collect_method_declarations(module_name, &class.name, method, declarations);
    }
    for (_, method) in &class.operator_impls {
        collect_method_declarations(module_name, &class.name, method, declarations);
    }
}

fn collect_method_declarations(
    module_name: Option<&str>,
    class_name: &str,
    method: &HirFunction,
    declarations: &mut Vec<RustInteropPlanDeclaration>,
) {
    extend_declarations(
        module_name,
        &RustInteropOwner::Method {
            class_name: class_name.to_string(),
            name: method.name.clone(),
        },
        &method.rust_interop,
        declarations,
    );
}

fn extend_declarations(
    module_name: Option<&str>,
    owner: &RustInteropOwner,
    rust_interop: &[RustInteropDeclaration],
    declarations: &mut Vec<RustInteropPlanDeclaration>,
) {
    declarations.extend(rust_interop.iter().cloned().map(|declaration| {
        RustInteropPlanDeclaration {
            module_name: module_name.map(str::to_string),
            owner: owner.clone(),
            declaration,
        }
    }));
}

fn push_owner(out: &mut String, owner: &RustInteropOwner) {
    match owner {
        RustInteropOwner::Function { name } => {
            out.push_str("function:");
            out.push_str(name);
        }
        RustInteropOwner::Class { name } => {
            out.push_str("class:");
            out.push_str(name);
        }
        RustInteropOwner::Method { class_name, name } => {
            out.push_str("method:");
            out.push_str(class_name);
            out.push('.');
            out.push_str(name);
        }
    }
}

fn push_declaration(out: &mut String, declaration: &RustInteropDeclaration) {
    out.push_str("kind=");
    out.push_str(match declaration.kind {
        sifr_ir::RustInteropDecoratorKind::Function => "function",
        sifr_ir::RustInteropDecoratorKind::Opaque => "opaque",
        sifr_ir::RustInteropDecoratorKind::Async => "async",
        sifr_ir::RustInteropDecoratorKind::ZeroCopy => "zero_copy",
        sifr_ir::RustInteropDecoratorKind::View => "view",
    });
    out.push_str(";target=");
    if let Some(target) = &declaration.target {
        push_target_path(out, target);
    } else {
        out.push_str("<none>");
    }
    out.push_str(";span=");
    push_span(out, declaration.span);
    out.push_str(";effect=");
    out.push_str(match declaration.effect {
        sifr_ir::RustInteropEffect::Sync => "sync",
        sifr_ir::RustInteropEffect::Async => "async",
        sifr_ir::RustInteropEffect::BlockingIo => "blocking_io",
        sifr_ir::RustInteropEffect::CpuHeavy => "cpu_heavy",
    });
    out.push_str(";abi=");
    out.push_str(if declaration.abi_requirements.async_boundary {
        "async:1"
    } else {
        "async:0"
    });
    out.push(',');
    out.push_str(if declaration.abi_requirements.opaque_handle {
        "opaque:1"
    } else {
        "opaque:0"
    });
    out.push(',');
    out.push_str(if declaration.abi_requirements.zero_copy {
        "zero_copy:1"
    } else {
        "zero_copy:0"
    });
    out.push(',');
    out.push_str(if declaration.abi_requirements.view {
        "view:1"
    } else {
        "view:0"
    });
    for argument in &declaration.arguments {
        out.push_str(";arg=");
        out.push_str(argument.name.as_deref().unwrap_or("<pos>"));
        out.push(':');
        push_value(out, &argument.value);
        out.push('@');
        push_span(out, argument.span);
    }
}

fn push_value(out: &mut String, value: &RustInteropValue) {
    match value {
        RustInteropValue::Boolean(value) => {
            out.push_str(if *value { "bool:true" } else { "bool:false" });
        }
        RustInteropValue::Symbol(value) => {
            out.push_str("symbol:");
            out.push_str(value);
        }
        RustInteropValue::Integer(value) => {
            out.push_str("int:");
            out.push_str(&value.to_string());
        }
        RustInteropValue::PolicyCall {
            name,
            argument,
            span,
        } => {
            out.push_str("policy:");
            out.push_str(name);
            out.push('(');
            push_value(out, argument);
            out.push_str(")@");
            push_span(out, *span);
        }
        RustInteropValue::TargetPath(path) => {
            out.push_str("path:");
            push_target_path(out, path);
        }
    }
}

fn push_target_path(out: &mut String, path: &RustTargetPath) {
    out.push_str(&path.segments.join("."));
    out.push('@');
    push_span(out, path.span);
}

fn push_span(out: &mut String, span: ruff_text_size::TextRange) {
    out.push_str(&span.start().to_u32().to_string());
    out.push_str("..");
    out.push_str(&span.end().to_u32().to_string());
}

#[cfg(test)]
mod tests {
    use super::*;
    use sifr_ir::{
        HirClass, HirClassKind, HirFunction, MethodKind, RustInteropAbiRequirements,
        RustInteropDecoratorKind, RustInteropEffect, RustTargetPath,
    };
    use sifr_type_system::Type;

    #[test]
    fn interop_build_plan_collects_function_class_and_method_declarations() {
        let module = HirModule {
            functions: vec![function_with_declaration(
                "hash",
                RustInteropDecoratorKind::Function,
                "bridge.hash.digest",
            )],
            classes: vec![HirClass {
                name: "Consumer".to_string(),
                fields: Vec::new(),
                methods: vec![function_with_declaration(
                    "poll",
                    RustInteropDecoratorKind::Function,
                    "Self.poll",
                )],
                is_hashable: false,
                is_error_type: false,
                kind: HirClassKind::Regular,
                operator_impls: Vec::new(),
                newtype_inner: None,
                implements_protocols: Vec::new(),
                parent_class: None,
                type_params: Vec::new(),
                enum_variants: Vec::new(),
                rust_interop: vec![declaration(
                    RustInteropDecoratorKind::Opaque,
                    "bridge.kafka.Consumer",
                )],
            }],
            imports: Vec::new(),
            constants: Vec::new(),
            generic_functions: std::collections::HashMap::new(),
            type_param_bounds: std::collections::HashMap::new(),
        };

        let plan = interop_build_plan_for_named_modules([(Some("main"), &module)]);

        assert_eq!(plan.rust.declarations.len(), 3);
        assert!(plan.rust.declarations.iter().any(|entry| {
            entry.module_name.as_deref() == Some("main")
                && matches!(entry.owner, RustInteropOwner::Function { ref name } if name == "hash")
        }));
        assert!(plan.rust.declarations.iter().any(|entry| {
            matches!(entry.owner, RustInteropOwner::Class { ref name } if name == "Consumer")
                && entry.declaration.kind == RustInteropDecoratorKind::Opaque
        }));
        assert!(plan.rust.declarations.iter().any(|entry| {
            matches!(
                entry.owner,
                RustInteropOwner::Method {
                    ref class_name,
                    ref name,
                } if class_name == "Consumer" && name == "poll"
            )
        }));
        let cache_key_fragment = plan.cache_key_fragment();
        assert!(cache_key_fragment.contains("owner=function:hash"));
        assert!(cache_key_fragment.contains("kind=opaque"));
        assert!(cache_key_fragment.contains("target=bridge.kafka.Consumer@0..0"));
    }

    fn function_with_declaration(
        name: &str,
        kind: RustInteropDecoratorKind,
        target: &str,
    ) -> HirFunction {
        HirFunction {
            name: name.to_string(),
            params: Vec::new(),
            return_type: Type::None,
            body: Vec::new(),
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: Vec::new(),
            rust_interop: vec![declaration(kind, target)],
            type_params: Vec::new(),
        }
    }

    fn declaration(kind: RustInteropDecoratorKind, target: &str) -> RustInteropDeclaration {
        RustInteropDeclaration {
            kind,
            target: Some(RustTargetPath {
                segments: target.split('.').map(str::to_string).collect(),
                span: Default::default(),
            }),
            arguments: Vec::new(),
            span: Default::default(),
            effect: RustInteropEffect::Sync,
            abi_requirements: RustInteropAbiRequirements::default(),
        }
    }
}
