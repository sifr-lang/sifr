/// A vector whose first element is established by construction.
///
/// Generated code uses this representation only when Sifr's static sequence
/// analysis proves that a comprehension executes at least once. Keeping the
/// head separate lets a proven zero-index read stay total without an unwrap,
/// panic, default value, or unsafe access.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SifrNonEmptyVec<T> {
    pub head: T,
    tail: Vec<T>,
}

impl<T> SifrNonEmptyVec<T> {
    #[must_use]
    pub fn new(head: T, tail: Vec<T>) -> Self {
        Self { head, tail }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.tail.len() + 1
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        false
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<&T> {
        if index == 0 {
            Some(&self.head)
        } else {
            self.tail.get(index - 1)
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index == 0 {
            Some(&mut self.head)
        } else {
            self.tail.get_mut(index - 1)
        }
    }

    #[must_use]
    pub fn into_vec(self) -> Vec<T> {
        let mut values = Vec::with_capacity(self.len());
        values.push(self.head);
        values.extend(self.tail);
        values
    }
}
