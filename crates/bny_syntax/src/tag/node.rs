// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

use std::fmt::Display;

use kdl::KdlIdentifier;

use crate::tag::Type;

/// A node can have only one attribute at most,
/// which can be used to mark it as a schema attribute
/// or a type restriction rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeTag {
    /// The node is a schema attribute, ie `(barney)version`
    Schema,

    /// This node has a value type restriction
    TypeRestriction { restrict: Type },
}

impl<'a> TryFrom<&'a KdlIdentifier> for NodeTag {
    type Error = crate::Error;

    fn try_from(other: &'a KdlIdentifier) -> Result<Self, Self::Error> {
        match other.value() {
            "barney" | "bny" => Ok(Self::Schema),
            "string" => Ok(Self::TypeRestriction {
                restrict: Type::String,
            }),
            "list" => Ok(Self::TypeRestriction {
                restrict: Type::List,
            }),
            mismatch => Err(Self::Error::InvalidTag {
                span: other.span(),
                tag: mismatch.to_string(),
            }),
        }
    }
}

impl Display for NodeTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeTag::Schema => f.write_str("(barney)"),
            NodeTag::TypeRestriction { restrict } => match restrict {
                Type::String => f.write_str("(string)"),
                Type::List => f.write_str("(list)"),
            },
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
