// src/main.rs
use ::sifr_runtime::SifrInt;
fn collect_value_groups(items: &[SifrInt], limit: SifrInt) -> Vec<Vec<SifrInt>> {
    fn dfs(
        i: SifrInt,
        cur: &mut Vec<SifrInt>,
        total: SifrInt,
        items: &[SifrInt],
        limit: SifrInt,
        result: &mut Vec<Vec<SifrInt>>,
    ) {
        if &total == &limit {
            result.push(cur.clone());
            return;
        }
        if &i < &SifrInt::from_i64(0) || &i >= &SifrInt::from(items.len()) || &total > &limit {
            return;
        }
        let Some(sifr_generated_checked_value_0) = ({
            let sifr_generated_checked_read_collection = &items;
            let sifr_generated_checked_read_index = i.clone();
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        }) else {
            return;
        };
        cur.push(sifr_generated_checked_value_0.clone());
        dfs(
            i.clone(),
            cur,
            &total + &sifr_generated_checked_value_0.clone(),
            items,
            limit.clone(),
            result,
        );
        cur.pop();
        dfs(
            &i + &SifrInt::from_i64(1),
            cur,
            total.clone(),
            items,
            limit.clone(),
            result,
        );
    }
    let mut result: Vec<Vec<SifrInt>> = Vec::new();
    dfs(
        SifrInt::from_i64(0),
        &mut Vec::new(),
        SifrInt::from_i64(0),
        items,
        limit.clone(),
        &mut result,
    );
    result
}
fn collect_prefixes(nums: &[SifrInt]) -> Vec<Vec<SifrInt>> {
    fn dfs(
        i: SifrInt,
        nums: &[SifrInt],
        result: &mut Vec<Vec<SifrInt>>,
        subset: &mut Vec<SifrInt>,
    ) {
        if &i < &SifrInt::from_i64(0) || &i >= &SifrInt::from(nums.len()) {
            result.push(subset.clone());
            return;
        }
        let Some(sifr_generated_checked_value_2) = ({
            let sifr_generated_checked_read_collection = &nums;
            let sifr_generated_checked_read_index = i.clone();
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        }) else {
            result.push(subset.clone());
            return;
        };
        dfs(&i + &SifrInt::from_i64(1), nums, result, subset);
        subset.push(sifr_generated_checked_value_2.clone());
        dfs(&i + &SifrInt::from_i64(1), nums, result, subset);
        subset.pop();
    }
    let mut result: Vec<Vec<SifrInt>> = Vec::new();
    let mut subset: Vec<SifrInt> = Vec::new();
    dfs(SifrInt::from_i64(0), nums, &mut result, &mut subset);
    result
}
fn main() {
    assert_eq!(
        format!(
            "{:?}",
            collect_value_groups(
                &vec![
                    SifrInt::from_i64(1),
                    SifrInt::from_i64(2),
                    SifrInt::from_i64(4)
                ],
                SifrInt::from_i64(4)
            )
        ),
        "[[1, 1, 1, 1], [1, 1, 2], [2, 2], [4]]"
    );
    assert_eq!(
        format!(
            "{:?}",
            collect_prefixes(&vec![
                SifrInt::from_i64(1),
                SifrInt::from_i64(2),
                SifrInt::from_i64(3)
            ])
        ),
        "[[], [3], [2], [2, 3], [1], [1, 3], [1, 2], [1, 2, 3]]"
    );
}
