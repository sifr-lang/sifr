// src/main.rs
use std::collections::HashMap;

use std::collections::HashSet;

static __SIFR_HOISTED_DICT_0: std::sync::LazyLock<HashMap<String, Vec<String>>> = std::sync::LazyLock::new(|| HashMap::from([("L".to_string(), vec!["a".to_string(), "b".to_string(), "c".to_string()]), ("R".to_string(), vec!["d".to_string(), "e".to_string(), "f".to_string()])]));

fn expand_keyed_strings(keys: &String) -> Vec<String> {
    let mut __sifr_chars_keys: Vec<char> = keys.chars().collect::<Vec<char>>();
    let mut res: Vec<String> = vec![];
    let key_to_suffixes = &*__SIFR_HOISTED_DICT_0;
    fn backtrack(i: i64, cur: &String, key_to_suffixes: &HashMap<String, Vec<String>>, keys: &String, res: &mut Vec<String>) {
        if (i >= (keys.chars().count() as i64)) {
            res.push(format!("{}", cur));
            return;
        }
        let key: String = {
    let Some(__indexed_char) = keys.chars().nth(i as usize).map(|c| c.to_string()) else {
        unreachable!("compiler-verified string index should be in range");
    };
    __indexed_char
};
        if !key_to_suffixes.contains_key(&key) {
            return;
        }
        for suffix in {
    let Some(__sifr_dict_iter_source) = key_to_suffixes.get(&key) else {
        std::process::abort();
    };
    __sifr_dict_iter_source.iter().cloned()
} {
            backtrack(i + (1_i64), &format!("{}{}", cur, suffix), key_to_suffixes, keys, res);
        }
    }
    if ((__sifr_chars_keys.len() as i64) > (0_i64)) {
        backtrack(0_i64, &"".to_string(), &key_to_suffixes, keys, &mut res);
    }
    res
}

fn count_configurations(n: i64) -> i64 {
    fn backtrack(i: i64, cols: &mut HashSet<i64>, posdiag: &mut HashSet<i64>, negdiag: &mut HashSet<i64>, n: i64) -> i64 {
        if i == n {
            return 1_i64;
        }
        let mut count: i64 = 0_i64;
        for j in 0_i64..n {
            if (cols.contains(&j) || posdiag.contains(&(i + j))) || negdiag.contains(&(i - j)) {
                continue;
            }
            cols.insert((j).clone());
            posdiag.insert(i + j);
            negdiag.insert(i - j);
            count += backtrack(i + (1_i64), cols, posdiag, negdiag, n);
            cols.remove(&j);
            posdiag.remove(&(i + j));
            negdiag.remove(&(i - j));
        }
        return count;
    }
    backtrack(0_i64, &mut HashSet::new(), &mut HashSet::new(), &mut HashSet::new(), n)
}

fn find_root(n: i64, par: &Vec<i64>) -> i64 {
    if (n < (0_i64)) || (n >= (par.len() as i64)) {
        return 0_i64;
    }
    let mut p: i64 = n;
    while ((p >= (0_i64)) && (p < (par.len() as i64))) && (par[p as usize] != p) {
        p = par[p as usize];
    }
    p
}

fn union_nodes(n1: i64, n2: i64, par: &mut Vec<i64>, rank: &mut Vec<i64>) -> bool {
    let p1: i64 = find_root(n1, par);
    let p2: i64 = find_root(n2, par);
    if (((p1 < (0_i64)) || (p1 >= (rank.len() as i64))) || (p2 < (0_i64))) || (p2 >= (rank.len() as i64)) {
        return false;
    }
    if p1 == p2 {
        return false;
    }
    if (rank[p1 as usize] > rank[p2 as usize]) {
        {
            let __idx_raw = p2;
            let __idx_norm = if __idx_raw < 0 { (par.len() as i64) + __idx_raw } else { __idx_raw };
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
                let __idx_norm = if __idx_raw < 0 { (rank.len() as i64) + __idx_raw } else { __idx_raw };
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
            let __idx_norm = if __idx_raw < 0 { (par.len() as i64) + __idx_raw } else { __idx_raw };
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
                let __idx_norm = if __idx_raw < 0 { (rank.len() as i64) + __idx_raw } else { __idx_raw };
                if __idx_norm >= 0 {
                    if let Some(__elem) = rank.get_mut(__idx_norm as usize) {
                        *__elem = __assign_value;
                    }
                }
            }
        }
    }
    true
}

fn detect_first_cycle(edges: &Vec<(i64, i64)>) -> Vec<i64> {
    let mut par: Vec<i64> = {
    let mut __sifr_list_comp = vec![];
    for i in 0_i64..(edges.len() as i64) + (1_i64) {
        __sifr_list_comp.push(i);
    }
    __sifr_list_comp
};
    let mut rank: Vec<i64> = {
    let mut __sifr_list_comp = vec![];
    for _ in 0_i64..(edges.len() as i64) + (1_i64) {
        __sifr_list_comp.push(1_i64);
    }
    __sifr_list_comp
};
    for (n1, n2) in edges.iter().copied() {
        if !(union_nodes(n1, n2, &mut par, &mut rank)) {
            return vec![n1, n2];
        }
    }
    vec![]
}

fn main() {
    assert!((format!("{:?}", expand_keyed_strings(&"LR".to_string())) == "[\"ad\", \"ae\", \"af\", \"bd\", \"be\", \"bf\", \"cd\", \"ce\", \"cf\"]"));
    assert!((count_configurations(4_i64) == (2_i64)));
    assert!((detect_first_cycle(&vec![(1_i64, 2_i64), (1_i64, 3_i64), (2_i64, 3_i64)]) == vec![2_i64, 3_i64]));
}
