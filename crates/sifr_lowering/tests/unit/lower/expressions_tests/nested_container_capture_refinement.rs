use super::{HirStmt, Type, lower_source};

fn list_binding_type(source: &str, function_name: &str, binding: &str) -> Type {
    let module = lower_source(source).expect("source should lower");
    let function = module
        .functions
        .iter()
        .find(|function| function.name == function_name)
        .expect("function should lower");
    binding_type_in_stmts(&function.body, binding).expect("binding should lower")
}

fn binding_type_in_stmts(stmts: &[HirStmt], binding: &str) -> Option<Type> {
    for stmt in stmts {
        match stmt {
            HirStmt::Let { name, ty, .. } if name == binding => return Some(ty.clone()),
            HirStmt::NestedFunction { func, .. } => {
                if let Some(ty) = binding_type_in_stmts(&func.body, binding) {
                    return Some(ty);
                }
            }
            _ => {}
        }
    }
    None
}

#[test]
fn nested_capture_propagates_call_result_list_refinement() {
    let source = "def solve() -> list[str]:\n    called = []\n    def add(value: str):\n        called.append(\"\".join([value]))\n    add(\"x\")\n    return called\n";
    assert_eq!(
        list_binding_type(source, "solve", "called"),
        Type::List(Box::new(Type::Str))
    );
}

#[test]
fn multilevel_nested_capture_propagates_list_refinement() {
    let source = "def solve() -> list[str]:\n    called = []\n    def middle():\n        def add(value: str):\n            called.append(\"\".join([value]))\n        add(\"x\")\n    middle()\n    return called\n";
    assert_eq!(
        list_binding_type(source, "solve", "called"),
        Type::List(Box::new(Type::Str))
    );
}

#[test]
fn nested_same_named_list_keeps_outer_and_inner_types_independent() {
    let source = "def solve() -> list[int]:\n    values = []\n    def add() -> int:\n        values = []\n        values.append(\"inner\")\n        return len(values)\n    values.append(1)\n    assert add() == 1\n    return values\n";
    let module = lower_source(source).expect("source should lower");
    let function = module
        .functions
        .iter()
        .find(|function| function.name == "solve")
        .expect("solve should lower");
    let outer = function
        .body
        .iter()
        .find_map(|stmt| match stmt {
            HirStmt::Let { name, ty, .. } if name == "values" => Some(ty.clone()),
            _ => None,
        })
        .expect("outer values should lower");
    let inner = function
        .body
        .iter()
        .find_map(|stmt| match stmt {
            HirStmt::NestedFunction { func, .. } if func.name == "add" => {
                binding_type_in_stmts(&func.body, "values")
            }
            _ => None,
        })
        .expect("inner values should lower");
    assert_eq!(outer, Type::List(Box::new(Type::Int)));
    assert_eq!(inner, Type::List(Box::new(Type::Str)));
}
