// src/main.rs
use ::std::collections::HashMap;

use ::sifr_runtime::SifrInt;

fn guarded_lookup(table: &HashMap<SifrInt, SifrInt>, key: SifrInt) -> SifrInt {
    let Some(__sifr_checked_value_0) = table.get(&key) else {
        return -&SifrInt::from_i64(1);
    };
    let value: SifrInt = (*__sifr_checked_value_0).clone();
    value.clone()
}

fn expression_lookup(table: &HashMap<SifrInt, SifrInt>, base: SifrInt) -> SifrInt {
    compile_error!("structured statement emission missing for production path: If { condition: ContainsOp { element: BinOp { left: Name { name: \"base\", binding_id: Some(BindingId(4)), ty: Int }, op: \"+\", right: IntLiteral(1), ty: Int }, collection: MethodCall { object: Name { name: \"table\", binding_id: Some(BindingId(3)), ty: Dict(Int, Int) }, method: \"keys\", args: [], receiver_convention: Some(SharedBorrow), receiver_target: None, mutable_arg_places: [], source: Some(MethodCallSource { call_range: 378..390, receiver_range: 378..383, arg_ranges: [] }), ty: List(Int) }, ty: Bool }, then_body: [Let { name: \"value\", ty: Int, value: Index { object: Name { name: \"table\", binding_id: Some(BindingId(3)), ty: Dict(Int, Int) }, index: BinOp { left: Name { name: \"base\", binding_id: Some(BindingId(4)), ty: Int }, op: \"+\", right: IntLiteral(1), ty: Int }, ty: Int }, is_mutable: true }, Return { value: Some(Name { name: \"value\", binding_id: Some(BindingId(5)), ty: Int }) }], elif_clauses: [], else_body: None }");
    -&SifrInt::from_i64(1)
}

fn sum_known_keys(table: &HashMap<SifrInt, SifrInt>, keys: &Vec<SifrInt>) -> SifrInt {
    let mut total: SifrInt = SifrInt::from_i64(0);
    for key in keys.iter().cloned() {
        if let Some(__sifr_checked_value_1) = table.get(&key) {
            total = &total + &(*__sifr_checked_value_1).clone();
        }
    }
    total.clone()
}

fn main() {
    let t: HashMap<SifrInt, SifrInt> = HashMap::from([(SifrInt::from_i64(1), SifrInt::from_i64(10)), (SifrInt::from_i64(2), SifrInt::from_i64(20)), (SifrInt::from_i64(4), SifrInt::from_i64(40))]);
    assert!((&guarded_lookup(&t, SifrInt::from_i64(2)) == &SifrInt::from_i64(20)));
    assert!((&guarded_lookup(&t, SifrInt::from_i64(3)) == &-(SifrInt::from_i64(1))));
    assert!((&expression_lookup(&t, SifrInt::from_i64(1)) == &SifrInt::from_i64(20)));
    assert!((&expression_lookup(&t, SifrInt::from_i64(2)) == &-(SifrInt::from_i64(1))));
    assert!((&sum_known_keys(&t, &vec![SifrInt::from_i64(0), SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(5)]) == &SifrInt::from_i64(30)));
}
