//! Shared nominal payload storage. Contextual mutation preserves older owners.
use std::{
    fmt,
    ops::{Deref, DerefMut},
    sync::Arc,
};
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SharedVec<T>(Arc<Vec<T>>);
impl<T> Default for SharedVec<T> {
    fn default() -> Self {
        Self(Arc::new(Vec::new()))
    }
}
impl<T: fmt::Debug> fmt::Debug for SharedVec<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
impl<T> From<Vec<T>> for SharedVec<T> {
    fn from(value: Vec<T>) -> Self {
        Self(Arc::new(value))
    }
}
impl<T> FromIterator<T> for SharedVec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(values: I) -> Self {
        Self::from(values.into_iter().collect::<Vec<_>>())
    }
}
impl<T> Deref for SharedVec<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: Clone> DerefMut for SharedVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::make_mut(&mut self.0)
    }
}
impl<T> SharedVec<T> {
    pub fn shares_storage(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl<T: Clone> SharedVec<T> {
    pub fn into_vec(self) -> Vec<T> {
        match Arc::try_unwrap(self.0) {
            Ok(value) => value,
            Err(shared) => shared.as_ref().clone(),
        }
    }
}
impl<T: Clone> IntoIterator for SharedVec<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        self.into_vec().into_iter()
    }
}
impl<'a, T> IntoIterator for &'a SharedVec<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
impl<'a, T: Clone> IntoIterator for &'a mut SharedVec<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        Arc::make_mut(&mut self.0).iter_mut()
    }
}
impl<T: Clone> Extend<T> for SharedVec<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, values: I) {
        Arc::make_mut(&mut self.0).extend(values);
    }
}
impl<T: PartialEq> PartialEq<Vec<T>> for SharedVec<T> {
    fn eq(&self, other: &Vec<T>) -> bool {
        self.0.as_ref() == other
    }
}
impl<T: PartialEq> PartialEq<SharedVec<T>> for Vec<T> {
    fn eq(&self, other: &SharedVec<T>) -> bool {
        self == other.0.as_ref()
    }
}
#[cfg(test)]
mod tests {
    use super::SharedVec;
    #[test]
    fn dx7_shared_nominal_payload_clones_share_and_mutations_are_isolated() {
        let baseline = SharedVec::from(vec![String::from("field")]);
        let mut project = baseline.clone();
        assert!(baseline.shares_storage(&project));
        project.push("project".into());
        assert!(!baseline.shares_storage(&project));
        assert_eq!(baseline.as_slice(), &["field"]);
        assert_eq!(project.as_slice(), &["field", "project"]);
    }
}
