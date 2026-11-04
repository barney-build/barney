// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! String intern tables

use lasso::{Rodeo, Spur};

/// BnyInterner is used to intern symbols into a table
///
/// Internally we're just using `lasso::Rodeo` to encapsulate
/// the API and hide it from external consumers. Right this moment
/// we do not use the multithreaded implementation.
#[derive(Default)]
pub struct BnyInterner {
    intern: Rodeo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StringId(Spur);

impl BnyInterner {
    /// Return an interned symbol if it exists, otherwise inserting it
    pub fn intern(&mut self, key: &str) -> StringId {
        StringId(self.intern.get_or_intern(key))
    }

    /// Resolve a string from the symbol id
    pub fn resolve(&self, key: StringId) -> &str {
        self.intern.resolve(&key.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::BnyInterner;

    #[test]
    pub fn basic_intern() {
        let mut i = BnyInterner::default();
        let a = i.intern("name");
        let b = i.intern("age");
        let get_a = i.resolve(a);
        let get_b = i.resolve(b);
        assert_eq!(get_a, "name");
        assert_eq!(get_b, "age");
    }
}
