// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Barney syntax value support

use kdl::KdlValue;

/// A processed value within an entry, either a positional argument or a property pair's value
/// Note: Integer support directly maps to kdl.dev so we can later add type annotations for smaller
/// shortage if needed.
#[derive(Debug)]
pub enum ProcessedValue {
    Boolean(bool),
    Float(f64),
    String(String),
    Integer(i128),
    Null,
}

impl ProcessedValue {
    /// Same as KdlValue, valid string or nay
    pub fn as_string(&self) -> Option<&str> {
        match self {
            ProcessedValue::String(s) => Some(s),
            _ => None,
        }
    }
}

impl<'a> From<&'a KdlValue> for ProcessedValue {
    fn from(value: &'a KdlValue) -> Self {
        match value {
            KdlValue::String(s) => Self::String(s.clone()),
            KdlValue::Integer(i) => Self::Integer(*i),
            KdlValue::Float(f) => Self::Float(*f),
            KdlValue::Bool(b) => Self::Boolean(*b),
            KdlValue::Null => Self::Null,
        }
    }
}
