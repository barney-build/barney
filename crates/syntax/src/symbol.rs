// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Creates strong symbols based on namespace and ID

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SymbolError {
    #[error("namespace contains special prefix chars")]
    NamespaceContainsPrefixChars,

    #[error("id contains special prefix chars")]
    IdContainsPrefixChars,
}

#[repr(usize)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum SymbolType {
    // Defines a variable
    Variable,

    // Defines a (local) argument
    Argument,

    // Defines/provides a function
    Function,
}

/// A Symbol is essentially an encoding of symbol and namespace
/// ie `core::emit` or `autotools::configure`
/// As a special case the `core` domain is "always visible" and any
/// unqualified functor is assumed to be in `core` to emulate an import
/// pattern without the problems associated with them (shadowing etc)
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Symbol {
    // The namespace the symbol belongs to, i.e. `autotools`
    namespace: String,

    // The specific symbol ID, ie `configure`
    id: String,

    kind: SymbolType,
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}::{}", self.namespace, self.id))
    }
}

impl Symbol {
    /// Create a new symbol
    pub fn new(namespace: &str, id: &str, kind: SymbolType) -> Self {
        Self {
            namespace: namespace.to_string(),
            id: id.to_string(),
            kind,
        }
    }

    /// Return reference to the namespace
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Return a reference to the ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the kind of symbol
    pub fn kind(&self) -> SymbolType {
        self.kind
    }
}

/// A SymbolTable is basically an ordered map of symbols
/// that can be enumerated by prefix scans
#[derive(Default, Debug)]
pub struct SymbolTable {
    mapping: BTreeMap<String, Symbol>,

    // lazy cache for all known namespaces / prefixes
    namespaces: BTreeSet<String>,
}

impl<'a> SymbolTable {
    /// Construct a new SymbolTable
    pub fn new() -> Self {
        Default::default()
    }

    /// Insert and take ownership of a symbol
    pub fn insert(&mut self, symbol: Symbol) -> Result<(), SymbolError> {
        if symbol.namespace().contains("::") {
            Err(SymbolError::NamespaceContainsPrefixChars)
        } else if symbol.id().contains("::") {
            Err(SymbolError::IdContainsPrefixChars)
        } else {
            let fqdn = symbol.to_string();
            self.namespaces.insert(symbol.namespace.to_owned());
            self.mapping.insert(fqdn, symbol);
            Ok(())
        }
    }

    /// Allow symbol enumeration by way of a fixed prefix
    pub fn namespace_symbols<'b>(&'a self, namespace: &'b str) -> impl Iterator<Item = &'a Symbol> {
        let prefixed = format!("{}::", namespace);
        self.mapping
            .range(prefixed..)
            .take_while(move |(_, s)| s.namespace == namespace)
            .map(|(_, s)| s)
    }

    /// Read only references to namespaces
    pub fn namespaces(&'a self) -> impl Iterator<Item = &'a str> {
        self.namespaces.iter().map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    #[test]
    fn test_prefixes() {
        let mut tab = SymbolTable::new();
        tab.insert(Symbol {
            namespace: "autotools".to_string(),
            id: "configure".to_string(),
            kind: SymbolType::Function,
        })
        .expect("good symbol should work");
        tab.insert(Symbol {
            namespace: "autotools".to_string(),
            id: "make".to_string(),
            kind: SymbolType::Function,
        })
        .expect("good symbol should work");
        tab.insert(Symbol {
            namespace: "core".to_string(),
            id: "emit".to_string(),
            kind: SymbolType::Function,
        })
        .expect("good symbol should work");
        assert_eq!(tab.namespace_symbols("").collect_vec().len(), 0);
        assert_eq!(tab.namespace_symbols("autotools").collect_vec().len(), 2);
        assert_eq!(tab.namespace_symbols("core").collect_vec().len(), 1);
        assert_eq!(tab.namespaces.iter().collect_vec().len(), 2);
    }
}
