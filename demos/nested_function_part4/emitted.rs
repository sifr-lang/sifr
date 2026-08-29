// src/main.rs
use ::sifr_runtime::SifrInt;

fn collect_value_groups(items: &Vec<SifrInt>, limit: SifrInt) -> Vec<Vec<SifrInt>> {
    let mut result: Vec<Vec<SifrInt>> = vec![];
    fn dfs(i: SifrInt, cur: &mut Vec<SifrInt>, total: SifrInt, items: &Vec<SifrInt>, limit: SifrInt, result: &mut Vec<Vec<SifrInt>>) {
        if &total == &limit {
            result.push(cur.clone());
            return;
        }
        if (&i >= &SifrInt::from(items.len())) || (&total > &limit) {
            return;
        }
        cur.push({
    let Some(__sifr_index_value) = ({
    let __sifr_index_list = &items;
    let __sifr_index_i = i.clone();
    let __sifr_index_norm = __sifr_index_i.normalize_index_or_len(__sifr_index_list.len());
    __sifr_index_list.get(__sifr_index_norm).cloned()
}) else {
        unreachable!("compiler-verified index should be in range");
    };
    __sifr_index_value
});
        dfs((i).clone(), cur, &total + ({
    let Some(__sifr_index_value) = ({
    let __sifr_index_list = &items;
    let __sifr_index_i = i.clone();
    let __sifr_index_norm = __sifr_index_i.normalize_index_or_len(__sifr_index_list.len());
    __sifr_index_list.get(__sifr_index_norm).cloned()
}) else {
        unreachable!("compiler-verified index should be in range");
    };
    __sifr_index_value
}), items, limit.clone(), result);
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
        if (&i >= &SifrInt::from(nums.len())) {
            result.push(subset.clone());
            return;
        }
        dfs(&i + &SifrInt::from_i64(1), nums, result, subset);
        subset.push({
    let Some(__sifr_index_value) = ({
    let __sifr_index_list = &nums;
    let __sifr_index_i = i.clone();
    let __sifr_index_norm = __sifr_index_i.normalize_index_or_len(__sifr_index_list.len());
    __sifr_index_list.get(__sifr_index_norm).cloned()
}) else {
        unreachable!("compiler-verified index should be in range");
    };
    __sifr_index_value
});
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
