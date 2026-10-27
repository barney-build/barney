// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Strong types can have tags associated with them
//! which also permit a referencing system
//! Internally they're still strings

use std::{fmt::Display, str::FromStr};

use kdl::KdlEntry;
use thiserror::Error;

mod node;

/// Arguments and indeed properties may be decorated with
/// a type, i.e a tag, to help control the the intended type
/// or indeed reference another argument or variable by the FQDN
/// accessor method.
#[derive(Debug)]
pub enum Tag {
    /// Local property `(prop)` reference
    PropertyBinding,

    /// Qualified variable `(var)` reference
    VariableBinding,

    Type(Type),
}

#[derive(Debug, Error)]
pub enum TagError {
    #[error("unknown tag: {0}")]
    UnknownTag(String),
}

impl FromStr for Tag {
    type Err = TagError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "prop" => Ok(Tag::PropertyBinding),
            "var" => Ok(Tag::VariableBinding),
            "string" => Ok(Tag::Type(Type::String)),
            "list" => Ok(Tag::Type(Type::List)),
            _ => Err(TagError::UnknownTag(s.to_string())),
        }
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let r = match self {
            Tag::PropertyBinding => "prop",
            Tag::VariableBinding => "var",
            Tag::Type(t) => match t {
                Type::String => "string",
                Type::List => "list",
            },
        };
        f.write_str(r)
    }
}

/// Explicitly set type tag
#[derive(Debug, Clone)]
pub enum Type {
    /// Accepts string data only
    String,

    /// Accepts (merges) list of string data only
    List,
}

/// A tagged value is a property or argument in our DSL
/// which may have been strongly tagged to be an argument or
/// variable, or is "just" a value (String)
/// Type enforcements are only actually used in schema building,
/// i.e action language
#[derive(Debug)]
pub enum TaggedValue {
    PropertyBinding(String),
    VariableBinding(String),
    Content(String),
}

impl<'a> TaggedValue {
    /// Process a KDL entry and use the type annotation to generated
    /// a correctly tagged value (argument/variable/content)
    pub fn process_kdl_entry(entry: &'a KdlEntry) -> Result<TaggedValue, super::Error> {
        let tag = if let Some(id) = entry.ty() {
            id.value()
                .parse::<Tag>()
                .map_err(|_| super::Error::InvalidTag {
                    span: id.span(),
                    tag: id.value().to_string(),
                })?
        } else {
            Tag::Type(Type::String)
        };

        let result = match tag {
            Tag::PropertyBinding => TaggedValue::PropertyBinding(entry.value().to_string()),
            Tag::VariableBinding => TaggedValue::VariableBinding(entry.value().to_string()),
            Tag::Type(_) => {
                TaggedValue::Content(entry.value().as_string().unwrap_or_default().to_owned())
            }
        };

        Ok(result)
    }
}

impl Display for TaggedValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TaggedValue::PropertyBinding(arg) => arg,
            TaggedValue::VariableBinding(var) => var,
            TaggedValue::Content(c) => c,
        };
        f.write_str(s)
    }
}
