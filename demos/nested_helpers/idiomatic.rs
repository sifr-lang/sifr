use std::collections::{HashMap, HashSet};

fn demo_letter_combinations(digits: &str) -> Vec<String> {
    let mut results = Vec::new();
    let digit_to_chars = HashMap::from([('2', ['a', 'b', 'c']), ('3', ['d', 'e', 'f'])]);

    fn backtrack(
        index: usize,
        current: &mut String,
        digits: &[char],
        digit_to_chars: &HashMap<char, [char; 3]>,
        results: &mut Vec<String>,
    ) {
        if index >= digits.len() {
            results.push(current.clone());
            return;
        }

        let Some(chars) = digit_to_chars.get(&digits[index]) else {
            return;
        };

        for ch in chars {
            current.push(*ch);
            backtrack(index + 1, current, digits, digit_to_chars, results);
            current.pop();
        }
    }

    if !digits.is_empty() {
        let digits = digits.chars().collect::<Vec<_>>();
        let mut current = String::new();
        backtrack(0, &mut current, &digits, &digit_to_chars, &mut results);
    }

    results
}

fn demo_total_n_queens(n: i64) -> i64 {
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

fn demo_redundant_connection(edges: &[(i64, i64)]) -> Vec<i64> {
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
        format!("{:?}", demo_letter_combinations("23")),
        "[\"ad\", \"ae\", \"af\", \"bd\", \"be\", \"bf\", \"cd\", \"ce\", \"cf\"]"
    );
    assert_eq!(demo_total_n_queens(4), 2);
    assert_eq!(
        demo_redundant_connection(&[(1, 2), (1, 3), (2, 3)]),
        vec![2, 3]
    );
}
