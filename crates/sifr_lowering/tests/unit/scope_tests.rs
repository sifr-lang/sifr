use super::*;

#[test]
fn test_define_and_lookup() {
    let mut scope = Scope::new();
    scope.define("x".to_string(), Type::Int);
    let info = scope.lookup("x").unwrap();
    assert_eq!(info.ty, Type::Int);
    assert!(!info.is_moved);
}

#[test]
fn test_nested_scopes() {
    let mut scope = Scope::new();
    scope.define("x".to_string(), Type::Int);
    scope.push();
    scope.define("y".to_string(), Type::Str);
    assert!(scope.lookup("x").is_some());
    assert!(scope.lookup("y").is_some());
    scope.pop();
    assert!(scope.lookup("x").is_some());
    assert!(scope.lookup("y").is_none());
}

#[test]
fn binding_ids_distinguish_shadowed_names_and_outlive_frames() {
    let mut scope = Scope::new();
    scope.define("value".to_string(), Type::Int);
    let outer = scope.lookup("value").unwrap().binding_id;

    scope.push();
    scope.define("value".to_string(), Type::Str);
    let inner = scope.lookup("value").unwrap().binding_id;
    assert_ne!(outer, inner);
    scope.pop();

    assert_eq!(scope.lookup("value").unwrap().binding_id, outer);
    assert_eq!(scope.retained_binding(inner).unwrap().name, "value");
    assert_eq!(
        scope.retained_binding(inner).unwrap().binding_kind,
        BindingKind::Local
    );
}

#[test]
fn module_constants_have_distinct_immutable_binding_facts() {
    let mut scope = Scope::new();
    scope.define_module_constant("VALUES".to_string(), Type::List(Box::new(Type::Int)));

    let info = scope.lookup("VALUES").unwrap();
    assert_eq!(info.binding_kind, BindingKind::ModuleConstant);
    assert_eq!(info.mutability, BindingMutability::Immutable);
    assert_eq!(
        scope
            .retained_binding(info.binding_id)
            .unwrap()
            .binding_kind,
        BindingKind::ModuleConstant
    );
}

#[test]
fn retained_receiver_and_ephemeral_facts_keep_final_conventions() {
    let mut scope = Scope::new();
    scope.push();
    let receiver = scope.define_receiver(
        "self".to_string(),
        Type::Any,
        ReceiverConvention::SharedBorrow,
    );
    scope.define_ephemeral("item".to_string(), Type::Int, EphemeralOrigin::Iteration);
    let item = scope.lookup("item").unwrap().binding_id;
    scope.pop();

    scope.patch_receiver_convention(receiver, ReceiverConvention::MutableBorrow);
    let receiver_fact = scope.retained_binding(receiver).unwrap();
    assert_eq!(receiver_fact.binding_kind, BindingKind::Receiver);
    assert_eq!(
        receiver_fact.receiver_convention,
        Some(ReceiverConvention::MutableBorrow)
    );
    assert_eq!(receiver_fact.mutability, BindingMutability::Mutable);
    assert_eq!(
        scope.retained_binding(item).unwrap().binding_kind,
        BindingKind::EphemeralLocal(EphemeralOrigin::Iteration)
    );
}

#[test]
fn test_move_tracking() {
    let mut scope = Scope::new();
    scope.define("s".to_string(), Type::Str);
    assert!(!scope.is_moved("s"));
    scope.mark_moved("s");
    assert!(scope.is_moved("s"));
}

#[test]
fn test_copy_types_not_moved() {
    let mut scope = Scope::new();
    scope.define("x".to_string(), Type::Int);
    let moved = scope.mark_moved("x");
    assert!(!moved);
    assert!(!scope.is_moved("x"));
}

#[test]
fn test_specialized_copy_binding_can_be_marked_moved() {
    let mut scope = Scope::new();
    scope.define("handler".to_string(), Type::Int);

    assert!(scope.mark_binding_moved("handler"));
    assert!(scope.is_moved("handler"));
    scope.reset_moved("handler");
    assert!(scope.mark_moved("handler"));
    assert!(scope.is_moved("handler"));
}

#[test]
fn rebound_function_binding_becomes_an_ordinary_local() {
    let mut scope = Scope::new();
    scope.define_function("callback".to_string(), Type::Int);

    assert!(
        scope
            .lookup("callback")
            .is_some_and(VarInfo::is_function_binding)
    );
    scope.mark_rebound_local("callback");
    assert!(
        !scope
            .lookup("callback")
            .is_some_and(VarInfo::is_function_binding)
    );
    assert_eq!(
        scope.lookup("callback").map(|info| info.binding_kind),
        Some(BindingKind::Local)
    );
}

#[test]
fn test_narrowing() {
    let mut scope = Scope::new();
    let union_type = Type::Union(vec![Type::Int, Type::Str]);
    scope.define("x".to_string(), union_type.clone());

    assert_eq!(scope.effective_type("x"), Some(&union_type));
    scope.narrow_var("x", Type::Int);
    assert_eq!(scope.effective_type("x"), Some(&Type::Int));
    scope.clear_narrowing("x");
    assert_eq!(scope.effective_type("x"), Some(&union_type));
}

#[test]
fn test_narrowing_save_restore() {
    let mut scope = Scope::new();
    let union_type = Type::Union(vec![Type::Int, Type::Str]);
    scope.define("x".to_string(), union_type.clone());

    let snapshot = scope.save_narrowing_state();
    scope.narrow_var("x", Type::Int);
    assert_eq!(scope.effective_type("x"), Some(&Type::Int));
    scope.restore_narrowing_state(&snapshot);
    assert_eq!(scope.effective_type("x"), Some(&union_type));
}

#[test]
fn test_type_alias() {
    let mut scope = Scope::new();
    scope.define_type_alias("UserId".to_string(), Type::Int);
    assert_eq!(scope.lookup_type_alias("UserId"), Some(&Type::Int));
    assert_eq!(scope.lookup_type_alias("Unknown"), None);
}

#[test]
fn test_save_restore_moved_state() {
    let mut scope = Scope::new();
    scope.define("s".to_string(), Type::Str);
    assert!(!scope.is_moved("s"));

    let snapshot = scope.save_moved_state();
    scope.mark_moved("s");
    assert!(scope.is_moved("s"));
    scope.restore_moved_state(&snapshot);
    assert!(!scope.is_moved("s"));
}

#[test]
fn test_moved_since() {
    let mut scope = Scope::new();
    scope.define("s".to_string(), Type::Str);
    scope.define("x".to_string(), Type::Int);

    let snapshot = scope.save_moved_state();
    scope.mark_moved("s");
    scope.mark_moved("x");

    let newly = scope.moved_since(&snapshot);
    assert_eq!(newly, vec!["s".to_string()]);
}
