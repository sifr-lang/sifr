use super::*;

#[test]
fn zero_argument_constructor_implements_default_by_delegation() {
    let rust_code = generate_rust_from_source(
        r#"class Marker:
    pass

class Explicit:
    def __init__(self) -> None:
        pass
"#,
    );

    for class_name in ["Marker", "Explicit"] {
        assert!(
            rust_code.contains(&format!("impl ::std::default::Default for {class_name} {{")),
            "{rust_code}"
        );
        assert!(
            rust_code.contains("fn default() -> Self {\n        Self::new()"),
            "{rust_code}"
        );
    }
    assert_eq!(rust_code.matches("fn new() -> Self {").count(), 2);
    assert_eq!(rust_code.matches("fn default() -> Self {").count(), 2);
}

#[test]
fn sync_channel_runtime_exposes_clone_only_through_clone_trait() {
    let runtime = crate::lib_runtime_needs::sync_channel_runtime_rust_code();

    assert!(runtime.contains("impl<T: Clone> Clone for Channel<T>"));
    assert!(runtime.contains("impl<T: Clone> Clone for ChannelSender<T>"));
    assert!(!runtime.contains("fn clone(&self) -> Channel<T>"));
    assert!(!runtime.contains("fn clone(&self) -> ChannelSender<T>"));
}
