// src/main.rs
fn collect_value_groups(items: &Vec<i64>, limit: i64) -> Vec<Vec<i64>> {
    let mut result: Vec<Vec<i64>> = vec![];
    fn dfs(i: i64, cur: &mut Vec<i64>, total: i64, items: &Vec<i64>, limit: i64, result: &mut Vec<Vec<i64>>) {
        if total == limit {
            result.push(cur.clone());
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
}), items, limit, result);
        cur.pop();
        dfs(i + (1_i64), cur, total, items, limit, result);
    }
    dfs(0_i64, &mut vec![], 0_i64, items, limit, &mut result);
    result
}

fn collect_prefixes(nums: &Vec<i64>) -> Vec<Vec<i64>> {
    let mut result: Vec<Vec<i64>> = vec![];
    let mut subset: Vec<i64> = vec![];
    fn dfs(i: i64, nums: &Vec<i64>, result: &mut Vec<Vec<i64>>, subset: &mut Vec<i64>) {
        if (i >= (nums.len() as i64)) {
            result.push(subset.clone());
            return;
        }
        dfs(i + (1_i64), nums, result, subset);
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
        dfs(i + (1_i64), nums, result, subset);
        subset.pop();
    }
    dfs(0_i64, nums, &mut result, &mut subset);
    result
}

fn main() {
    assert!((format!("{:?}", collect_value_groups(&vec![1_i64, 2_i64, 4_i64], 4_i64)) == "[[1, 1, 1, 1], [1, 1, 2], [2, 2], [4]]"));
    assert!((format!("{:?}", collect_prefixes(&vec![1_i64, 2_i64, 3_i64])) == "[[], [3], [2], [2, 3], [1], [1, 3], [1, 2], [1, 2, 3]]"));
}
