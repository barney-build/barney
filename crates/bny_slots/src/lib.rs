// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Super simple slot storage mechanism as the bny crates
//! currently have no need for mutating slot storage, invariably
//! used as sidecars.

mod storage;
pub use storage::{Block, SlotStorage};

mod linked;
pub use linked::LinkedSlotStorage;
