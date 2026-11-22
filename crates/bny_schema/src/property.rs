// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Property schema definition

use bny_syntax::BnyInterner;

use crate::{AsOptionalBnyValueType, BnyValueType, SchemaKey, SchemaName, spec::PropertySpec};
use std::marker::PhantomData;

/// Property schema
#[derive(Debug)]
pub struct PropertyDef<'a, K: SchemaKey<'a>> {
    // exists simply for our type erasure semantics
    _data: PhantomData<&'a K>,

    // Typically a `#[repr(u8)]` style enum ala SchemaKey trait
    identity: K,

    // Permitted value type
    kind: BnyValueType,

    // Mandatory property, ie `Option<T>` value access
    required: bool,
}

impl<'a, K> PropertyDef<'a, K>
where
    K: SchemaKey<'a>,
{
    pub const fn new<T: AsOptionalBnyValueType<'a>>(identity: K) -> Self {
        Self {
            _data: PhantomData,
            identity,
            kind: T::TYPE,
            required: T::REQUIRED,
        }
    }

    /// Return a type erased property spec
    pub(crate) fn spec(&self, intern: &mut BnyInterner, index: usize) -> PropertySpec {
        PropertySpec {
            id: match self.identity.name() {
                SchemaName::Explicit(name) => Some(intern.intern(name)),
                SchemaName::Dynamic => None,
            },
            symbolic: self.identity.symbolic_id(),
            local_index: index,
            kind: self.kind,
            required: self.required,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{BnyValueType, PropertyDef, SchemaKey, SchemaName};

    #[derive(Debug, Copy, Clone)]
    #[repr(u8)]
    pub enum MyProperties {
        SomeProp,
        OtherProp,
    }

    impl<'a> SchemaKey<'a> for MyProperties {
        fn symbolic_id(self) -> u8 {
            self as u8
        }

        fn name(&self) -> SchemaName<'a> {
            match self {
                MyProperties::SomeProp => SchemaName::Explicit("prop"),
                MyProperties::OtherProp => SchemaName::Explicit("prop2"),
            }
        }
    }

    static PROPERTIES: &[PropertyDef<MyProperties>] = &[
        PropertyDef::new::<&str>(MyProperties::SomeProp),
        PropertyDef::new::<Option<bool>>(MyProperties::OtherProp),
    ];

    #[test]
    fn props_simple() {
        assert!(matches!(PROPERTIES[0].kind, BnyValueType::String));
        assert!(matches!(PROPERTIES[1].kind, BnyValueType::Bool));
        assert_eq!(PROPERTIES[0].required, true);
        assert_eq!(PROPERTIES[1].required, false);
    }
}
