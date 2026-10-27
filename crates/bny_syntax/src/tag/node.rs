// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

use kdl::KdlIdentifier;

use crate::tag::Type;

/// A node can have only one attribute at most,
/// which can be used to mark it as a schema attribute
/// or a type restriction rule.
#[derive(Debug, Clone)]
pub enum NodeTag {
    /// The node is a schema attribute, ie `(barney)version`
    Schema,

    /// This node has a value type restriction
    TypeRestriction { restrict: Type },
}

impl TryInto<NodeTag> for &KdlIdentifier {
    type Error = super::TagError;

    fn try_into(self) -> Result<NodeTag, Self::Error> {
        match self.value() {
            "barney" | "bny" => Ok(NodeTag::Schema),
            "string" => Ok(NodeTag::TypeRestriction {
                restrict: Type::String,
            }),
            "list" => Ok(NodeTag::TypeRestriction {
                restrict: Type::List,
            }),
            mismatch => Err(super::TagError::UnknownTag(mismatch.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use kdl::KdlDocument;

    use crate::tag::Type;
    use crate::tag::node::NodeTag;

    #[test]
    fn test_schema() {
        let doc = "(barney)version 1".parse::<KdlDocument>().unwrap();
        let root = doc.nodes().first().unwrap();
        let t: NodeTag = root.ty().unwrap().try_into().expect("couldnt decode ty");
        assert!(matches!(t, NodeTag::Schema {}));
    }

    #[test]
    fn test_types() {
        let doc = "(string)node \"bob\"".parse::<KdlDocument>().unwrap();
        let root = doc.nodes().first().unwrap();
        let t: NodeTag = root.ty().unwrap().try_into().expect("couldnt decode ty");
        assert!(matches!(
            t,
            NodeTag::TypeRestriction {
                restrict: Type::String
            }
        ));
    }
}
