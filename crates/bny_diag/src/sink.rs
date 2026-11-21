// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Diagnostic sink

use std::{error::Error, range::Range};

use bny_slots::{Block, LinkedSlotStorage, SlotStorage};
use lasso::{Rodeo, Spur};
use miette::{Severity, SourceSpan};

use crate::BnyDiagnosticRef;

#[derive(Debug, Default)]
pub(crate) struct BnyDiag {
    pub(crate) severity: Severity,
    pub(crate) message: Spur,
    pub(crate) help: Option<Spur>,
    pub(crate) labels: Option<usize>,
}

#[derive(Debug)]
pub(crate) struct BnyLabel {
    pub(crate) text: Spur,
    pub(crate) span: SourceSpan,
}

#[derive(Debug, Clone, Copy)]
pub struct DiagnosticId(Block);

/// Push diagnostics into a sink rather than maintain
/// multiple Vecs or require bail-early patterns
/// We may support warn/advice if/when needed
#[derive(Debug)]
pub struct DiagnosticSink {
    pub(crate) diagnostics: SlotStorage<BnyDiag>,
    pub(crate) labels: LinkedSlotStorage<BnyLabel>,
    pub(crate) intern: Rodeo,
}

impl DiagnosticSink {
    pub fn new() -> Self {
        Self {
            diagnostics: SlotStorage::new(),
            labels: LinkedSlotStorage::new(),
            intern: Rodeo::new(),
        }
    }

    /// Insert an error into the sink
    pub fn insert_error(&mut self, error: impl Error, help: Option<&str>) -> DiagnosticId {
        self.insert(Severity::Error, &error.to_string(), help)
    }

    pub fn insert(
        &mut self,
        severity: Severity,
        message: &str,
        help: Option<&str>,
    ) -> DiagnosticId {
        let block = self.diagnostics.allocate(1, Default::default);
        self.diagnostics.set(
            block.0.start,
            BnyDiag {
                severity,
                message: self.intern.get_or_intern(message),
                labels: None,
                help: help.map(|h| self.intern.get_or_intern(h)),
            },
        );
        DiagnosticId(block)
    }

    // First span is the primary label, everything else is secondary
    pub fn push_span(&mut self, id: DiagnosticId, span: SourceSpan, label: &str) {
        let diag = self.diagnostics.get_mut(id.0.0.start);
        let label = BnyLabel {
            text: self.intern.get_or_intern(label),
            span,
        };
        match diag.labels {
            // Link list of labels (head is primary, everything else is secondary)
            Some(head) => {
                self.labels.push(
                    Block(Range {
                        start: head,
                        end: head + 1,
                    }),
                    0,
                    label,
                );
            }
            None => {
                let alloc = self.labels.allocate(1);
                self.labels.push(alloc, 0, label);
                diag.labels.replace(alloc.0.start);
            }
        }
    }

    pub(crate) fn get_internal(&self, id: DiagnosticId) -> &BnyDiag {
        self.diagnostics.get(id.0.0.start)
    }

    /// Get a ref
    pub fn get(&self, id: DiagnosticId) -> BnyDiagnosticRef {
        BnyDiagnosticRef { id }
    }

    // Retrieve all diagnostics
    pub fn iter(&self) -> impl Iterator<Item = BnyDiagnosticRef> {
        self.diagnostics
            .iter()
            .enumerate()
            .map(|(id, _)| BnyDiagnosticRef {
                id: DiagnosticId(Block(Range {
                    start: id,
                    end: id + 1,
                })),
            })
    }
}

impl Default for DiagnosticSink {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::DiagnosticSink;
    use miette::NamedSource;
    use thiserror::Error;

    #[derive(Debug, Error)]
    enum FakeError {
        #[error("this is some kind of error message :O")]
        Main,
    }

    #[test]
    fn test_basic_emit() {
        let mut sink = DiagnosticSink::new();
        let id = sink.insert_error(FakeError::Main, Some("you should try harder"));
        sink.push_span(id, (0, 4).into(), "This is the word this");

        let diag = sink.get(id);
        let source = r#"this { is "my" source
#include "stdio.h"
code nyaha }
        "#;
        let report = miette::Report::from(diag.as_miette(&sink))
            .with_source_code(NamedSource::new("test.kdl", source).with_language("C"));

        eprintln!("{:?}", report);
    }
}
