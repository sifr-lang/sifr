fn apply_twice(f: impl Fn(i64) -> i64, value: i64) -> i64 {
    f(f(value))
}

fn score(base: i64) -> i64 {
    let offset = 3;
    let add_offset = |x| x + offset;
    let amplify = |x| x * 2;
    amplify(apply_twice(add_offset, base))
}

fn accumulate(values: &[i64]) -> i64 {
    let mut total = 0;
    let mut apply = || {
        for value in values.iter().copied() {
            total += value;
        }
    };
    apply();
    total
}

fn subsets(nums: &[i64]) -> Vec<Vec<i64>> {
    fn dfs(index: usize, nums: &[i64], subset: &mut Vec<i64>, result: &mut Vec<Vec<i64>>) {
        if index == nums.len() {
            result.push(subset.clone());
            return;
        }

        subset.push(nums[index]);
        dfs(index + 1, nums, subset, result);
        subset.pop();
        dfs(index + 1, nums, subset, result);
    }

    let mut result = Vec::new();
    dfs(0, nums, &mut Vec::new(), &mut result);
    result
}

fn combination_sum(candidates: &[i64], target: i64) -> Vec<Vec<i64>> {
    fn dfs(
        start: usize,
        total: i64,
        candidates: &[i64],
        target: i64,
        current: &mut Vec<i64>,
        result: &mut Vec<Vec<i64>>,
    ) {
        if total == target {
            result.push(current.clone());
            return;
        }
        if total > target {
            return;
        }

        for i in start..candidates.len() {
            let value = candidates[i];
            current.push(value);
            dfs(i, total + value, candidates, target, current, result);
            current.pop();
        }
    }

    let mut result = Vec::new();
    dfs(0, 0, candidates, target, &mut Vec::new(), &mut result);
    result
}

fn main() {
    assert_eq!(score(4), 20);
    assert_eq!(accumulate(&[2, 7, 1, 8]), 18);
    assert_eq!(
        subsets(&[1, 2, 3]),
        vec![
            vec![1, 2, 3],
            vec![1, 2],
            vec![1, 3],
            vec![1],
            vec![2, 3],
            vec![2],
            vec![3],
            vec![],
        ]
    );
    assert_eq!(
        combination_sum(&[2, 3, 6, 7], 7),
        vec![vec![2, 2, 3], vec![7]]
    );
}
