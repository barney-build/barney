// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Simple Vec<> storage

use std::{
    fmt::Debug,
    range::{Range, RangeIter},
    slice::Iter,
};

/// Simplistic slot allocation storage
#[derive(Debug, Default)]
pub struct SlotStorage<T>
where
    T: Debug,
{
    pub(crate) data: Vec<T>,
}

/// An allocation block
#[derive(Debug, Copy, Clone)]
pub struct Block(pub Range<usize>);

impl Block {
    /// Returns the block length
    pub fn len(&self) -> usize {
        self.0.end - self.0.start
    }

    /// Returns true if empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl IntoIterator for &Block {
    type Item = usize;

    type IntoIter = RangeIter<usize>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<T> SlotStorage<T>
where
    T: Debug,
{
    /// Construct new SlotStorage with no allocated data
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Allocate a block and return the storage mapping
    pub fn allocate<F>(&mut self, many: usize, f: F) -> Block
    where
        F: FnMut() -> T,
    {
        let block = Block(Range {
            start: self.data.len(),
            end: self.data.len() + many,
        });
        self.data.resize_with(self.data.len() + many, f);
        block
    }

    /// Set the value at index within the block to datum
    pub fn set(&mut self, index: usize, datum: T) {
        debug_assert!(index < self.data.len());
        self.data[index] = datum;
    }

    /// Retrieve reference to value at index within block
    pub fn get(&self, index: usize) -> &T {
        debug_assert!(index < self.data.len());
        &self.data[index]
    }

    /// Retrieve mutable reference to value at index within block
    pub fn get_mut(&mut self, index: usize) -> &mut T {
        debug_assert!(index < self.data.len());
        &mut self.data[index]
    }

    /// Iterate the items
    pub fn iter<'a>(&'a self) -> Iter<'a, T> {
        self.data.iter()
    }

    /// Return storage allocation length
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true if storage is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::SlotStorage;

    #[test]
    fn storage_test_simple() {
        let mut slots = SlotStorage::new();
        let _ = slots.allocate(5, || 0u32);
        slots.set(0, 1);
        slots.set(1, 2);

        assert_eq!(*slots.get(1), 2);
        assert_eq!(*slots.get(0), 1);

        // Now we have got allocations up to 5 so test its 0
        assert_eq!(*slots.get(3), 0);
    }
}
