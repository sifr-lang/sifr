// src/main.rs
use ::sifr_runtime::SifrInt;
fn apply_twice(f: impl Fn(SifrInt) -> SifrInt, value: SifrInt) -> SifrInt {
    f(f(value))
}
fn score(base: SifrInt) -> SifrInt {
    let offset: SifrInt = SifrInt::from_i64(3);
    let add_offset = |x: SifrInt| ::std::ops::Add::add(&x, &offset);
    let amplify = |x: SifrInt| ::std::ops::Mul::mul(&x, &SifrInt::from_i64(2));
    let adjusted: SifrInt = apply_twice(add_offset, base);
    amplify(adjusted)
}
fn accumulate(values: &[SifrInt]) -> SifrInt {
    let mut total: SifrInt = SifrInt::from_i64(0);
    let mut apply = || {
        for value in values.iter().cloned() {
            total += value;
        }
    };
    apply();
    total.clone()
}
fn collect_prefixes(nums: &[SifrInt]) -> Vec<Vec<SifrInt>> {
    fn dfs(i: &SifrInt, nums: &[SifrInt], res: &mut Vec<Vec<SifrInt>>, subset: &mut Vec<SifrInt>) {
        if i < SifrInt::from_i64(0) || i >= nums.len() {
            res.push(subset.clone());
            return;
        }
        let Some(sifr_generated_checked_value_0) = ({
            let sifr_generated_checked_read_collection = &nums;
            let sifr_generated_checked_read_index = i.clone();
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        }) else {
            res.push(subset.clone());
            return;
        };
        subset.push(sifr_generated_checked_value_0);
        dfs(
            &::std::ops::Add::add(i, &SifrInt::from_i64(1)),
            nums,
            res,
            subset,
        );
        subset.pop();
        dfs(
            &::std::ops::Add::add(i, &SifrInt::from_i64(1)),
            nums,
            res,
            subset,
        );
    }
    let mut res: Vec<Vec<SifrInt>> = Vec::new();
    let mut subset: Vec<SifrInt> = Vec::new();
    dfs(&SifrInt::from_i64(0), nums, &mut res, &mut subset);
    res
}
#[expect(
    clippy::needless_pass_by_value,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn collect_value_groups(items: &[SifrInt], limit: SifrInt) -> Vec<Vec<SifrInt>> {
    fn dfs(
        i: &SifrInt,
        cur: &mut Vec<SifrInt>,
        total: &SifrInt,
        items: &[SifrInt],
        limit: &SifrInt,
        res: &mut Vec<Vec<SifrInt>>,
    ) {
        if total == limit {
            res.push(cur.clone());
            return;
        }
        if i < SifrInt::from_i64(0) || i >= items.len() || total > limit {
            return;
        }
        let Some(sifr_generated_checked_value_1) = ({
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
        cur.push(sifr_generated_checked_value_1.clone());
        dfs(
            &i.clone(),
            cur,
            &::std::ops::Add::add(total, sifr_generated_checked_value_1),
            items,
            &limit.clone(),
            res,
        );
        cur.pop();
        dfs(
            &::std::ops::Add::add(i, &SifrInt::from_i64(1)),
            cur,
            &total.clone(),
            items,
            &limit.clone(),
            res,
        );
    }
    let mut res: Vec<Vec<SifrInt>> = Vec::new();
    dfs(
        &SifrInt::from_i64(0),
        &mut Vec::new(),
        &SifrInt::from_i64(0),
        items,
        &limit,
        &mut res,
    );
    res
}
fn main() {
    assert_eq!(score(SifrInt::from_i64(4)), SifrInt::from_i64(20));
    assert_eq!(
        accumulate(&[
            SifrInt::from_i64(2),
            SifrInt::from_i64(7),
            SifrInt::from_i64(1),
            SifrInt::from_i64(8)
        ]),
        SifrInt::from_i64(18)
    );
    assert_eq!(
        format!(
            "{:?}",
            collect_prefixes(&[
                SifrInt::from_i64(1),
                SifrInt::from_i64(2),
                SifrInt::from_i64(3)
            ])
        ),
        "[[1, 2, 3], [1, 2], [1, 3], [1], [2, 3], [2], [3], []]"
    );
    assert_eq!(
        format!(
            "{:?}",
            collect_value_groups(
                &[
                    SifrInt::from_i64(1),
                    SifrInt::from_i64(2),
                    SifrInt::from_i64(4)
                ],
                SifrInt::from_i64(4)
            )
        ),
        "[[1, 1, 1, 1], [1, 1, 2], [2, 2], [4]]"
    );
}
