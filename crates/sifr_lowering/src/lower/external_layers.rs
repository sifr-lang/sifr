//! Immutable module baseline and revision-owned external export overlay.
use std::{
    collections::{HashMap, hash_map::Entry},
    sync::Arc,
};
/// Reads combine a shared baseline with project exports. Baseline modules retain
/// priority; mutations operate only on the overlay, including removal/taking.
#[derive(Debug, Clone)]
pub struct ModuleMap<K, V> {
    baseline: Arc<HashMap<K, Arc<V>>>,
    overlay: HashMap<K, V>,
}
impl<K, V> Default for ModuleMap<K, V> {
    fn default() -> Self {
        Self {
            baseline: Arc::default(),
            overlay: HashMap::new(),
        }
    }
}
impl<V: Clone> ModuleMap<String, V> {
    pub fn extend_baseline(&mut self, other: &Self) {
        Arc::make_mut(&mut self.baseline)
            .extend(other.baseline.iter().map(|(k, v)| (k.clone(), v.clone())));
    }
    pub fn copy_overlay_from(&mut self, other: &Self) {
        self.overlay = other.overlay.clone();
    }
    pub fn freeze(&mut self) {
        if self.baseline.is_empty() {
            self.baseline = Arc::new(
                std::mem::take(&mut self.overlay)
                    .into_iter()
                    .map(|(k, v)| (k, Arc::new(v)))
                    .collect(),
            );
        }
    }
    pub fn get(&self, key: &str) -> Option<&V> {
        self.baseline
            .get(key)
            .map(Arc::as_ref)
            .or_else(|| self.overlay.get(key))
    }
    pub fn contains_key(&self, key: &str) -> bool {
        self.get(key).is_some()
    }
    pub fn insert(&mut self, key: String, value: V) -> Option<V> {
        self.overlay.insert(key, value)
    }
    pub fn remove(&mut self, key: &str) -> Option<V> {
        self.overlay.remove(key)
    }
    pub fn entry(&mut self, key: String) -> Entry<'_, String, V> {
        self.overlay.entry(key)
    }
    pub fn get_mut(&mut self, key: &str) -> Option<&mut V> {
        self.overlay.get_mut(key)
    }
    pub fn iter(&self) -> impl Iterator<Item = (&String, &V)> {
        self.baseline.iter().map(|(k, v)| (k, v.as_ref())).chain(
            self.overlay
                .iter()
                .filter(|(key, _)| !self.baseline.contains_key(*key)),
        )
    }
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.iter().map(|(k, _)| k)
    }
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.iter().map(|(_, v)| v)
    }
    pub fn is_empty(&self) -> bool {
        self.iter().next().is_none()
    }
    pub fn len(&self) -> usize {
        self.iter().count()
    }
}
impl<V: Clone> Extend<(String, V)> for ModuleMap<String, V> {
    fn extend<T: IntoIterator<Item = (String, V)>>(&mut self, iter: T) {
        for (key, value) in iter {
            self.insert(key, value);
        }
    }
}
impl<V: Clone> FromIterator<(String, V)> for ModuleMap<String, V> {
    fn from_iter<T: IntoIterator<Item = (String, V)>>(iter: T) -> Self {
        let mut map = Self::default();
        map.extend(iter);
        map
    }
}
impl<'a, V: Clone> IntoIterator for &'a ModuleMap<String, V> {
    type Item = (&'a String, &'a V);
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;
    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_baseline_clones_share_values_and_isolate_project_mutation() {
        let mut source = ModuleMap::default();
        source.insert("sifr.math".to_owned(), vec!["sqrt".to_owned()]);
        source.freeze();
        let mut first = source.clone();
        let mut second = source.clone();
        assert!(std::ptr::eq(
            source.get("sifr.math").unwrap(),
            first.get("sifr.math").unwrap()
        ));
        first.insert("app".to_owned(), vec!["first".to_owned()]);
        second.insert("app".to_owned(), vec!["second".to_owned()]);
        assert_eq!(first.get("app").unwrap(), &["first"]);
        assert_eq!(second.get("app").unwrap(), &["second"]);
        assert!(!source.contains_key("app"));
        first.remove("app");
        assert!(!first.contains_key("app"));
        assert!(second.contains_key("app"));
    }
    #[test]
    fn revision_updates_do_not_mutate_older_snapshots() {
        let mut defs = ModuleMap::default();
        defs.insert("user".to_owned(), vec![1]);
        let prior = defs.clone();
        defs.entry("user".to_owned()).or_default().push(2);
        assert_eq!(prior.get("user"), Some(&vec![1]));
        assert_eq!(defs.get("user"), Some(&vec![1, 2]));
    }
}
