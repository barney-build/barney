// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Diagnostic core type (miette power)

use std::{error::Error, fmt::Display, range::Range};

use bny_slots::Block;
use miette::{Diagnostic, LabeledSpan, Severity};

use crate::{DiagnosticId, DiagnosticSink};

#[derive(Debug)]
pub struct BnyDiagnosticRef {
    pub(crate) id: DiagnosticId,
}

impl BnyDiagnosticRef {
    pub fn as_miette(&self, sink: &DiagnosticSink) -> BnyDiagnosticMiette {
        let us: &crate::BnyDiag = sink.get_internal(self.id);
        let labels = if let Some(label_head) = us.labels {
            sink.labels
                .iter(
                    Block(Range {
                        start: label_head,
                        end: label_head + 1,
                    }),
                    0,
                )
                .enumerate()
                .map(|(index, label)| {
                    let label_span = sink.intern.resolve(&label.text).to_owned();
                    if index == 0 {
                        LabeledSpan::new_primary_with_span(Some(label_span), label.span)
                    } else {
                        LabeledSpan::new_with_span(Some(label_span), label.span)
                    }
                })
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        BnyDiagnosticMiette {
            message: sink.intern.resolve(&us.message).to_owned(),
            severity: us.severity,
            labels,
            help: us.help.map(|h| sink.intern.resolve(&h).to_owned()),
        }
    }
}

#[derive(Debug)]
pub struct BnyDiagnosticMiette {
    message: String,
    severity: Severity,
    labels: Vec<LabeledSpan>,
    help: Option<String>,
}

impl Display for BnyDiagnosticMiette {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for BnyDiagnosticMiette {}

impl Diagnostic for BnyDiagnosticMiette {
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        None
    }

    fn severity(&self) -> Option<miette::Severity> {
        Some(self.severity)
    }

    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        if let Some(help) = self.help.as_ref() {
            Some(Box::new(help))
        } else {
            None
        }
    }

    fn url<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        None
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        None
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        Some(Box::new(self.labels.iter().cloned()))
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
        None
    }

    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        None
    }
}
