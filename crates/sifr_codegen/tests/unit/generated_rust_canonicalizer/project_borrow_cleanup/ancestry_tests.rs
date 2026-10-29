use super::rewrite_named_project_borrows;

#[test]
fn ancestry_scalar_borrows_reach_callers_through_reexports() {
    for reverse in [false, true] {
        let mut sources = vec![
                ("base", "pub struct Root; impl Root { pub fn new(value: SifrInt) -> SifrInt { value } }".to_string()),
                ("middle", "use crate::base::Root as BaseRoot; pub struct Root; impl Root { pub fn new(value: SifrInt, middle: SifrInt) -> (SifrInt, SifrInt) { (BaseRoot::new(value), middle.clone()) } }".to_string()),
                ("child", "use crate::middle::Root as MiddleRoot; pub struct Child; impl Child { pub fn new(value: SifrInt, middle: SifrInt, extra: SifrInt) -> ((SifrInt, SifrInt), SifrInt) { (MiddleRoot::new(value, middle), extra.clone()) } }".to_string()),
                ("facade", "pub use crate::child::Child as PublicChild;".to_string()),
                ("", "use crate::facade::PublicChild as Child; fn main() { Child::new(SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3)); }".to_string()),
            ];
        if reverse {
            sources.reverse();
        }
        let mut named = sources
            .iter_mut()
            .map(|(name, source)| (*name, source))
            .collect::<Vec<_>>();
        rewrite_named_project_borrows(&mut named).expect("ancestry borrow closure");
        let first = named
            .iter()
            .map(|(_, source)| (**source).clone())
            .collect::<Vec<_>>();
        rewrite_named_project_borrows(&mut named).expect("stable project borrow closure");
        assert_eq!(
            first,
            named
                .iter()
                .map(|(_, source)| (**source).clone())
                .collect::<Vec<_>>(),
            "one invocation must close the complete call graph"
        );
        let main = sources
            .iter()
            .find(|(name, _)| name.is_empty())
            .unwrap()
            .1
            .split_whitespace()
            .collect::<String>();
        assert!(
            main.contains(
                "Child::new(SifrInt::from_i64(1),&SifrInt::from_i64(2),&SifrInt::from_i64(3)"
            ),
            "{main}"
        );
        let base = &sources.iter().find(|(name, _)| *name == "base").unwrap().1;
        assert!(
            base.contains("new(value: SifrInt)"),
            "consumed integer must remain owned: {base}"
        );
    }
}
