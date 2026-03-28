fn combination_sum(candidates: &Vec<i64>, target: i64) -> Vec<Vec<i64>> {
    let mut result: Vec<Vec<i64>> = vec![];
    fn dfs(
        i: i64,
        cur: &mut Vec<i64>,
        total: i64,
        candidates: &Vec<i64>,
        result: &mut Vec<Vec<i64>>,
        target: i64,
    ) {
        if total == target {
            result.push(cur.clone());
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
            result,
            target,
        );
        cur.pop();
        dfs(i + (1 as i64), cur, total, candidates, result, target);
    }
    dfs(
        0 as i64,
        &mut vec![],
        0 as i64,
        candidates,
        &mut result,
        target,
    );
    return result;
}

fn subsets(nums: &Vec<i64>) -> Vec<Vec<i64>> {
    let mut result: Vec<Vec<i64>> = vec![];
    let mut subset: Vec<i64> = vec![];
    fn dfs(i: i64, nums: &Vec<i64>, result: &mut Vec<Vec<i64>>, subset: &mut Vec<i64>) {
        if i >= (nums.len() as i64) {
            result.push(subset.clone());
            return;
        }
        dfs(i + (1 as i64), nums, result, subset);
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
        dfs(i + (1 as i64), nums, result, subset);
        subset.pop();
    }
    dfs(0 as i64, nums, &mut result, &mut subset);
    return result;
}

fn main() {
    assert!(
        format!(
            "{:?}",
            combination_sum(&vec![2 as i64, 3 as i64, 6 as i64, 7 as i64], 7 as i64)
        ) == "[[2, 2, 3], [7]]".to_string()
    );
    assert!(
        format!("{:?}", subsets(&vec![1 as i64, 2 as i64, 3 as i64]))
            == "[[], [3], [2], [2, 3], [1], [1, 3], [1, 2], [1, 2, 3]]".to_string()
    );
}
