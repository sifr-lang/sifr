use super::rewrite_named_project_borrows;

#[test]
fn project_value_borrows_rewrite_imported_and_local_calls() {
    let mut helper = r#"
            pub trait Unwrappable { fn unwrap(&self) -> i64; }
            pub struct Carrier(i64);
            impl Unwrappable for Carrier { fn unwrap(&self) -> i64 { self.0 } }
            pub fn through(value: Box<dyn Unwrappable>) -> i64 { value.unwrap() }
            fn optional_read(value: &Option<String>) -> usize {
                value.as_ref().map_or(0, String::len)
            }
            pub fn optional(value: Option<String>) -> usize { optional_read(&value) }
            pub fn outer(value: Option<String>) -> usize { optional(value) }
            pub fn local() -> i64 { through(Box::new(Carrier(2))) }
        "#
    .to_string();
    let mut main = r#"
            use crate::helper::{through as imported_through, optional, outer};
            fn main() {
                assert_eq!(imported_through(Box::new(helper::Carrier(3))), 3);
                assert_eq!(optional(Some("abc".to_string())), 3);
                assert_eq!(outer(Some("abcd".to_string())), 4);
                assert_eq!(helper::local(), 2);
            }
        "#
    .to_string();

    rewrite_named_project_borrows(&mut [("", &mut main), ("helper", &mut helper)])
        .expect("project borrow rewrite");

    let main_compact = main
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>();
    let helper_compact = helper
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>();
    assert!(
        main_compact.contains("imported_through(&helper::Carrier(3))"),
        "{main}"
    );
    assert!(main_compact.contains("optional(&Some("), "{main}");
    assert!(main_compact.contains("outer(&Some("), "{main}");
    assert!(helper_compact.contains("through(&Carrier(2))"), "{helper}");
    assert!(
        helper_compact.contains("pubfnthrough(value:&dynUnwrappable)"),
        "{helper}"
    );
    assert!(
        helper_compact.contains("pubfnoptional(value:&Option<String>)"),
        "{helper}"
    );
    assert!(
        helper_compact.contains("pubfnouter(value:&Option<String>)"),
        "{helper}"
    );
    assert!(helper_compact.contains("optional(value)"), "{helper}");
    assert!(
        !main_compact.contains("imported_through(Box::new("),
        "{main}"
    );
}
