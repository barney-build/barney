// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! linked list semantics atop SlotStorage

use std::{fmt::Debug, iter};

use crate::{Block, SlotStorage};

/// Chaining of rows as a linked list
#[derive(Debug)]
struct ListSlot<T>
where
    T: Debug,
{
    // Head or list node data
    datum: Option<T>,

    // Next row in the list
    next: Option<usize>,

    // Last row in the list
    last: Option<usize>,
}

/// Wraps the SlotStorage to provide linked lists on rows
#[derive(Debug)]
pub struct LinkedSlotStorage<T>
where
    T: Debug,
{
    store: SlotStorage<ListSlot<T>>,
}

impl<T> LinkedSlotStorage<T>
where
    T: Debug,
{
    pub fn new() -> Self {
        Self {
            store: SlotStorage::new(),
        }
    }

    /// Allocate a block of storage
    pub fn allocate(&mut self, many: usize) -> Block {
        self.store.allocate(many, || ListSlot {
            datum: None,
            next: None,
            last: None,
        })
    }

    /// Retrieve data
    pub fn get(&self, block: Block, index: usize) -> Option<&T> {
        let head_idx = block.0.start + index;
        self.store.get(head_idx).datum.as_ref()
    }

    /// Push an item to the head or linked as a list.
    pub fn push(&mut self, head_block: Block, index: usize, datum: T) {
        debug_assert!(!head_block.is_empty(), "Cannot push to an empty block!");
        let head_idx = head_block.0.start + index;

        if self.store.data[head_idx].datum.is_none() {
            self.store.data[head_idx].datum.replace(datum);
        } else {
            let region = self.allocate(1);
            let storage = &mut self.store.data;
            debug_assert!(
                storage[region.0.start].datum.is_none(),
                "Cannot overwrite linked storage    "
            );

            if let Some(last_idx) = storage[head_idx].last {
                storage[last_idx].next.replace(region.0.start);
            } else {
                storage[head_idx].next.replace(region.0.start);
            }
            storage[region.0.start].datum.replace(datum);
            storage[head_idx].last.replace(region.0.start);
        }
    }

    /// iterate all the chains
    pub fn iter(&self, head_block: Block, index: usize) -> impl Iterator<Item = &T> {
        let item = if head_block.0.start + index < self.store.len() {
            Some(head_block.0.start + index)
        } else {
            None
        };
        iter::successors(item, |e| self.store.get(*e).next)
            .flat_map(|i| self.store.get(i).datum.as_ref())
    }
}

impl<T> Default for LinkedSlotStorage<T>
where
    T: Debug,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use crate::linked::LinkedSlotStorage;

    #[test]
    fn test_basic_rows() {
        let mut store: LinkedSlotStorage<u32> = LinkedSlotStorage::new();

        let block1 = store.allocate(3);
        let _block2 = store.allocate(3);
        let _block3 = store.allocate(3);

        store.push(block1, 0, 3);
        store.push(block1, 1, 4);

        store.push(block1, 0, 5);
        store.push(block1, 1, 6);
        store.push(block1, 1, 7);

        let chain1 = store.iter(block1, 0).collect::<Vec<_>>();
        let chain2 = store.iter(block1, 1).collect::<Vec<_>>();

        assert_eq!(chain1.as_slice(), &[&3, &5]);
        assert_eq!(chain2.as_slice(), &[&4, &6, &7]);

        let chain3 = store.iter(block1, 2).collect::<Vec<_>>();
        assert!(chain3.is_empty());
    }
}
