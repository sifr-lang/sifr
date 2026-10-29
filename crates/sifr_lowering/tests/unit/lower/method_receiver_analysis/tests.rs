
use super::*;

#[test]
fn fixed_trait_receiver_registry_is_explicit() {
    assert_eq!(
        fixed_trait_receiver_convention("__eq__"),
        Some(ReceiverConvention::SharedBorrow)
    );
    assert_eq!(
        fixed_trait_receiver_convention("__add__"),
        Some(ReceiverConvention::Owned)
    );
    assert_eq!(fixed_trait_receiver_convention("bump"), None);
}
