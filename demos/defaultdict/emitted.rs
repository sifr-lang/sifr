// src/main.rs
mod sifr_generated_project_nominals {
    use ::sifr_runtime::SifrInt;
    use ::std::collections::VecDeque;
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SifrGeneratedStdlibSifrX2ecollectionsX2edeque<T> {
        pub data: VecDeque<T>,
        pub maxlen: Option<SifrInt>,
    }
    impl<T: Clone> SifrGeneratedStdlibSifrX2ecollectionsX2edeque<T> {
        #[must_use]
        pub fn new(items: Option<Vec<T>>, maxlen: Option<SifrInt>) -> Self {
            let mut data: Vec<T> = Vec::new();
            if let Some(items) = items {
                let start: SifrInt = if let Some(maxlen) = maxlen.clone()
                    && &SifrInt::from(items.len()) > &maxlen
                {
                    &SifrInt::from(items.len()) - &maxlen
                } else {
                    SifrInt::from_i64(0)
                };
                let mut i: SifrInt = start.clone();
                while &i < &SifrInt::from(items.len()) {
                    let item_value_2841a0c596d6f426: Option<T> = {
                        let sifr_generated_checked_read_collection = &items;
                        let sifr_generated_checked_read_index = i.clone();
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    if let Some(item) = item_value_2841a0c596d6f426 {
                        data.push(item.clone());
                    }
                    i = &i + &SifrInt::from_i64(1);
                }
            }
            let sifr_generated_field_value_169953f6befb0270_6d61786c656e: Option<SifrInt> =
                maxlen.clone();
            let sifr_generated_field_value_90770dc80a1c57ce_5f64617461: VecDeque<T> =
                VecDeque::from(data);
            Self {
                maxlen: sifr_generated_field_value_169953f6befb0270_6d61786c656e,
                data: sifr_generated_field_value_90770dc80a1c57ce_5f64617461,
            }
        }
    }
    impl<T> SifrGeneratedStdlibSifrX2ecollectionsX2edeque<T> {
        #[must_use]
        pub fn len(&self) -> SifrInt {
            SifrInt::from(self.data.len())
        }
    }
}
use ::sifr_runtime::SifrInt;
use ::std::collections::HashMap;
use ::std::collections::HashSet;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ecollectionsX2edeque;
fn main() {
    let mut groups: HashMap<String, Vec<String>> = HashMap::new();
    {
        groups
            .entry("hit".to_string())
            .or_insert(Vec::new())
            .push("hot".to_string());
    };
    {
        groups
            .entry("hit".to_string())
            .or_insert(Vec::new())
            .push("hut".to_string());
    };
    assert_eq!(
        &groups.get("hit").map_or_else(
            || SifrInt::from_i64(0),
            |sifr_generated_bucket| SifrInt::from(sifr_generated_bucket.len())
        ),
        &SifrInt::from_i64(2)
    );
    let mut seen: HashMap<SifrInt, HashSet<String>> = HashMap::new();
    {
        seen.entry(SifrInt::from_i64(1))
            .or_insert(HashSet::new())
            .insert("a".to_string());
    };
    {
        seen.entry(SifrInt::from_i64(1))
            .or_insert(HashSet::new())
            .insert("b".to_string());
    };
    assert!(
        seen.get(&SifrInt::from_i64(1))
            .is_some_and(
                |sifr_generated_defaultdict_bucket| sifr_generated_defaultdict_bucket
                    .contains(&"a".to_string())
            )
    );
    let mut counts: HashMap<String, SifrInt> = HashMap::new();
    {
        let sifr_generated_elem = counts
            .entry("steps".to_string())
            .or_insert(SifrInt::from_i64(0));
        *sifr_generated_elem += SifrInt::from_i64(1);
    }
    {
        let sifr_generated_elem = counts
            .entry("steps".to_string())
            .or_insert(SifrInt::from_i64(0));
        *sifr_generated_elem += SifrInt::from_i64(2);
    }
    assert_eq!(
        &*counts
            .entry("steps".to_string())
            .or_insert(SifrInt::from_i64(0)),
        &SifrInt::from_i64(3)
    );
    let q: SifrGeneratedStdlibSifrX2ecollectionsX2edeque<SifrInt> =
        SifrGeneratedStdlibSifrX2ecollectionsX2edeque::new(
            Some(vec![
                SifrInt::from_i64(1),
                SifrInt::from_i64(2),
                SifrInt::from_i64(3),
            ]),
            None,
        );
    assert_eq!(&q.len(), &SifrInt::from_i64(3));
}
