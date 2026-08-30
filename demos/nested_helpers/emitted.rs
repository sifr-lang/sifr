// src/main.rs
use ::std::collections::HashMap;

use ::std::collections::HashSet;

use ::sifr_runtime::SifrInt;

use ::sifr_runtime::SifrRange;

static __SIFR_HOISTED_DICT_0: ::std::sync::LazyLock<HashMap<String, Vec<String>>> = ::std::sync::LazyLock::new(|| HashMap::from([("L".to_string(), vec!["a".to_string(), "b".to_string(), "c".to_string()]), ("R".to_string(), vec!["d".to_string(), "e".to_string(), "f".to_string()])]));

fn expand_keyed_strings(keys: &String) -> Vec<String> {
    let __sifr_chars_keys: Vec<char> = keys.chars().collect::<Vec<char>>();
    let mut res: Vec<String> = vec![];
    let key_to_suffixes = &*__SIFR_HOISTED_DICT_0;
    fn backtrack(i: SifrInt, cur: &String, key_to_suffixes: &HashMap<String, Vec<String>>, keys: &String, res: &mut Vec<String>) {
        if (&i >= &SifrInt::from(keys.chars().count())) {
            res.push(format!("{}", cur));
            return;
        }
        let key: String = {
    let __indexed_char_option = keys.chars().nth(::sifr_runtime::to_usize_proven(&(i))).map(|c| c.to_string());
    __indexed_char_option.as_slice()[0_usize].clone()
};
        if !key_to_suffixes.contains_key(&key) {
            return;
        }
        for suffix in key_to_suffixes[&key].iter().cloned() {
            backtrack(&i + &SifrInt::from_i64(1), &format!("{}{}", cur, suffix), key_to_suffixes, keys, res);
        }
    }
    if (&SifrInt::from(__sifr_chars_keys.len()) > &SifrInt::from_i64(0)) {
        backtrack(SifrInt::from_i64(0), &"".to_string(), &key_to_suffixes, keys, &mut res);
    }
    res
}

fn count_configurations(n: SifrInt) -> SifrInt {
    fn backtrack(i: SifrInt, cols: &mut HashSet<SifrInt>, posdiag: &mut HashSet<SifrInt>, negdiag: &mut HashSet<SifrInt>, n: SifrInt) -> SifrInt {
        if &i == &n {
            return SifrInt::from_i64(1);
        }
        let mut count: SifrInt = SifrInt::from_i64(0);
        for j in SifrRange::new_known_nonzero(SifrInt::from_i64(0), n.clone(), SifrInt::from_i64(1)) {
            if (cols.contains(&j) || posdiag.contains(&(&i + &j))) || negdiag.contains(&(&i - &j)) {
                continue;
            }
            cols.insert((j).clone());
            posdiag.insert(&i + &j);
            negdiag.insert(&i - &j);
            count = &count + &backtrack(&i + &SifrInt::from_i64(1), cols, posdiag, negdiag, n.clone());
            cols.remove(&j);
            posdiag.remove(&(&i + &j));
            negdiag.remove(&(&i - &j));
        }
        return count.clone();
    }
    backtrack(SifrInt::from_i64(0), &mut HashSet::new(), &mut HashSet::new(), &mut HashSet::new(), n.clone())
}

fn find_root(n: SifrInt, par: &Vec<SifrInt>) -> SifrInt {
    if (&n < &SifrInt::from_i64(0)) || (&n >= &SifrInt::from(par.len())) {
        return SifrInt::from_i64(0);
    }
    let mut p: SifrInt = n.clone();
    while ((&p >= &SifrInt::from_i64(0)) && (&p < &SifrInt::from(par.len()))) && (&par[::sifr_runtime::to_usize_proven(&(p))].clone() != &p) {
        p = par[::sifr_runtime::to_usize_proven(&(p))].clone();
    }
    p.clone()
}

fn union_nodes(n1: SifrInt, n2: SifrInt, par: &mut Vec<SifrInt>, rank: &mut Vec<SifrInt>) -> bool {
    let p1: SifrInt = find_root((n1).clone(), par);
    let p2: SifrInt = find_root((n2).clone(), par);
    if (((&p1 < &SifrInt::from_i64(0)) || (&p1 >= &SifrInt::from(rank.len()))) || (&p2 < &SifrInt::from_i64(0))) || (&p2 >= &SifrInt::from(rank.len())) {
        return false;
    }
    if &p1 == &p2 {
        return false;
    }
    if (rank[::sifr_runtime::to_usize_proven(&(p1))].clone() > rank[::sifr_runtime::to_usize_proven(&(p2))].clone()) {
        {
            let __idx_raw = p2.clone();
            let __idx_norm = __idx_raw.normalize_index_or_len(par.len());
            if let Some(__elem) = par.get_mut(__idx_norm) {
                *__elem = p1.clone();
            }
        }
        {
            let __assign_value = &rank[::sifr_runtime::to_usize_proven(&(p1))].clone() + &rank[::sifr_runtime::to_usize_proven(&(p2))].clone();
            {
                let __idx_raw = p1.clone();
                let __idx_norm = __idx_raw.normalize_index_or_len(rank.len());
                if let Some(__elem) = rank.get_mut(__idx_norm) {
                    *__elem = __assign_value;
                }
            }
        }
    } else {
        {
            let __idx_raw = p1.clone();
            let __idx_norm = __idx_raw.normalize_index_or_len(par.len());
            if let Some(__elem) = par.get_mut(__idx_norm) {
                *__elem = p2.clone();
            }
        }
        {
            let __assign_value = &rank[::sifr_runtime::to_usize_proven(&(p2))].clone() + &rank[::sifr_runtime::to_usize_proven(&(p1))].clone();
            {
                let __idx_raw = p2.clone();
                let __idx_norm = __idx_raw.normalize_index_or_len(rank.len());
                if let Some(__elem) = rank.get_mut(__idx_norm) {
                    *__elem = __assign_value;
                }
            }
        }
    }
    true
}

fn detect_first_cycle(edges: &Vec<(SifrInt, SifrInt)>) -> Vec<SifrInt> {
    let mut par: Vec<SifrInt> = {
    let mut __sifr_list_comp = vec![];
    for i in SifrRange::new_known_nonzero(SifrInt::from_i64(0), &SifrInt::from(edges.len()) + &SifrInt::from_i64(1), SifrInt::from_i64(1)) {
        __sifr_list_comp.push(i);
    }
    __sifr_list_comp
};
    let mut rank: Vec<SifrInt> = {
    let mut __sifr_list_comp = vec![];
    for _ in SifrRange::new_known_nonzero(SifrInt::from_i64(0), &SifrInt::from(edges.len()) + &SifrInt::from_i64(1), SifrInt::from_i64(1)) {
        __sifr_list_comp.push(SifrInt::from_i64(1));
    }
    __sifr_list_comp
};
    for (n1, n2) in edges.iter().cloned() {
        if !(union_nodes((n1).clone(), (n2).clone(), &mut par, &mut rank)) {
            return vec![n1.clone(), n2.clone()];
        }
    }
    vec![]
}

fn main() {
    assert!((format!("{:?}", expand_keyed_strings(&"LR".to_string())) == "[\"ad\", \"ae\", \"af\", \"bd\", \"be\", \"bf\", \"cd\", \"ce\", \"cf\"]"));
    assert!((&count_configurations(SifrInt::from_i64(4)) == &SifrInt::from_i64(2)));
    assert!((detect_first_cycle(&vec![(SifrInt::from_i64(1), SifrInt::from_i64(2)), (SifrInt::from_i64(1), SifrInt::from_i64(3)), (SifrInt::from_i64(2), SifrInt::from_i64(3))]) == vec![SifrInt::from_i64(2), SifrInt::from_i64(3)]));
}
