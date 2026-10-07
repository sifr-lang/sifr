fn nominal_builtin_receivers() -> Vec<(Type, &'static str)> {
    let enumeration = Type::Enum {
        identity: Some("fixture.Status".to_string()),
        name: "Status".to_string(),
        variants: vec![("READY".to_string(), Some(7))].into(),
    };
    let mut receivers = vec![(enumeration.clone(), "name"), (enumeration, "value")];
    for inner in [Type::Int, Type::Str] {
        receivers.push((
            Type::Newtype {
                identity: Some("fixture.Wrapper".to_string()),
                name: "Wrapper".to_string(),
                inner: Box::new(inner),
            },
            "value",
        ));
    }
    let aliases = receivers
        .iter()
        .map(|(ty, method)| (Type::alias("NominalAlias", ty.clone()), *method))
        .collect::<Vec<_>>();
    receivers.extend(aliases);
    receivers
}

#[test]
fn nominal_builtin_methods_dispatch_with_resolved_receiver_and_zero_arguments() {
    for (ty, method) in nominal_builtin_receivers() {
        let expression = builtin(ty, method, Vec::new());
        assert_eq!(
            crate::method_call_emitter::source_method_path(&expression).unwrap(),
            crate::method_call_emitter::SourceMethodPath::BuiltinIntrinsic
        );
        let lowered = RustEmitter::new()
            .lower_stmt_expr_for_ir(&expression)
            .unwrap()
            .unwrap();
        assert_eq!(render_expr(&lowered), format!("value.{method}()"));
    }
}

#[test]
fn nominal_builtin_methods_reject_invalid_arity_receiver_and_authority() {
    for (ty, method) in nominal_builtin_receivers() {
        for expression in [
            builtin(ty.clone(), method, vec![HirExpr::IntLiteral(1)]),
            builtin(ty.clone(), "invented", Vec::new()),
            call(
                ty.clone(),
                method,
                MethodAuthority::Unclassified,
                Vec::new(),
            ),
            call(
                ty.clone(),
                method,
                MethodAuthority::BuiltinIntrinsic {
                    declaration: identity("wrong.module", method),
                },
                Vec::new(),
            ),
            call(
                ty,
                method,
                MethodAuthority::BuiltinIntrinsic {
                    declaration: identity("sifr.builtin", "different_method"),
                },
                Vec::new(),
            ),
        ] {
            assert!(
                RustEmitter::new()
                    .lower_stmt_expr_for_ir(&expression)
                    .is_err()
            );
        }
    }
    for ty in [
        Type::Int,
        Type::Str,
        Type::List(Box::new(Type::Int)),
        Type::Class {
            identity: None,
            type_args: Vec::new(),
            name: "Owner".to_string(),
            fields: Default::default(),
            methods: Default::default(),
            parent_class: None,
        },
    ] {
        for method in ["name", "value"] {
            let expression = builtin(ty.clone(), method, Vec::new());
            assert!(
                RustEmitter::new()
                    .lower_stmt_expr_for_ir(&expression)
                    .is_err()
            );
        }
    }
    let wrapper = Type::Newtype {
        identity: None,
        name: "Wrapper".to_string(),
        inner: Box::new(Type::Int),
    };
    assert!(
        RustEmitter::new()
            .lower_stmt_expr_for_ir(&builtin(wrapper, "name", Vec::new()))
            .is_err()
    );
}

#[test]
fn nominal_builtin_methods_survive_real_source_codegen() {
    let generated = crate::lib_codegen_tests::generate_rust_from_source(
        r#"
class Status(Enum):
    READY = 7

    def label(self) -> str:
        return "custom"

class Port(int):
    pass

class Label(str):
    pass

def name(value: Status) -> str:
    return value.name()

def number(value: Status) -> int:
    return value.value()

def port(value: Port) -> int:
    return value.value()

def label(value: Label) -> str:
    return value.value()

def main():
    value: Status = Status.READY
    print(name(value))
    print(number(value))
    print(value.label())
    print(port(Port(8080)))
    print(label(Label("owned")))
"#,
    );
    assert!(!generated.contains("compile_error!"), "{generated}");
    assert!(generated.contains(".name()"), "{generated}");
    assert!(generated.contains(".value()"), "{generated}");
    assert!(generated.contains(".label()"), "{generated}");
    syn::parse_file(&generated).expect("nominal builtin methods should generate valid Rust syntax");
}
