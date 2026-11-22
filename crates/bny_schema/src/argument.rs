// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

use std::marker::PhantomData;

use crate::{ArgumentSpec, AsBnyValueType, AsOptionalBnyValueType, BnyValueType, SchemaKey};

/// Argument definitions
#[derive(Debug)]
pub struct ArgumentDef<'a, K: SchemaKey<'a>> {
    _data: PhantomData<&'a K>,
    identity: K,
    kind: BnyValueType,
    required: bool,
    arg_kind: ArgumentKind,
}

/// Variadic or positional is supported
#[derive(Debug, Default, Clone)]
pub(crate) enum ArgumentKind {
    #[default]
    Positional,
    Variadic,
}

impl<'a, K> ArgumentDef<'a, K>
where
    K: SchemaKey<'a>,
{
    /// Construct a new ArgumentDef
    pub const fn positional<T: AsOptionalBnyValueType<'a>>(identity: K) -> Self {
        Self {
            _data: PhantomData,
            identity,
            kind: T::TYPE,
            required: T::REQUIRED,
            arg_kind: ArgumentKind::Positional,
        }
    }

    /// Variadic arguments - sentinal use ONLY (consume remaining arguments)
    pub const fn variadic<T: AsBnyValueType<'a>>(identity: K) -> Self {
        Self {
            _data: PhantomData,
            identity,
            kind: T::TYPE,
            required: false,
            arg_kind: ArgumentKind::Variadic,
        }
    }

    /// Erase to a spec
    pub(crate) fn spec(&self, local_index: usize) -> ArgumentSpec {
        ArgumentSpec {
            symbolic: self.identity.symbolic_id(),
            local_index,
            kind: self.kind,
            required: self.required,
            arg_kind: self.arg_kind.clone(),
        }
    }
}
