// src/main.rs
use ::sifr_runtime::SifrInt;
use ::sifr_runtime::SifrRange;
use ::std::collections::HashMap;
use ::std::collections::HashSet;
static SIFR_GENERATED_SIFR_HOISTED_DICT_0: ::std::sync::LazyLock<HashMap<String, Vec<String>>> =
    ::std::sync::LazyLock::new(|| {
        HashMap::from([
            (
                "L".to_string(),
                vec!["a".to_string(), "b".to_string(), "c".to_string()],
            ),
            (
                "R".to_string(),
                vec!["d".to_string(), "e".to_string(), "f".to_string()],
            ),
        ])
    });
fn expand_keyed_strings(keys: &str) -> Vec<String> {
    fn backtrack(
        i: &SifrInt,
        cur: &str,
        key_to_suffixes: &HashMap<String, Vec<String>>,
        keys: &str,
        res: &mut Vec<String>,
    ) {
        if i < &SifrInt::from_i64(0) || i >= &SifrInt::from(keys.chars().count()) {
            res.push(cur.to_string());
            return;
        }
        let Some(sifr_generated_checked_value_0) = {
            let sifr_generated_string_chars = keys.chars().collect::<Vec<char>>();
            let sifr_generated_string_index = i;
            let sifr_generated_string_index_normalized = sifr_generated_string_index
                .normalize_index_or_len(sifr_generated_string_chars.len());
            sifr_generated_string_chars
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string()) else {
            res.push(cur.to_string());
            return;
        };
        let key: String = sifr_generated_checked_value_0;
        let Some(sifr_generated_checked_value_1) = key_to_suffixes.get(&key) else {
            return;
        };
        #[expect(
            clippy::explicit_iter_loop,
            reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
        )]
        for suffix in sifr_generated_checked_value_1.iter() {
            backtrack(
                &::std::ops::Add::add(i, &SifrInt::from_i64(1)),
                &format!("{cur}{suffix}"),
                key_to_suffixes,
                keys,
                res,
            );
        }
    }
    let mut res: Vec<String> = Vec::new();
    let key_to_suffixes = &*SIFR_GENERATED_SIFR_HOISTED_DICT_0;
    if keys.chars().count() > SifrInt::from_i64(0) {
        backtrack(&SifrInt::from_i64(0), "", key_to_suffixes, keys, &mut res);
    }
    res
}
fn count_configurations(n: &SifrInt) -> SifrInt {
    fn backtrack(
        i: &SifrInt,
        cols: &mut HashSet<SifrInt>,
        posdiag: &mut HashSet<SifrInt>,
        negdiag: &mut HashSet<SifrInt>,
        n: &SifrInt,
    ) -> SifrInt {
        if i == n {
            return SifrInt::from_i64(1);
        }
        let mut count: SifrInt = SifrInt::from_i64(0);
        for j in
            SifrRange::new_known_nonzero(SifrInt::from_i64(0), (*n).clone(), SifrInt::from_i64(1))
        {
            if cols.contains(&j)
                || posdiag.contains(&::std::ops::Add::add(i, &j))
                || negdiag.contains(&::std::ops::Sub::sub(i, &j))
            {
                continue;
            }
            cols.insert(j.clone());
            posdiag.insert(::std::ops::Add::add(i, &j));
            negdiag.insert(::std::ops::Sub::sub(i, &j));
            count = ::std::ops::Add::add(
                &count,
                &backtrack(
                    &::std::ops::Add::add(i, &SifrInt::from_i64(1)),
                    cols,
                    posdiag,
                    negdiag,
                    n,
                ),
            );
            cols.remove(&j);
            posdiag.remove(&::std::ops::Add::add(i, &j));
            negdiag.remove(&::std::ops::Sub::sub(i, &j));
        }
        count
    }
    backtrack(
        &SifrInt::from_i64(0),
        &mut HashSet::new(),
        &mut HashSet::new(),
        &mut HashSet::new(),
        n,
    )
}
fn find_root(n: &SifrInt, par: &[SifrInt]) -> SifrInt {
    if n < &SifrInt::from_i64(0) || n >= &SifrInt::from(par.len()) {
        return SifrInt::from_i64(0);
    }
    let mut p: SifrInt = (*n).clone();
    while p >= SifrInt::from_i64(0) && p < par.len() && {
        let sifr_generated_checked_read_collection = &par;
        let sifr_generated_checked_read_index = &p;
        let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
            .normalize_index_or_len(sifr_generated_checked_read_collection.len());
        sifr_generated_checked_read_collection
            .get(sifr_generated_checked_read_normalized)
            .cloned()
    }
    .is_some_and(|sifr_generated_checked_value_2| sifr_generated_checked_value_2 != p)
    {
        let Some(sifr_generated_checked_value_3) = ({
            let sifr_generated_checked_read_collection = &par;
            let sifr_generated_checked_read_index = &p;
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        }) else {
            break;
        };
        p.clone_from(&sifr_generated_checked_value_3);
    }
    p.clone()
}
fn union_nodes(n1: &SifrInt, n2: &SifrInt, par: &mut [SifrInt], rank: &mut [SifrInt]) -> bool {
    let p1: SifrInt = find_root(n1, par);
    let p2: SifrInt = find_root(n2, par);
    if p1 < SifrInt::from_i64(0)
        || p1 >= rank.len()
        || p1 >= par.len()
        || p2 < SifrInt::from_i64(0)
        || p2 >= rank.len()
        || p2 >= par.len()
    {
        return false;
    }
    let Some(sifr_generated_checked_value_4) = ({
        let sifr_generated_checked_read_collection = &rank;
        let sifr_generated_checked_read_index = &p1;
        let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
            .normalize_index_or_len(sifr_generated_checked_read_collection.len());
        sifr_generated_checked_read_collection
            .get(sifr_generated_checked_read_normalized)
            .cloned()
    }) else {
        return false;
    };
    let Some(sifr_generated_checked_value_5) = ({
        let sifr_generated_checked_read_collection = &rank;
        let sifr_generated_checked_read_index = &p2;
        let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
            .normalize_index_or_len(sifr_generated_checked_read_collection.len());
        sifr_generated_checked_read_collection
            .get(sifr_generated_checked_read_normalized)
            .cloned()
    }) else {
        return false;
    };
    if p1 == p2 {
        return false;
    }
    if sifr_generated_checked_value_4 > sifr_generated_checked_value_5 {
        {
            let sifr_generated_assign_value = p1.clone();
            {
                let sifr_generated_index_raw = &p2;
                let sifr_generated_index_normalized =
                    sifr_generated_index_raw.normalize_index_or_len(par.len());
                if let Some(sifr_generated_elem) = par.get_mut(sifr_generated_index_normalized) {
                    *sifr_generated_elem = sifr_generated_assign_value;
                }
            }
        }
        {
            let sifr_generated_assign_value = ::std::ops::Add::add(
                &sifr_generated_checked_value_4,
                &sifr_generated_checked_value_5,
            );
            {
                let sifr_generated_index_raw = &p1;
                let sifr_generated_index_normalized =
                    sifr_generated_index_raw.normalize_index_or_len(rank.len());
                if let Some(sifr_generated_elem) = rank.get_mut(sifr_generated_index_normalized) {
                    *sifr_generated_elem = sifr_generated_assign_value;
                }
            }
        }
    } else {
        {
            let sifr_generated_assign_value = p2.clone();
            {
                let sifr_generated_index_raw = &p1;
                let sifr_generated_index_normalized =
                    sifr_generated_index_raw.normalize_index_or_len(par.len());
                if let Some(sifr_generated_elem) = par.get_mut(sifr_generated_index_normalized) {
                    *sifr_generated_elem = sifr_generated_assign_value;
                }
            }
        }
        {
            let sifr_generated_assign_value = ::std::ops::Add::add(
                &sifr_generated_checked_value_5,
                &sifr_generated_checked_value_4,
            );
            {
                let sifr_generated_index_raw = &p2;
                let sifr_generated_index_normalized =
                    sifr_generated_index_raw.normalize_index_or_len(rank.len());
                if let Some(sifr_generated_elem) = rank.get_mut(sifr_generated_index_normalized) {
                    *sifr_generated_elem = sifr_generated_assign_value;
                }
            }
        }
    }
    true
}
fn detect_first_cycle(edges: &[(SifrInt, SifrInt)]) -> Vec<SifrInt> {
    let mut par: Vec<SifrInt> = {
        let mut sifr_generated_list_comp = Vec::new();
        for i in SifrRange::new_known_nonzero(
            SifrInt::from_i64(0),
            ::std::ops::Add::add(&SifrInt::from(edges.len()), &SifrInt::from_i64(1)),
            SifrInt::from_i64(1),
        ) {
            sifr_generated_list_comp.push(i);
        }
        sifr_generated_list_comp
    };
    let mut rank: Vec<SifrInt> = {
        let mut sifr_generated_list_comp = Vec::new();
        for _ in SifrRange::new_known_nonzero(
            SifrInt::from_i64(0),
            ::std::ops::Add::add(&SifrInt::from(edges.len()), &SifrInt::from_i64(1)),
            SifrInt::from_i64(1),
        ) {
            sifr_generated_list_comp.push(SifrInt::from_i64(1));
        }
        sifr_generated_list_comp
    };
    for (n1, n2) in edges.iter().cloned() {
        if !union_nodes(&n1, &n2, &mut par, &mut rank) {
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
    assert_eq!(
        count_configurations(&SifrInt::from_i64(4)),
        SifrInt::from_i64(2)
    );
    assert_eq!(
        detect_first_cycle(&[
            (SifrInt::from_i64(1), SifrInt::from_i64(2)),
            (SifrInt::from_i64(1), SifrInt::from_i64(3)),
            (SifrInt::from_i64(2), SifrInt::from_i64(3))
        ]),
        vec![SifrInt::from_i64(2), SifrInt::from_i64(3)]
    );
}
