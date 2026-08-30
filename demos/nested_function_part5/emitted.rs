// src/main.rs
use ::sifr_runtime::SifrInt;

fn apply_twice(f: impl Fn(SifrInt) -> SifrInt, value: SifrInt) -> SifrInt {
    f(f((value).clone()))
}

fn score(base: SifrInt) -> SifrInt {
    let offset: SifrInt = SifrInt::from_i64(3);
    let add_offset = |x: SifrInt| {
    &x + &offset
};
    let amplify = |x: SifrInt| {
    &x * &SifrInt::from_i64(2)
};
    let adjusted: SifrInt = apply_twice(add_offset, (base).clone());
    amplify((adjusted).clone())
}

fn accumulate(values: &Vec<SifrInt>) -> SifrInt {
    let mut total: SifrInt = SifrInt::from_i64(0);
    let mut apply = || {
    for value in values.iter().cloned() {
        total += value;
    }
};
    apply();
    total.clone()
}

fn collect_prefixes(nums: &Vec<SifrInt>) -> Vec<Vec<SifrInt>> {
    let mut res: Vec<Vec<SifrInt>> = vec![];
    let mut subset: Vec<SifrInt> = vec![];
    fn dfs(i: SifrInt, nums: &Vec<SifrInt>, res: &mut Vec<Vec<SifrInt>>, subset: &mut Vec<SifrInt>) {
        if (&i >= &SifrInt::from(nums.len())) {
            res.push(subset.clone());
            return;
        }
        subset.push({
    let __sifr_index_value_option = {
    let __sifr_index_list = &nums;
    let __sifr_index_i = i.clone();
    let __sifr_index_norm = __sifr_index_i.normalize_index_or_len(__sifr_index_list.len());
    __sifr_index_list.get(__sifr_index_norm).cloned()
};
    __sifr_index_value_option.as_slice()[0_usize].clone()
});
        dfs(&i + &SifrInt::from_i64(1), nums, res, subset);
        subset.pop();
        dfs(&i + &SifrInt::from_i64(1), nums, res, subset);
    }
    dfs(SifrInt::from_i64(0), nums, &mut res, &mut subset);
    res
}

fn collect_value_groups(items: &Vec<SifrInt>, limit: SifrInt) -> Vec<Vec<SifrInt>> {
    let mut res: Vec<Vec<SifrInt>> = vec![];
    fn dfs(i: SifrInt, cur: &mut Vec<SifrInt>, total: SifrInt, items: &Vec<SifrInt>, limit: SifrInt, res: &mut Vec<Vec<SifrInt>>) {
        if &total == &limit {
            res.push(cur.clone());
            return;
        }
        if (&i >= &SifrInt::from(items.len())) || (&total > &limit) {
            return;
        }
        cur.push({
    let __sifr_index_value_option = {
    let __sifr_index_list = &items;
    let __sifr_index_i = i.clone();
    let __sifr_index_norm = __sifr_index_i.normalize_index_or_len(__sifr_index_list.len());
    __sifr_index_list.get(__sifr_index_norm).cloned()
};
    __sifr_index_value_option.as_slice()[0_usize].clone()
});
        dfs((i).clone(), cur, &total + ({
    let __sifr_index_value_option = {
    let __sifr_index_list = &items;
    let __sifr_index_i = i.clone();
    let __sifr_index_norm = __sifr_index_i.normalize_index_or_len(__sifr_index_list.len());
    __sifr_index_list.get(__sifr_index_norm).cloned()
};
    __sifr_index_value_option.as_slice()[0_usize].clone()
}), items, limit.clone(), res);
        cur.pop();
        dfs(&i + &SifrInt::from_i64(1), cur, (total).clone(), items, limit.clone(), res);
    }
    dfs(SifrInt::from_i64(0), &mut vec![], SifrInt::from_i64(0), items, limit.clone(), &mut res);
    res
}

fn main() {
    assert!((&score(SifrInt::from_i64(4)) == &SifrInt::from_i64(20)));
    assert!((&accumulate(&vec![SifrInt::from_i64(2), SifrInt::from_i64(7), SifrInt::from_i64(1), SifrInt::from_i64(8)]) == &SifrInt::from_i64(18)));
    assert!((format!("{:?}", collect_prefixes(&vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3)])) == "[[1, 2, 3], [1, 2], [1, 3], [1], [2, 3], [2], [3], []]"));
    assert!((format!("{:?}", collect_value_groups(&vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(4)], SifrInt::from_i64(4))) == "[[1, 1, 1, 1], [1, 1, 2], [2, 2], [4]]"));
}
