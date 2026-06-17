fn add(x: i64, y: i64) -> i64 {
    return x + y;
}

fn main() {
    let mut rev_it: Box<dyn Iterator<Item = i64>> = Box::new((vec![9 as i64, 7 as i64, 5 as i64]).into_iter().rev());
    assert!(rev_it.next() == Some(5 as i64));
    assert!(format!("{:?}", rev_it.collect::<Vec<_>>()) == "[7, 9]".to_string());
    let mut enum_it: Box<dyn Iterator<Item = (i64, String)>> = Box::new((vec!["a".to_string(), "b".to_string()]).into_iter().enumerate().map(|__pair| ((__pair.0 as i64) + (3 as i64), __pair.1)));
    assert!(format!("{:?}", enum_it.collect::<Vec<_>>()) == "[(3, \"a\"), (4, \"b\")]".to_string());
    let mut zip_it: Box<dyn Iterator<Item = (i64, String)>> = Box::new((vec![1 as i64, 2 as i64]).into_iter().zip((vec!["x".to_string(), "y".to_string()]).into_iter()).map(|__zip_item| (__zip_item.0, __zip_item.1)));
    assert!(format!("{:?}", zip_it.collect::<Vec<_>>()) == "[(1, \"x\"), (2, \"y\")]".to_string());
    let mut mapped_it: Box<dyn Iterator<Item = i64>> = Box::new((vec![1 as i64, 2 as i64, 3 as i64]).into_iter().zip((vec![4 as i64, 5 as i64, 6 as i64]).into_iter()).map(|__map_item| {
    let __map_arg_0 = __map_item.0;
    let __map_arg_1 = __map_item.1;
    return add(__map_arg_0, __map_arg_1);
}).into_iter());
    assert!(mapped_it.next() == Some(5 as i64));
    assert!(format!("{:?}", mapped_it.collect::<Vec<_>>()) == "[7, 9]".to_string());
    assert!(format!("{}", "parity_ext_extended_builtin_iterators_iterator_reclosure_demo: ok".to_string()) == "parity_ext_extended_builtin_iterators_iterator_reclosure_demo: ok".to_string());
}
