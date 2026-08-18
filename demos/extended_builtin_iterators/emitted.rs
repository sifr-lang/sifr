// src/main.rs
fn add(x: i64, y: i64) -> i64 {
    x + y
}

fn main() {
    let mut rev_it: Box<dyn Iterator<Item = i64>> = Box::new((vec![9_i64, 7_i64, 5_i64]).into_iter().rev());
    assert!((rev_it.next() == Some(5_i64)));
    assert!((format!("{:?}", rev_it.collect::<Vec<_>>()) == "[7, 9]"));
    let enum_it: Box<dyn Iterator<Item = (i64, String)>> = Box::new((vec!["a".to_string(), "b".to_string()]).into_iter().enumerate().map(|__pair| ((__pair.0 as i64) + (3_i64), __pair.1)));
    assert!((format!("{:?}", enum_it.collect::<Vec<_>>()) == "[(3, \"a\"), (4, \"b\")]"));
    let zip_it: Box<dyn Iterator<Item = (i64, String)>> = Box::new((vec![1_i64, 2_i64]).into_iter().zip((vec!["x".to_string(), "y".to_string()]).into_iter()).map(|__zip_item| (__zip_item.0, __zip_item.1)));
    assert!((format!("{:?}", zip_it.collect::<Vec<_>>()) == "[(1, \"x\"), (2, \"y\")]"));
    let mut mapped_it: Box<dyn Iterator<Item = i64>> = Box::new((vec![1_i64, 2_i64, 3_i64]).into_iter().zip((vec![4_i64, 5_i64, 6_i64]).into_iter()).map(|__map_item| {
    let __map_arg_0 = __map_item.0;
    let __map_arg_1 = __map_item.1;
    add(__map_arg_0, __map_arg_1)
}).into_iter());
    assert!((mapped_it.next() == Some(5_i64)));
    assert!((format!("{:?}", mapped_it.collect::<Vec<_>>()) == "[7, 9]"));
    assert!((format!("{}", "parity_ext_extended_builtin_iterators_iterator_reclosure_demo: ok") == "parity_ext_extended_builtin_iterators_iterator_reclosure_demo: ok"));
}
