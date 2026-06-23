use std::collections::HashSet;
use std::sync::{Arc, Mutex, RwLock};

/// A tracker which makes sure that shared ownership objects are only accounted for once.
pub trait GetSizeTracker {
    /// Tracks an arbitrary object located at `addr`.
    ///
    /// Returns `true` if the reference, as indexed by the pointed to `addr`, has not yet
    /// been seen by this tracker. Otherwise it returns `false`.
    fn track<A>(&mut self, addr: *const A) -> bool;
}

impl<T: GetSizeTracker> GetSizeTracker for &mut T {
    fn track<A>(&mut self, addr: *const A) -> bool {
        GetSizeTracker::track(*self, addr)
    }
}

impl<T: GetSizeTracker> GetSizeTracker for Box<T> {
    fn track<A>(&mut self, addr: *const A) -> bool {
        GetSizeTracker::track(&mut **self, addr)
    }
}

impl<T: GetSizeTracker> GetSizeTracker for Mutex<T> {
    fn track<A>(&mut self, addr: *const A) -> bool {
        let tracker = self
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        GetSizeTracker::track(&mut *tracker, addr)
    }
}

impl<T: GetSizeTracker> GetSizeTracker for RwLock<T> {
    fn track<A>(&mut self, addr: *const A) -> bool {
        let mut tracker = self
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        GetSizeTracker::track(&mut *tracker, addr)
    }
}

impl<T: GetSizeTracker> GetSizeTracker for Arc<Mutex<T>> {
    fn track<A>(&mut self, addr: *const A) -> bool {
        let mut tracker = self
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        GetSizeTracker::track(&mut *tracker, addr)
    }
}

impl<T: GetSizeTracker> GetSizeTracker for Arc<RwLock<T>> {
    fn track<A>(&mut self, addr: *const A) -> bool {
        let mut tracker = self
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        GetSizeTracker::track(&mut *tracker, addr)
    }
}

/// A simple standard tracker which can be used to track shared ownership references.
#[derive(Debug, Default)]
pub struct StandardTracker {
    inner: HashSet<usize>,
}

impl StandardTracker {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

impl GetSizeTracker for StandardTracker {
    fn track<A>(&mut self, addr: *const A) -> bool {
        self.inner.insert(addr.addr())
    }
}

/// A pseudo tracker which does not track anything.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoTracker {
    answer: bool,
}

impl NoTracker {
    /// Creates a new pseudo tracker, which will always return the given `answer`.
    #[must_use]
    pub const fn new(answer: bool) -> Self {
        Self { answer }
    }

    /// Get the answer which will always be returned by this pseudo tracker.
    #[must_use]
    pub const fn answer(&self) -> bool {
        self.answer
    }

    /// Changes the answer which will always be returned by this pseudo tracker.
    pub const fn set_answer(&mut self, answer: bool) {
        self.answer = answer;
    }
}

impl GetSizeTracker for NoTracker {
    fn track<A>(&mut self, _addr: *const A) -> bool {
        self.answer
    }
}
