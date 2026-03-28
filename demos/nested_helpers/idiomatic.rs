use std::collections::HashMap;

use std::collections::HashSet;

fn demo_letter_combinations(digits: &String) -> Vec<String> {
    let mut res: Vec<String> = vec![];
    let digit_to_chars: HashMap<String, Vec<String>> = HashMap::from([
        (
            "2".to_string(),
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
        ),
        (
            "3".to_string(),
            vec!["d".to_string(), "e".to_string(), "f".to_string()],
        ),
    ]);
    fn backtrack(
        i: i64,
        cur: &String,
        digit_to_chars: &HashMap<String, Vec<String>>,
        digits: &String,
        res: &mut Vec<String>,
    ) {
        if i >= (digits.chars().count() as i64) {
            res.push(format!("{}", cur));
            return;
        }
        let d: String = {
            let Some(__indexed_char) = digits.chars().nth(i as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        };
        if !digit_to_chars.contains_key(&d) {
            return;
        }
        for ch in digit_to_chars
            .get(&d)
            .cloned()
            .expect(&"dict index proven by guard".to_string())
        {
            backtrack(
                i + (1 as i64),
                &format!("{}{}", cur, ch),
                digit_to_chars,
                digits,
                res,
            );
        }
    }
    if (digits.chars().count() as i64) > (0 as i64) {
        backtrack(0 as i64, &"".to_string(), &digit_to_chars, digits, &mut res);
    }
    return res;
}

fn demo_total_n_queens(n: i64) -> i64 {
    fn backtrack(
        i: i64,
        cols: &mut HashSet<i64>,
        posdiag: &mut HashSet<i64>,
        negdiag: &mut HashSet<i64>,
        n: i64,
    ) -> i64 {
        if i == n {
            return 1 as i64;
        }
        let mut count: i64 = 0 as i64;
        for j in 0 as i64..n {
            if (cols.contains(&j) || posdiag.contains(&(i + j))) || negdiag.contains(&(i - j)) {
                continue;
            }
            cols.insert(j);
            posdiag.insert(i + j);
            negdiag.insert(i - j);
            count += backtrack(i + (1 as i64), cols, posdiag, negdiag, n);
            cols.remove(&j);
            posdiag.remove(&(i + j));
            negdiag.remove(&(i - j));
        }
        return count;
    }
    return backtrack(
        0 as i64,
        &mut HashSet::new(),
        &mut HashSet::new(),
        &mut HashSet::new(),
        n,
    );
}

fn find_root(n: i64, par: &Vec<i64>) -> i64 {
    if ((n < (0 as i64)) || (n >= (par.len() as i64))) {
        return 0 as i64;
    }
    let mut p: i64 = n;
    while (((p >= (0 as i64)) && (p < (par.len() as i64)))
        && (({
            let __sifr_index_list = &par;
            let __sifr_index_i = p;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).copied()
        }) != Some(p)))
    {
        p = par[p as usize];
    }
    return p;
}

fn union_nodes(n1: i64, n2: i64, par: &mut Vec<i64>, rank: &mut Vec<i64>) -> bool {
    let p1: i64 = find_root(n1, par);
    let p2: i64 = find_root(n2, par);
    if ((((p1 < (0 as i64)) || (p1 >= (rank.len() as i64))) || (p2 < (0 as i64)))
        || (p2 >= (rank.len() as i64)))
    {
        return false;
    }
    if p1 == p2 {
        return false;
    }
    if rank[p1 as usize] > rank[p2 as usize] {
        {
            let __idx_raw = p2;
            let __idx_norm = if __idx_raw < 0 {
                (par.len() as i64) + __idx_raw
            } else {
                __idx_raw
            };
            if __idx_norm >= 0 {
                if let Some(__elem) = par.get_mut(__idx_norm as usize) {
                    *__elem = p1;
                }
            }
        }
        {
            let __assign_value = rank[p1 as usize] + rank[p2 as usize];
            {
                let __idx_raw = p1;
                let __idx_norm = if __idx_raw < 0 {
                    (rank.len() as i64) + __idx_raw
                } else {
                    __idx_raw
                };
                if __idx_norm >= 0 {
                    if let Some(__elem) = rank.get_mut(__idx_norm as usize) {
                        *__elem = __assign_value;
                    }
                }
            }
        }
    } else {
        {
            let __idx_raw = p1;
            let __idx_norm = if __idx_raw < 0 {
                (par.len() as i64) + __idx_raw
            } else {
                __idx_raw
            };
            if __idx_norm >= 0 {
                if let Some(__elem) = par.get_mut(__idx_norm as usize) {
                    *__elem = p2;
                }
            }
        }
        {
            let __assign_value = rank[p2 as usize] + rank[p1 as usize];
            {
                let __idx_raw = p2;
                let __idx_norm = if __idx_raw < 0 {
                    (rank.len() as i64) + __idx_raw
                } else {
                    __idx_raw
                };
                if __idx_norm >= 0 {
                    if let Some(__elem) = rank.get_mut(__idx_norm as usize) {
                        *__elem = __assign_value;
                    }
                }
            }
        }
    }
    return true;
}

fn demo_redundant_connection(edges: &Vec<(i64, i64)>) -> Vec<i64> {
    let mut par: Vec<i64> = {
        let mut __sifr_list_comp = vec![];
        for i in 0 as i64..(edges.len() as i64) + (1 as i64) {
            __sifr_list_comp.push(i);
        }
        __sifr_list_comp
    };
    let mut rank: Vec<i64> = {
        let mut __sifr_list_comp = vec![];
        for _ in 0 as i64..(edges.len() as i64) + (1 as i64) {
            __sifr_list_comp.push(1 as i64);
        }
        __sifr_list_comp
    };
    for (n1, n2) in edges.iter().copied() {
        if !(union_nodes(n1, n2, &mut par, &mut rank)) {
            return vec![n1, n2];
        }
    }
    return vec![];
}

fn main() {
    assert!(
        format!("{:?}", demo_letter_combinations(&"23".to_string()))
            == "[\"ad\", \"ae\", \"af\", \"bd\", \"be\", \"bf\", \"cd\", \"ce\", \"cf\"]"
                .to_string()
    );
    assert!(demo_total_n_queens(4 as i64) == (2 as i64));
    assert!(
        demo_redundant_connection(&vec![
            (1 as i64, 2 as i64),
            (1 as i64, 3 as i64),
            (2 as i64, 3 as i64)
        ]) == vec![2 as i64, 3 as i64]
    );
}
