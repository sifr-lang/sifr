use std::collections::{HashMap, HashSet};

fn expand_keyed_strings(keys: &str) -> Vec<String> {
    let mut results = Vec::new();
    let key_to_suffixes = HashMap::from([('L', ['a', 'b', 'c']), ('R', ['d', 'e', 'f'])]);

    fn backtrack(
        index: usize,
        current: &mut String,
        keys: &[char],
        key_to_suffixes: &HashMap<char, [char; 3]>,
        results: &mut Vec<String>,
    ) {
        if index >= keys.len() {
            results.push(current.clone());
            return;
        }

        let Some(suffixes) = key_to_suffixes.get(&keys[index]) else {
            return;
        };

        for suffix in suffixes {
            current.push(*suffix);
            backtrack(index + 1, current, keys, key_to_suffixes, results);
            current.pop();
        }
    }

    if !keys.is_empty() {
        let keys = keys.chars().collect::<Vec<_>>();
        let mut current = String::new();
        backtrack(0, &mut current, &keys, &key_to_suffixes, &mut results);
    }

    results
}

fn count_configurations(n: i64) -> i64 {
    fn backtrack(
        row: i64,
        n: i64,
        cols: &mut HashSet<i64>,
        pos_diag: &mut HashSet<i64>,
        neg_diag: &mut HashSet<i64>,
    ) -> i64 {
        if row == n {
            return 1;
        }

        let mut count = 0;
        for col in 0..n {
            if cols.contains(&col)
                || pos_diag.contains(&(row + col))
                || neg_diag.contains(&(row - col))
            {
                continue;
            }

            cols.insert(col);
            pos_diag.insert(row + col);
            neg_diag.insert(row - col);
            count += backtrack(row + 1, n, cols, pos_diag, neg_diag);
            cols.remove(&col);
            pos_diag.remove(&(row + col));
            neg_diag.remove(&(row - col));
        }

        count
    }

    backtrack(
        0,
        n,
        &mut HashSet::new(),
        &mut HashSet::new(),
        &mut HashSet::new(),
    )
}

fn find_root(node: i64, parent: &[i64]) -> i64 {
    if node < 0 || node >= parent.len() as i64 {
        return 0;
    }

    let mut current = node;
    while current >= 0 && current < parent.len() as i64 && parent[current as usize] != current {
        current = parent[current as usize];
    }
    current
}

fn union_nodes(n1: i64, n2: i64, parent: &mut [i64], rank: &mut [i64]) -> bool {
    let p1 = find_root(n1, parent);
    let p2 = find_root(n2, parent);
    if p1 < 0 || p1 >= rank.len() as i64 || p2 < 0 || p2 >= rank.len() as i64 {
        return false;
    }
    if p1 == p2 {
        return false;
    }

    let p1 = p1 as usize;
    let p2 = p2 as usize;
    if rank[p1] > rank[p2] {
        parent[p2] = p1 as i64;
        rank[p1] += rank[p2];
    } else {
        parent[p1] = p2 as i64;
        rank[p2] += rank[p1];
    }

    true
}

fn detect_first_cycle(edges: &[(i64, i64)]) -> Vec<i64> {
    let mut parent = (0..=edges.len() as i64).collect::<Vec<_>>();
    let mut rank = vec![1; edges.len() + 1];

    for &(n1, n2) in edges {
        if !union_nodes(n1, n2, &mut parent, &mut rank) {
            return vec![n1, n2];
        }
    }

    Vec::new()
}

fn main() {
    assert_eq!(
        format!("{:?}", expand_keyed_strings("LR")),
        "[\"ad\", \"ae\", \"af\", \"bd\", \"be\", \"bf\", \"cd\", \"ce\", \"cf\"]"
    );
    assert_eq!(count_configurations(4), 2);
    assert_eq!(
        detect_first_cycle(&[(1, 2), (1, 3), (2, 3)]),
        vec![2, 3]
    );
}
