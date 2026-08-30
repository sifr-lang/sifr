// src/main.rs
use ::sifr_runtime::SifrInt;

fn collect_value_groups(items: &Vec<SifrInt>, limit: SifrInt) -> Vec<Vec<SifrInt>> {
    let mut result: Vec<Vec<SifrInt>> = vec![];
    fn dfs(i: SifrInt, cur: &mut Vec<SifrInt>, total: SifrInt, items: &Vec<SifrInt>, limit: SifrInt, result: &mut Vec<Vec<SifrInt>>) {
        if (&total == &limit) {
            result.push(cur.clone());
            return;
        }
        if ((&i < &SifrInt::from_i64(0)) || (&i >= &SifrInt::from(items.len()))) || (&total > &limit) {
            return;
        }
        let Some(__sifr_checked_value_0) = ({
    let __sifr_checked_read_collection = &items;
    let __sifr_checked_read_index = i.clone();
    let __sifr_checked_read_normalized = __sifr_checked_read_index.normalize_index_or_len(__sifr_checked_read_collection.len());
    __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
}) else {
            return;
        };
        cur.push(__sifr_checked_value_0.clone());
        dfs((i).clone(), cur, &total + __sifr_checked_value_0.clone(), items, limit.clone(), result);
        cur.pop();
        dfs(&i + &SifrInt::from_i64(1), cur, (total).clone(), items, limit.clone(), result);
    }
    dfs(SifrInt::from_i64(0), &mut vec![], SifrInt::from_i64(0), items, limit.clone(), &mut result);
    result
}

fn collect_prefixes(nums: &Vec<SifrInt>) -> Vec<Vec<SifrInt>> {
    let mut result: Vec<Vec<SifrInt>> = vec![];
    let mut subset: Vec<SifrInt> = vec![];
    fn dfs(i: SifrInt, nums: &Vec<SifrInt>, result: &mut Vec<Vec<SifrInt>>, subset: &mut Vec<SifrInt>) {
        if (&i < &SifrInt::from_i64(0)) || (&i >= &SifrInt::from(nums.len())) {
            result.push(subset.clone());
            return;
        }
        let Some(__sifr_checked_value_2) = ({
    let __sifr_checked_read_collection = &nums;
    let __sifr_checked_read_index = i.clone();
    let __sifr_checked_read_normalized = __sifr_checked_read_index.normalize_index_or_len(__sifr_checked_read_collection.len());
    __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
}) else {
            result.push(subset.clone());
            return;
        };
        dfs(&i + &SifrInt::from_i64(1), nums, result, subset);
        subset.push(__sifr_checked_value_2.clone());
        dfs(&i + &SifrInt::from_i64(1), nums, result, subset);
        subset.pop();
    }
    dfs(SifrInt::from_i64(0), nums, &mut result, &mut subset);
    result
}

fn main() {
    assert!((format!("{:?}", collect_value_groups(&vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(4)], SifrInt::from_i64(4))) == "[[1, 1, 1, 1], [1, 1, 2], [2, 2], [4]]"));
    assert!((format!("{:?}", collect_prefixes(&vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3)])) == "[[], [3], [2], [2, 3], [1], [1, 3], [1, 2], [1, 2, 3]]"));
}
