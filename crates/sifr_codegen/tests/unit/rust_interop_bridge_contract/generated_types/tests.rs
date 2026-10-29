use super::is_generated_bridge_type_path;

#[test]
fn generated_bridge_type_paths_accept_only_owned_canonical_forms() {
    assert!(is_generated_bridge_type_path(
        "crate::__sifr_bridge::main::BytesViewBridge"
    ));
    assert!(is_generated_bridge_type_path(
        "crate::__sifr_bridge::BytesViewBridge"
    ));
    assert!(!is_generated_bridge_type_path(
        "Vec<crate::__sifr_bridge::main::BytesViewBridge>"
    ));
    assert!(!is_generated_bridge_type_path(
        "crate::__sifr_bridge::main::BytesViewBridgeAlias"
    ));
    assert!(!is_generated_bridge_type_path(
        "crate::__sifr_bridge::3main::BytesViewBridge"
    ));
}
