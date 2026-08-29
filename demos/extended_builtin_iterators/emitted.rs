// src/main.rs
use ::sifr_runtime::SifrInt;

fn add(x: SifrInt, y: SifrInt) -> SifrInt {
    &x + &y
}

fn main() {
    let mut rev_it: Box<dyn Iterator<Item = SifrInt>> = Box::new((vec![SifrInt::from_i64(9), SifrInt::from_i64(7), SifrInt::from_i64(5)]).into_iter().rev());
    assert!((rev_it.next() == Some(SifrInt::from_i64(5))));
    assert!((format!("{:?}", rev_it.collect::<Vec<_>>()) == "[7, 9]"));
    let enum_it: Box<dyn Iterator<Item = (SifrInt, String)>> = Box::new((vec!["a".to_string(), "b".to_string()]).into_iter().enumerate().map(|__pair| (SifrInt::from(__pair.0) + SifrInt::from_i64(3), __pair.1)));
    assert!((format!("{:?}", enum_it.collect::<Vec<_>>()) == "[(3, \"a\"), (4, \"b\")]"));
    let zip_it: Box<dyn Iterator<Item = (SifrInt, String)>> = Box::new((vec![SifrInt::from_i64(1), SifrInt::from_i64(2)]).into_iter().zip((vec!["x".to_string(), "y".to_string()]).into_iter()).map(|__zip_item| (__zip_item.0, __zip_item.1)));
    assert!((format!("{:?}", zip_it.collect::<Vec<_>>()) == "[(1, \"x\"), (2, \"y\")]"));
    let mut mapped_it: Box<dyn Iterator<Item = SifrInt>> = Box::new((vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3)]).into_iter().zip((vec![SifrInt::from_i64(4), SifrInt::from_i64(5), SifrInt::from_i64(6)]).into_iter()).map(|__map_item| {
    let __map_arg_0 = __map_item.0;
    let __map_arg_1 = __map_item.1;
    add(__map_arg_0, __map_arg_1)
}).into_iter());
    assert!((mapped_it.next() == Some(SifrInt::from_i64(5))));
    assert!((format!("{:?}", mapped_it.collect::<Vec<_>>()) == "[7, 9]"));
    assert!((format!("{}", "parity_ext_extended_builtin_iterators_iterator_reclosure_demo: ok") == "parity_ext_extended_builtin_iterators_iterator_reclosure_demo: ok"));
}
