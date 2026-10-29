// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

use std::fmt::Display;

use kdl::KdlIdentifier;

#[derive(Debug, Clone)]
pub enum EntryTag {
    /// Bound to a static property (`(prop)`)
    Property,

    /// Local contextual variable (`(var)`)
    Variable,
}

impl Display for EntryTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntryTag::Property => f.write_str("(prop)"),
            EntryTag::Variable => f.write_str("(var)"),
        }
    }
}

impl<'a> TryFrom<&'a KdlIdentifier> for EntryTag {
    type Error = crate::Error;

    fn try_from(other: &'a KdlIdentifier) -> Result<Self, Self::Error> {
        match other.value() {
            "prop" => Ok(Self::Property),
            "var" => Ok(Self::Variable),
            mismatch => Err(Self::Error::InvalidTag {
                span: other.span(),
                tag: mismatch.to_owned(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tag::EntryTag;
    use kdl::KdlDocument;

    #[test]
    fn test_prop() {
        let doc = "some (prop)thing".parse::<KdlDocument>().unwrap();
        let root = doc.nodes().first().unwrap();
        let child = root.entries().first().unwrap();
        let t: EntryTag = child.ty().unwrap().try_into().expect("couldn't decode ty");
        assert!(matches!(t, EntryTag::Property {}));
    }

    #[test]
    fn test_var() {
        let doc = "some (var)thing".parse::<KdlDocument>().unwrap();
        let root = doc.nodes().first().unwrap();
        let child = root.entries().first().unwrap();
        let t: EntryTag = child.ty().unwrap().try_into().expect("couldn't decode ty");
        assert!(matches!(t, EntryTag::Variable {}));
    }
}
