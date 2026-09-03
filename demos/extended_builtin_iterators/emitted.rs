// src/main.rs
use ::sifr_runtime::SifrInt;
fn add(x: SifrInt, y: SifrInt) -> SifrInt {
    &x + &y
}
fn main() {
    let mut rev_it: Box<dyn Iterator<Item = SifrInt>> = Box::new(
        vec![
            SifrInt::from_i64(9),
            SifrInt::from_i64(7),
            SifrInt::from_i64(5),
        ]
        .into_iter()
        .rev(),
    );
    assert_eq!(rev_it.next(), Some(SifrInt::from_i64(5)));
    assert_eq!(format!("{:?}", rev_it.collect::<Vec<_>>()), "[7, 9]");
    let enum_it: Box<dyn Iterator<Item = (SifrInt, String)>> = Box::new(
        vec!["a".to_string(), "b".to_string()]
            .into_iter()
            .enumerate()
            .map(|sifr_generated_pair| {
                (
                    SifrInt::from(sifr_generated_pair.0) + SifrInt::from_i64(3),
                    sifr_generated_pair.1,
                )
            }),
    );
    assert_eq!(
        format!("{:?}", enum_it.collect::<Vec<_>>()),
        "[(3, \"a\"), (4, \"b\")]"
    );
    let zip_it: Box<dyn Iterator<Item = (SifrInt, String)>> = Box::new(
        vec![SifrInt::from_i64(1), SifrInt::from_i64(2)]
            .into_iter()
            .zip(vec!["x".to_string(), "y".to_string()])
            .map(|sifr_generated_zip_item| (sifr_generated_zip_item.0, sifr_generated_zip_item.1)),
    );
    assert_eq!(
        format!("{:?}", zip_it.collect::<Vec<_>>()),
        "[(1, \"x\"), (2, \"y\")]"
    );
    let mut mapped_it: Box<dyn Iterator<Item = SifrInt>> = Box::new(
        vec![
            SifrInt::from_i64(1),
            SifrInt::from_i64(2),
            SifrInt::from_i64(3),
        ]
        .into_iter()
        .zip(vec![
            SifrInt::from_i64(4),
            SifrInt::from_i64(5),
            SifrInt::from_i64(6),
        ])
        .map(|sifr_generated_map_item| {
            let sifr_generated_map_arg_0 = sifr_generated_map_item.0;
            let sifr_generated_map_arg_1 = sifr_generated_map_item.1;
            add(sifr_generated_map_arg_0, sifr_generated_map_arg_1)
        }),
    );
    assert_eq!(mapped_it.next(), Some(SifrInt::from_i64(5)));
    assert_eq!(format!("{:?}", mapped_it.collect::<Vec<_>>()), "[7, 9]");
    assert_eq!(
        "parity_ext_extended_builtin_iterators_iterator_reclosure_demo: ok".to_string(),
        "parity_ext_extended_builtin_iterators_iterator_reclosure_demo: ok"
    );
}
