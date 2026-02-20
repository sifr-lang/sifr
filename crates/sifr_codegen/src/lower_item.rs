//! Item lowering scaffolds for the IR migration.

use crate::{CodegenError, RustItem};

pub fn lower_item_raw(raw: &str) -> Result<Vec<RustItem>, CodegenError> {
    Ok(vec![RustItem::RawCode(raw.to_string())])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowers_raw_item_placeholder() {
        let items = lower_item_raw("fn helper() {}").expect("placeholder lower should succeed");
        assert_eq!(items.len(), 1);
        assert!(matches!(items[0], RustItem::RawCode(_)));
    }
}
