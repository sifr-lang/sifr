fn collect_value_groups(items: &[i64], limit: i64) -> Vec<Vec<i64>> {
    fn dfs(
        start: usize,
        total: i64,
        items: &[i64],
        limit: i64,
        current: &mut Vec<i64>,
        result: &mut Vec<Vec<i64>>,
    ) {
        if total == limit {
            result.push(current.clone());
            return;
        }
        if total > limit {
            return;
        }

        for i in start..items.len() {
            let value = items[i];
            current.push(value);
            dfs(i, total + value, items, limit, current, result);
            current.pop();
        }
    }

    let mut result = Vec::new();
    dfs(0, 0, items, limit, &mut Vec::new(), &mut result);
    result
}

fn collect_prefixes(nums: &[i64]) -> Vec<Vec<i64>> {
    fn dfs(index: usize, nums: &[i64], subset: &mut Vec<i64>, result: &mut Vec<Vec<i64>>) {
        if index == nums.len() {
            result.push(subset.clone());
            return;
        }

        dfs(index + 1, nums, subset, result);
        subset.push(nums[index]);
        dfs(index + 1, nums, subset, result);
        subset.pop();
    }

    let mut result = Vec::new();
    dfs(0, nums, &mut Vec::new(), &mut result);
    result
}

fn main() {
    assert_eq!(
        collect_value_groups(&[1, 2, 4], 4),
        vec![vec![1, 1, 1, 1], vec![1, 1, 2], vec![2, 2], vec![4]]
    );
    assert_eq!(
        collect_prefixes(&[1, 2, 3]),
        vec![
            vec![],
            vec![3],
            vec![2],
            vec![2, 3],
            vec![1],
            vec![1, 3],
            vec![1, 2],
            vec![1, 2, 3],
        ]
    );
}
