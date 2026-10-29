// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Distro / OS definition cruft

use std::{fs, io, path::Path};

use kdl::{KdlDocument, KdlError};
use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

mod phase;
pub use phase::Phase;
mod module;
pub use module::Module;

use bny_syntax::{self, NodeSpec, SymbolTable};

/// A distro definition is taken from a bootstrap.kdl
///
#[derive(Debug)]
pub struct BootstrapSpec {
    phases: Vec<Phase>,
    modules: Vec<Module>,
    _symbols: SymbolTable,
}

#[repr(usize)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum SpecIdentity {
    /// Root phase
    Phase,
    /// The `variables` block itself
    PhaseVariables,
    /// Some variable in the phase variables list
    PhaseVariable,

    /// Module block
    Module,

    /// Module variables block
    ModuleVariables,

    /// Individual variable in module
    ModuleVariable,

    /// Module exports block
    ModuleExports,

    /// Individual export in a module
    ModuleExport,

    /// An action within a moduke
    ModuleAction,

    /// An actions expected properties
    ModuleActionProperties,

    /// An individual property
    ModuleActionProperty,

    /// Executor definition
    ModuleActionExecute,
}

impl From<SpecIdentity> for usize {
    fn from(value: SpecIdentity) -> Self {
        value as usize
    }
}

// Our entire schema is a composite of loader rules by way of NodeSpec sets
static RULES: &[&NodeSpec<'static, SpecIdentity>] = &[&phase::RULES, &module::RULES];

#[derive(Diagnostic, Error, Debug)]
#[diagnostic()]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] io::Error),

    // KDL raw parser issues
    #[error("Error parsing {}", src.name())]
    #[diagnostic()]
    KDL {
        #[source_code]
        src: NamedSource<String>,

        #[diagnostic_source]
        source: KdlError,
    },

    #[error("Syntax parsing")]
    #[diagnostic()]
    Syntax {
        #[source_code]
        src: NamedSource<String>,

        #[diagnostic_source(transparent)]
        source: bny_syntax::Error,
    },

    #[error("Module parsing")]
    #[diagnostic()]
    Module {
        #[source_code]
        src: NamedSource<String>,

        #[diagnostic_source(transparent)]
        source: module::Error,
    },

    #[error("unsupported node")]
    #[diagnostic()]
    UnsupportedID {
        #[source_code]
        src: NamedSource<String>,

        id: SpecIdentity,

        #[label("unsupported SpecIdentity node: {id:?}")]
        span: SourceSpan,
    },
}

impl BootstrapSpec {
    /// Load a bootstrap spec from the given path
    pub fn from_path(whence: &impl AsRef<Path>) -> Result<Self, Error> {
        let whence_path = whence.as_ref().to_string_lossy().to_string();
        let contents = fs::read_to_string(&whence_path)?;

        let source_code = NamedSource::new(whence_path.clone(), contents);

        let kdl_doc = KdlDocument::parse_v2(source_code.inner()).map_err(|e| Error::KDL {
            src: source_code.clone(),
            source: e,
        })?;

        BootstrapSpec::new(&source_code, &kdl_doc)
    }

    /// Load a bootstrap definition (into AST) from a valid KDL document
    /// using the correct procedural lingo.
    pub fn new(source: &NamedSource<String>, doc: &KdlDocument) -> Result<Self, Error> {
        let nodes = bny_syntax::process_kdl(doc, RULES).map_err(|e| Error::Syntax {
            src: source.clone(),
            source: e,
        })?;

        let mut phases = vec![];
        let mut modules = vec![];
        let mut symbols = SymbolTable::new();

        for node in nodes.into_iter() {
            match node.identity {
                SpecIdentity::Phase => {
                    phases.push(Phase::new(node));
                }
                SpecIdentity::Module => {
                    let module = Module::new(node, &mut symbols).map_err(|e| Error::Module {
                        src: source.clone(),
                        source: e,
                    })?;
                    modules.push(module);
                }
                _ => {
                    // essentially unimplemented()
                    return Err(Error::UnsupportedID {
                        src: source.clone(),
                        id: node.identity.clone(),
                        span: node.span(),
                    });
                }
            }
        }

        Ok(Self {
            phases,
            modules,
            _symbols: symbols,
        })
    }

    /// Access the underlying phases
    pub fn phases(&self) -> &[Phase] {
        self.phases.as_slice()
    }

    /// Access the modules
    pub fn modules(&self) -> &[Module] {
        self.modules.as_slice()
    }
}
