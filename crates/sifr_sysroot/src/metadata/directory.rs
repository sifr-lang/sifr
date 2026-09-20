//! Already validated canonical order needs no per-record tree allocation at open.
use super::RecordId;
use super::container::Entry;

pub(super) struct Directory(pub Vec<(RecordId, Entry)>);
impl Directory {
    pub(super) fn get(&self, id: &RecordId) -> Option<&Entry> {
        self.0
            .binary_search_by_key(id, |(key, _)| *key)
            .ok()
            .map(|index| &self.0[index].1)
    }
    pub(super) fn iter(&self) -> impl Iterator<Item = (&RecordId, &Entry)> {
        self.0.iter().map(|(id, entry)| (id, entry))
    }
}
impl<'a> IntoIterator for &'a Directory {
    type Item = (&'a RecordId, &'a Entry);
    type IntoIter = std::iter::Map<
        std::slice::Iter<'a, (RecordId, Entry)>,
        fn(&'a (RecordId, Entry)) -> Self::Item,
    >;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().map(|(id, entry)| (id, entry))
    }
}
