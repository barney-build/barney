// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Slot storage for a doubly linked list of siblings on a rule match

use crate::validated::MatchId;

#[derive(Debug, Default, Clone, Copy)]
pub(super) struct ChildSlot {
    pub(super) first: Option<MatchId>,
    pub(super) last: Option<MatchId>,
}

#[derive(Debug, Default)]
pub(super) struct ChildSlotStorage(pub Vec<ChildSlot>);

impl ChildSlotStorage {
    /// Construct new minimum capacity slot storage
    pub fn new(size: usize) -> Self {
        let mut s = Vec::with_capacity(size);
        s.resize_with(size, Default::default);
        ChildSlotStorage(s)
    }

    pub fn reserve(&mut self, many: usize) {
        // inclusive range but need length
        self.0.resize_with(many + self.len() + 1, Default::default);
    }

    // Actual length
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get_mut(&mut self, idx: usize) -> &mut ChildSlot {
        &mut self.0[idx]
    }

    // Safely get by index
    pub fn get(&self, idx: usize) -> Option<&ChildSlot> {
        self.0.get(idx)
    }
}
