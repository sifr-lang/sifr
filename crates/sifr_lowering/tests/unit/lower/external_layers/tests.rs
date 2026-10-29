
use super::*;
#[test]
fn source_baseline_clones_share_values_and_isolate_project_mutation() {
    let mut source = ModuleMap::default();
    source.insert("sifr.math".to_owned(), vec!["sqrt".to_owned()]);
    source.freeze();
    let mut first = source.clone();
    let mut second = source.clone();
    assert!(std::ptr::eq(
        source.get("sifr.math").unwrap(),
        first.get("sifr.math").unwrap()
    ));
    first.insert("app".to_owned(), vec!["first".to_owned()]);
    second.insert("app".to_owned(), vec!["second".to_owned()]);
    assert_eq!(first.get("app").unwrap(), &["first"]);
    assert_eq!(second.get("app").unwrap(), &["second"]);
    assert!(!source.contains_key("app"));
    first.remove("app");
    assert!(!first.contains_key("app"));
    assert!(second.contains_key("app"));
}
#[test]
fn revision_updates_do_not_mutate_older_snapshots() {
    let mut defs = ModuleMap::default();
    defs.insert("user".to_owned(), vec![1]);
    let prior = defs.clone();
    defs.entry("user".to_owned()).or_default().push(2);
    assert_eq!(prior.get("user"), Some(&vec![1]));
    assert_eq!(defs.get("user"), Some(&vec![1, 2]));
}
