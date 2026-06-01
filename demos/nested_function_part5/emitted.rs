// src/main.rs
fn apply_twice(f: impl Fn(i64) -> i64, value: i64) -> i64 {
    f(f(value))
}

fn score(base: i64) -> i64 {
    let offset: i64 = 3_i64;
    let add_offset = |x| {
    x + offset
};
    let amplify = |x| {
    x * (2_i64)
};
    let adjusted: i64 = apply_twice(add_offset, base);
    amplify(adjusted)
}

fn accumulate(values: &Vec<i64>) -> i64 {
    let mut total: i64 = 0_i64;
    let mut apply = || {
    for value in values.iter().copied() {
        total += value;
    }
};
    apply();
    total
}

fn collect_prefixes(nums: &Vec<i64>) -> Vec<Vec<i64>> {
    let mut res: Vec<Vec<i64>> = vec![];
    let mut subset: Vec<i64> = vec![];
    fn dfs(i: i64, nums: &Vec<i64>, res: &mut Vec<Vec<i64>>, subset: &mut Vec<i64>) {
        if (i >= (nums.len() as i64)) {
            res.push(subset.clone());
            return;
        }
        subset.push({
    let Some(__sifr_index_value) = ({
    let __sifr_index_list = &nums;
    let __sifr_index_i = i;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
}) else {
        unreachable!("compiler-verified index should be in range");
    };
    __sifr_index_value
});
        dfs(i + (1_i64), nums, res, subset);
        subset.pop();
        dfs(i + (1_i64), nums, res, subset);
    }
    dfs(0_i64, nums, &mut res, &mut subset);
    res
}

fn collect_value_groups(items: &Vec<i64>, limit: i64) -> Vec<Vec<i64>> {
    let mut res: Vec<Vec<i64>> = vec![];
    fn dfs(i: i64, cur: &mut Vec<i64>, total: i64, items: &Vec<i64>, limit: i64, res: &mut Vec<Vec<i64>>) {
        if total == limit {
            res.push(cur.clone());
            return;
        }
        if (i >= (items.len() as i64)) || (total > limit) {
            return;
        }
        cur.push({
    let Some(__sifr_index_value) = ({
    let __sifr_index_list = &items;
    let __sifr_index_i = i;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
}) else {
        unreachable!("compiler-verified index should be in range");
    };
    __sifr_index_value
});
        dfs(i, cur, total + ({
    let Some(__sifr_index_value) = ({
    let __sifr_index_list = &items;
    let __sifr_index_i = i;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
}) else {
        unreachable!("compiler-verified index should be in range");
    };
    __sifr_index_value
}), items, limit, res);
        cur.pop();
        dfs(i + (1_i64), cur, total, items, limit, res);
    }
    dfs(0_i64, &mut vec![], 0_i64, items, limit, &mut res);
    res
}

fn main() {
    assert!((score(4_i64) == (20_i64)));
    assert!((accumulate(&vec![2_i64, 7_i64, 1_i64, 8_i64]) == (18_i64)));
    assert!((format!("{:?}", collect_prefixes(&vec![1_i64, 2_i64, 3_i64])) == "[[1, 2, 3], [1, 2], [1, 3], [1], [2, 3], [2], [3], []]"));
    assert!((format!("{:?}", collect_value_groups(&vec![1_i64, 2_i64, 4_i64], 4_i64)) == "[[1, 1, 1, 1], [1, 1, 2], [2, 2], [4]]"));
}
