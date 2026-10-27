// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

use kdl::KdlIdentifier;

#[derive(Debug, Clone)]
pub enum EntryTag {
    /// Bound to a static property (`(prop)`)
    Property,

    /// Local contextual variable (`(var)`)
    Variable,
}

impl TryInto<EntryTag> for &KdlIdentifier {
    type Error = super::TagError;

    fn try_into(self) -> Result<EntryTag, Self::Error> {
        match self.value() {
            "prop" => Ok(EntryTag::Property),
            "var" => Ok(EntryTag::Variable),
            mismatch => Err(super::TagError::UnknownTag(mismatch.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tag::entry::EntryTag;
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
