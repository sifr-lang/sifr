use std::ffi::{CStr, CString, OsStr, OsString};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

use crate::{GetSize, GetSizeTracker};

impl GetSize for String {
    fn get_heap_size_with_tracker<T: GetSizeTracker>(&self, tracker: T) -> (usize, T) {
        (self.capacity(), tracker)
    }
}

impl GetSize for &str {}

impl GetSize for CString {
    fn get_heap_size_with_tracker<T: GetSizeTracker>(&self, tracker: T) -> (usize, T) {
        (self.as_bytes_with_nul().len(), tracker)
    }
}

impl GetSize for &CStr {
    fn get_heap_size_with_tracker<T: GetSizeTracker>(&self, tracker: T) -> (usize, T) {
        (self.to_bytes_with_nul().len(), tracker)
    }
}

impl GetSize for OsString {
    fn get_heap_size_with_tracker<T: GetSizeTracker>(&self, tracker: T) -> (usize, T) {
        (self.len(), tracker)
    }
}

impl GetSize for &OsStr {
    fn get_heap_size_with_tracker<T: GetSizeTracker>(&self, tracker: T) -> (usize, T) {
        (self.len(), tracker)
    }
}

impl GetSize for PathBuf {
    fn get_heap_size_with_tracker<T: GetSizeTracker>(&self, tracker: T) -> (usize, T) {
        (self.capacity(), tracker)
    }
}

impl GetSize for &Path {}

impl GetSize for std::fs::DirBuilder {}
impl GetSize for std::fs::DirEntry {}
impl GetSize for std::fs::File {}
impl GetSize for std::fs::FileType {}
impl GetSize for std::fs::Metadata {}
impl GetSize for std::fs::OpenOptions {}
impl GetSize for std::fs::Permissions {}
impl GetSize for std::fs::ReadDir {}

impl<T> GetSize for BufReader<T>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        let (total, tracker) = T::get_heap_size_with_tracker(self.get_ref(), tracker);
        (total + self.capacity(), tracker)
    }
}

impl<T> GetSize for BufWriter<T>
where
    T: GetSize + Write,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        let (total, tracker) = T::get_heap_size_with_tracker(self.get_ref(), tracker);
        (total + self.capacity(), tracker)
    }
}

impl<T> GetSize for Box<[T]>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        let (size, tracker) = self.iter().fold((0, tracker), |(size, tracker), element| {
            let (elem_size, tracker) = T::get_heap_size_with_tracker(element, tracker);
            (size + elem_size, tracker)
        });

        let allocation_size = self.len() * T::get_stack_size();
        (size + allocation_size, tracker)
    }
}

impl GetSize for Box<str> {
    fn get_heap_size_with_tracker<T: GetSizeTracker>(&self, tracker: T) -> (usize, T) {
        (self.len(), tracker)
    }
}

impl<T> GetSize for Rc<[T]>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        let (size, tracker) = self.iter().fold((0, tracker), |(size, tracker), element| {
            let (elem_size, tracker) = T::get_heap_size_with_tracker(element, tracker);
            (size + elem_size, tracker)
        });

        let allocation_size = self.len() * T::get_stack_size();
        (size + allocation_size, tracker)
    }
}

impl GetSize for Rc<str> {
    fn get_heap_size_with_tracker<T: GetSizeTracker>(&self, tracker: T) -> (usize, T) {
        (self.len(), tracker)
    }
}

impl<T> GetSize for Arc<[T]>
where
    T: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        let (size, tracker) = self.iter().fold((0, tracker), |(size, tracker), element| {
            let (elem_size, tracker) = T::get_heap_size_with_tracker(element, tracker);
            (size + elem_size, tracker)
        });

        let allocation_size = self.len() * T::get_stack_size();
        (size + allocation_size, tracker)
    }
}

impl GetSize for Arc<str> {
    fn get_heap_size_with_tracker<T: GetSizeTracker>(&self, tracker: T) -> (usize, T) {
        (self.len(), tracker)
    }
}
