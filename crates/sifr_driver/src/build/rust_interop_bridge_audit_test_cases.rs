pub(super) const CONSTRUCTION_CASES: [(&str, &str); 12] = [
    (
        "super glob",
        r#"
        use tokio::runtime::Builder;
        mod consumer {
            use super::*;
            fn forbidden() { let _runtime = Builder::new_current_thread(); }
        }
        "#,
    ),
    (
        "crate glob",
        r#"
        use tokio::runtime::Builder;
        mod consumer {
            use crate::*;
            fn forbidden() { let _runtime = Builder::new_current_thread(); }
        }
        "#,
    ),
    (
        "named relative re-export",
        r#"
        use tokio::runtime::Builder;
        mod consumer {
            use super::Builder as B;
            fn forbidden() { let _runtime = B::new_current_thread(); }
        }
        "#,
    ),
    (
        "renamed runtime module through glob",
        r#"
        mod exports { pub use tokio::runtime as rt2; }
        mod consumer {
            use crate::exports::*;
            fn forbidden() { let _runtime = rt2::Builder::new_multi_thread(); }
        }
        "#,
    ),
    (
        "renamed crate through glob",
        r#"
        mod exports { pub use tokio as t; }
        mod consumer {
            use crate::exports::*;
            fn forbidden() { let _runtime = t::runtime::Runtime::new(); }
        }
        "#,
    ),
    (
        "type alias through glob",
        r#"
        mod exports { pub type Rt = tokio::runtime::Builder; }
        mod consumer {
            use crate::exports::*;
            fn forbidden() { let _runtime = Rt::new_multi_thread(); }
        }
        "#,
    ),
    (
        "uniform-path runtime module",
        r#"
        mod exports { pub use tokio::runtime; }
        use exports::runtime;
        fn forbidden() { let _runtime = runtime::Builder::new_multi_thread(); }
        "#,
    ),
    (
        "uniform-path renamed type",
        r#"
        mod exports { pub use tokio::runtime::Builder as Rt; }
        use exports::Rt;
        fn forbidden() { let _runtime = Rt::new_multi_thread(); }
        "#,
    ),
    (
        "crate-path renamed runtime module",
        r#"
        mod exports { pub use tokio::runtime as rt2; }
        use crate::exports::rt2;
        fn forbidden() { let _runtime = rt2::Builder::new_multi_thread(); }
        "#,
    ),
    (
        "qualified Tokio Builder",
        r#"
        fn forbidden() {
            let _runtime = <tokio::runtime::Builder>::new_current_thread();
        }
        "#,
    ),
    (
        "qualified Tokio Runtime",
        r#"
        fn forbidden() {
            let _runtime = <tokio::runtime::Runtime>::new();
        }
        "#,
    ),
    (
        "qualified runtime type alias",
        r#"
        type Rt = tokio::runtime::Builder;
        fn forbidden() {
            let _runtime = <Rt>::new_multi_thread();
        }
        "#,
    ),
];

pub(super) const BLOCKING_CASES: [(&str, &str); 3] = [
    (
        "renamed task module through glob",
        r#"
        mod exports { pub use tokio::task as tk; }
        mod consumer {
            use crate::exports::*;
            fn forbidden() { tk::block_in_place(|| {}); }
        }
        "#,
    ),
    (
        "bare blocking function through glob",
        r#"
        mod exports { pub use tokio::task::block_in_place; }
        mod consumer {
            use crate::exports::*;
            fn forbidden() { block_in_place(|| {}); }
        }
        "#,
    ),
    (
        "uniform-path blocking function",
        r#"
        mod exports { pub use tokio::task::block_in_place; }
        use exports::block_in_place;
        fn forbidden() { block_in_place(|| {}); }
        "#,
    ),
];

pub(super) const NO_VIOLATION_CASES: [(&str, &str); 3] = [
    (
        "explicit non-Tokio Builder shadows unresolved glob",
        r#"
        mod prelude;
        use crate::prelude::*;
        use bindgen::Builder;
        fn safe() { let _builder = Builder::new(); }
        "#,
    ),
    (
        "fully qualified thread Builder with unresolved glob",
        r#"
        mod prelude;
        use crate::prelude::*;
        fn safe() { let _builder = std::thread::Builder::new(); }
        "#,
    ),
    (
        "qualified non-Tokio Builder",
        r#"
        fn safe() {
            let _builder = <std::thread::Builder>::new();
        }
        "#,
    ),
];
