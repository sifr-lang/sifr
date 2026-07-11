use sifr_runtime::interop::SifrIntBridge;

#[must_use]
pub const fn feature_name() -> &'static str {
    "collections"
}

#[must_use]
pub fn new_set() -> Vec<SifrIntBridge> {
    Vec::new()
}

#[must_use]
pub fn set_from_list(items: Vec<SifrIntBridge>) -> Vec<SifrIntBridge> {
    let mut items = bridge_vec_to_i64(items);
    items.sort_unstable();
    items.dedup();
    int_vec_to_bridge(items)
}

#[must_use]
pub fn set_add(s: Vec<SifrIntBridge>, item: SifrIntBridge) -> Vec<SifrIntBridge> {
    let mut s = bridge_vec_to_i64(s);
    let item = item.to_i64_saturating();
    if !s.contains(&item) {
        s.push(item);
    }
    int_vec_to_bridge(s)
}

#[must_use]
pub fn set_contains(s: &[SifrIntBridge], item: SifrIntBridge) -> bool {
    let item = item.to_i64_saturating();
    s.iter()
        .any(|candidate| candidate.to_i64_saturating() == item)
}

#[must_use]
pub fn set_remove(s: Vec<SifrIntBridge>, item: SifrIntBridge) -> Vec<SifrIntBridge> {
    let mut s = bridge_vec_to_i64(s);
    let item = item.to_i64_saturating();
    s.retain(|candidate| *candidate != item);
    int_vec_to_bridge(s)
}

#[must_use]
pub fn set_len(s: &[SifrIntBridge]) -> SifrIntBridge {
    SifrIntBridge::from(s.len() as i64)
}

#[must_use]
pub fn set_union(left: Vec<SifrIntBridge>, right: Vec<SifrIntBridge>) -> Vec<SifrIntBridge> {
    let mut left = bridge_vec_to_i64(left);
    let right = bridge_vec_to_i64(right);
    for item in right {
        if !left.contains(&item) {
            left.push(item);
        }
    }
    left.sort_unstable();
    int_vec_to_bridge(left)
}

#[must_use]
pub fn set_intersection(left: Vec<SifrIntBridge>, right: Vec<SifrIntBridge>) -> Vec<SifrIntBridge> {
    let left = bridge_vec_to_i64(left);
    let right = bridge_vec_to_i64(right);
    int_vec_to_bridge(
        left.into_iter()
            .filter(|item| right.contains(item))
            .collect(),
    )
}

fn int_vec_to_bridge(values: Vec<i64>) -> Vec<SifrIntBridge> {
    values.into_iter().map(SifrIntBridge::from).collect()
}

fn bridge_vec_to_i64(values: Vec<SifrIntBridge>) -> Vec<i64> {
    values
        .into_iter()
        .map(|value| value.to_i64_saturating())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bridge_values(values: Vec<SifrIntBridge>) -> Vec<i64> {
        values
            .into_iter()
            .map(|value| value.to_i64_saturating())
            .collect()
    }

    #[test]
    fn set_helpers_match_legacy_list_backed_semantics() {
        assert!(new_set().is_empty());
        assert_eq!(
            bridge_values(set_from_list(int_vec_to_bridge(vec![3, 1, 3, 2]))),
            vec![1, 2, 3]
        );
        assert_eq!(
            bridge_values(set_add(
                int_vec_to_bridge(vec![1, 2]),
                SifrIntBridge::from(3)
            )),
            vec![1, 2, 3]
        );
        assert_eq!(
            bridge_values(set_add(
                int_vec_to_bridge(vec![1, 2]),
                SifrIntBridge::from(2)
            )),
            vec![1, 2]
        );
        assert!(set_contains(
            &int_vec_to_bridge(vec![1, 2, 3]),
            SifrIntBridge::from(2)
        ));
        assert!(!set_contains(
            &int_vec_to_bridge(vec![1, 2, 3]),
            SifrIntBridge::from(9)
        ));
        assert_eq!(
            bridge_values(set_remove(
                int_vec_to_bridge(vec![1, 2, 2, 3]),
                SifrIntBridge::from(2)
            )),
            vec![1, 3]
        );
        assert_eq!(
            set_len(&int_vec_to_bridge(vec![1, 2, 3])).to_i64_saturating(),
            3
        );
        assert_eq!(
            bridge_values(set_union(
                int_vec_to_bridge(vec![3, 1, 1]),
                int_vec_to_bridge(vec![2, 3, 4, 4])
            )),
            vec![1, 1, 2, 3, 4]
        );
        assert_eq!(
            bridge_values(set_intersection(
                int_vec_to_bridge(vec![1, 2, 2, 3]),
                int_vec_to_bridge(vec![2, 4])
            )),
            vec![2, 2]
        );
    }
}
