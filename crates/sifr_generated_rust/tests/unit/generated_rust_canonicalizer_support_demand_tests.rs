use super::canonicalize_generated_rust_source;

#[test]
fn retains_exact_support_demand_from_a_sibling_generated_module() {
    let source = r#"
        mod generated_support {
            use crate::{NominalKind, SharedError};

            pub(crate) enum SecondaryKind {
                Message,
                Unused,
            }
            pub(crate) trait PatternMethods {
                fn search(&self) -> bool;
                fn full_contract(&self) -> bool;
            }
            pub(crate) fn bridge() -> Result<i64, SharedError> {
                if matches!(NominalKind::Value, NominalKind::Value) {
                    Ok(transitive_helper())
                } else {
                    Ok(0)
                }
            }
            fn transitive_helper() -> i64 {
                1
            }
            fn unused_support() -> i64 {
                2
            }
        }
        mod generated_nominals {
            use crate::generated_support::*;

            pub struct Child;
            pub struct SharedError;
            pub enum NominalKind {
                Value,
                UnusedNominal,
            }
            impl Child {
                pub fn wait(&self) -> Result<i64, SharedError> {
                    bridge()
                }
                pub fn has_message(&self) -> bool {
                    matches!(SecondaryKind::Message, SecondaryKind::Message)
                }
            }
            impl PatternMethods for Child {
                fn search(&self) -> bool {
                    true
                }
                fn full_contract(&self) -> bool {
                    true
                }
            }
        }
        pub use generated_nominals::{Child, NominalKind, SharedError};

        fn main() {
            let child = Child;
            println!("{:?}", child.wait().ok());
            println!("{}", child.has_message());
            println!("{}", child.search());
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("sibling support demand should canonicalize");

    assert!(canonical.contains("fn bridge()"), "{canonical}");
    assert!(canonical.contains("fn transitive_helper()"), "{canonical}");
    assert!(canonical.contains("struct SharedError"), "{canonical}");
    assert!(canonical.contains("Message"), "{canonical}");
    assert!(!canonical.contains("Unused"), "{canonical}");
    assert!(canonical.contains("Value"), "{canonical}");
    assert!(!canonical.contains("UnusedNominal"), "{canonical}");
    assert!(canonical.contains("fn search(&self)"), "{canonical}");
    assert!(canonical.contains("fn full_contract(&self)"), "{canonical}");
    assert!(!canonical.contains("fn unused_support()"), "{canonical}");
}
