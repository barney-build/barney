// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Syntax document

use std::sync::Arc;

use kdl::{KdlDocument, KdlError};
use miette::{Diagnostic, MietteDiagnostic};
use thiserror::Error;

use crate::ProcessedNode;

/// A BnyDocument is used for parsing a `bny` file into
/// a set of nodes. Largely this is identical to a `kdl` file
/// but with the core tags etc locked in
pub struct BnyDocument {
    data: Arc<String>,
    doc: KdlDocument,
}

/// Diagnostic errors from various sources
#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    #[error(transparent)]
    #[diagnostic()]
    Kdl(#[from] KdlError),
}

/// A BnyResult yields diagnostics from parsing a `.bny` file
pub struct BnyResult<'a> {
    /// Processed nodes for what we've got so far.
    pub nodes: Vec<ProcessedNode<'a, usize>>,

    /// All diagnostics, error or otherwise
    pub diagnostics: Vec<MietteDiagnostic>,
}

impl<'a> BnyResult<'a> {
    /// Returns true if the result means we've got legal parsing,
    /// ie diagnostics are non fatal
    pub fn is_valid(&self) -> bool {
        self.diagnostics
            .iter()
            .find(|f| matches!(f.severity, Some(miette::Severity::Error)))
            .is_none()
    }
}

impl BnyDocument {
    pub fn new(data: Arc<String>) -> Result<Self, Error> {
        let doc = KdlDocument::parse_v2(&data)?;
        Ok(Self { data, doc })
    }

    /// Return all processed nodes and any diagnostics
    pub fn process<'a>(&self) -> BnyResult<'a> {
        BnyResult {
            nodes: vec![],
            diagnostics: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::BnyDocument;

    #[test]
    fn new_doc_simple() {
        let input = r#"document {
    child_node {
        list
    }
}
"#;
        let doc = BnyDocument::new(Arc::new(input.to_owned())).expect("unable to load doc");
        let res = doc.process();
        assert_eq!(res.diagnostics.len(), 0);
        assert!(res.is_valid());
    }
}
