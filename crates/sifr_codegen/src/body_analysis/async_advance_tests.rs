use super::{HirExpr, Type, collect_expr_mutation};
use std::collections::{HashMap, HashSet};

#[test]
fn async_advance_mutates_only_the_canonical_builtin_argument() {
    for (function, arity, expected) in [
        ("anext", 1, true),
        ("user.anext", 1, false),
        ("anext", 2, false),
    ] {
        let argument = HirExpr::Name {
            name: "stream".to_string(),
            binding_id: None,
            ty: Type::None,
        };
        let call = HirExpr::Call {
            func: function.to_string(),
            args: vec![argument; arity],
            mutable_arg_places: Vec::new(),
            ty: Type::None,
        };
        let mut mutations = HashSet::new();
        collect_expr_mutation(&call, &HashMap::new(), &mut mutations);
        assert_eq!(mutations.contains("stream"), expected, "{function}/{arity}");
    }
}
