fn apply_twice(f: impl Fn(i64) -> i64, value: i64) -> i64 {
    return f(f(value));
}

fn score(base: i64) -> i64 {
    let offset: i64 = 3 as i64;
    let add_offset = |x| {
        return x + offset;
    };
    let amplify = |x| {
        return x * (2 as i64);
    };
    let adjusted: i64 = apply_twice(add_offset, base);
    return amplify(adjusted);
}

fn accumulate(values: &Vec<i64>) -> i64 {
    let mut total: i64 = 0 as i64;
    let mut apply = || {
        for value in values.iter().copied() {
            total += value;
        }
    };
    apply();
    return total;
}

fn subsets(nums: &Vec<i64>) -> Vec<Vec<i64>> {
    let mut res: Vec<Vec<i64>> = vec![];
    let mut subset: Vec<i64> = vec![];
    fn dfs(i: i64, nums: &Vec<i64>, res: &mut Vec<Vec<i64>>, subset: &mut Vec<i64>) {
        if i >= (nums.len() as i64) {
            res.push(subset.clone());
            return;
        }
        subset.push({
            let Some(__sifr_index_value) = ({
                let __sifr_index_list = &nums;
                let __sifr_index_i = i;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).copied()
            }) else {
                unreachable!("compiler-verified index should be in range");
            };
            __sifr_index_value
        });
        dfs(i + (1 as i64), nums, res, subset);
        subset.pop();
        dfs(i + (1 as i64), nums, res, subset);
    }
    dfs(0 as i64, nums, &mut res, &mut subset);
    return res;
}

fn combination_sum(candidates: &Vec<i64>, target: i64) -> Vec<Vec<i64>> {
    let mut res: Vec<Vec<i64>> = vec![];
    fn dfs(
        i: i64,
        cur: &mut Vec<i64>,
        total: i64,
        candidates: &Vec<i64>,
        res: &mut Vec<Vec<i64>>,
        target: i64,
    ) {
        if total == target {
            res.push(cur.clone());
            return;
        }
        if ((i >= (candidates.len() as i64)) || (total > target)) {
            return;
        }
        cur.push({
            let Some(__sifr_index_value) = ({
                let __sifr_index_list = &candidates;
                let __sifr_index_i = i;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).copied()
            }) else {
                unreachable!("compiler-verified index should be in range");
            };
            __sifr_index_value
        });
        dfs(
            i,
            cur,
            total
                + ({
                    let Some(__sifr_index_value) = ({
                        let __sifr_index_list = &candidates;
                        let __sifr_index_i = i;
                        let __sifr_index_norm = if __sifr_index_i < 0 {
                            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                        } else {
                            __sifr_index_i as usize
                        };
                        __sifr_index_list.get(__sifr_index_norm).copied()
                    }) else {
                        unreachable!("compiler-verified index should be in range");
                    };
                    __sifr_index_value
                }),
            candidates,
            res,
            target,
        );
        cur.pop();
        dfs(i + (1 as i64), cur, total, candidates, res, target);
    }
    dfs(
        0 as i64,
        &mut vec![],
        0 as i64,
        candidates,
        &mut res,
        target,
    );
    return res;
}

fn main() {
    assert!(score(4 as i64) == (20 as i64));
    assert!(accumulate(&vec![2 as i64, 7 as i64, 1 as i64, 8 as i64]) == (18 as i64));
    assert!(
        format!("{:?}", subsets(&vec![1 as i64, 2 as i64, 3 as i64]))
            == "[[1, 2, 3], [1, 2], [1, 3], [1], [2, 3], [2], [3], []]".to_string()
    );
    assert!(
        format!(
            "{:?}",
            combination_sum(&vec![2 as i64, 3 as i64, 6 as i64, 7 as i64], 7 as i64)
        ) == "[[2, 2, 3], [7]]".to_string()
    );
}
