// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Legal values + mappings from bny_syntax (i.e KDL) to BNY/Rusty  types

use std::{fmt::Display, num::TryFromIntError};

use bny_syntax::ProcessedValue;
use thiserror::Error;

mod private {
    pub trait Sealed {}
}

/// Legal value types in bny
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum BnyValueType {
    String,
    Bool,
    Float,

    Int32,
    Uint32,

    Int64,
    Uint64,

    #[default]
    Unknown,
}

impl Display for BnyValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id = match self {
            BnyValueType::String => "&str",
            BnyValueType::Bool => "bool",
            BnyValueType::Float => "float",
            BnyValueType::Int32 => "i32",
            BnyValueType::Uint32 => "u32",
            BnyValueType::Int64 => "i64",
            BnyValueType::Uint64 => "u64",
            BnyValueType::Unknown => "(unknown)",
        };
        f.write_str(id)
    }
}

/// Value parsing/transmute errors
#[derive(Error, Debug)]
pub enum ValueError {
    #[error("wrong type of value")]
    WrongType,

    #[error(transparent)]
    ConvertInt(#[from] TryFromIntError),
}

/// A locked down mapping of permitted base value types
pub trait AsBnyValueType<'a>: private::Sealed + Sized {
    const TYPE: BnyValueType;
    const REQUIRED: bool;

    fn from_processed_value(value: Option<&'a ProcessedValue>) -> Result<Self, ValueError>;
}

// String types
impl<'a> AsBnyValueType<'a> for &'a str {
    const TYPE: BnyValueType = BnyValueType::String;
    const REQUIRED: bool = true;

    fn from_processed_value(value: Option<&'a ProcessedValue>) -> Result<Self, ValueError> {
        match value.as_ref() {
            Some(ProcessedValue::String(v)) => Ok(v.as_str()),
            _ => Err(ValueError::WrongType),
        }
    }
}
impl private::Sealed for &str {}

// Boolean types
impl<'a> AsBnyValueType<'a> for bool {
    const TYPE: BnyValueType = BnyValueType::Bool;
    const REQUIRED: bool = true;

    fn from_processed_value(value: Option<&'a ProcessedValue>) -> Result<Self, ValueError> {
        match value.as_ref() {
            Some(ProcessedValue::Boolean(b)) => Ok(*b),
            _ => Err(ValueError::WrongType),
        }
    }
}
impl private::Sealed for bool {}

// float types
impl<'a> AsBnyValueType<'a> for f64 {
    const TYPE: BnyValueType = BnyValueType::Float;
    const REQUIRED: bool = true;

    fn from_processed_value(value: Option<&'a ProcessedValue>) -> Result<Self, ValueError> {
        match value.as_ref() {
            Some(ProcessedValue::Float(f)) => Ok(*f),
            _ => Err(ValueError::WrongType),
        }
    }
}
impl private::Sealed for f64 {}

// i32
impl<'a> AsBnyValueType<'a> for i32 {
    const TYPE: BnyValueType = BnyValueType::Int32;
    const REQUIRED: bool = true;

    fn from_processed_value(value: Option<&'a ProcessedValue>) -> Result<Self, ValueError> {
        match value.as_ref() {
            Some(ProcessedValue::Integer(i)) => Ok(i32::try_from(*i)?),
            _ => Err(ValueError::WrongType),
        }
    }
}
impl private::Sealed for i32 {}

// u32
impl<'a> AsBnyValueType<'a> for u32 {
    const TYPE: BnyValueType = BnyValueType::Uint32;
    const REQUIRED: bool = true;

    fn from_processed_value(value: Option<&'a ProcessedValue>) -> Result<Self, ValueError> {
        match value.as_ref() {
            Some(ProcessedValue::Integer(i)) => Ok(u32::try_from(*i)?),
            _ => Err(ValueError::WrongType),
        }
    }
}
impl private::Sealed for u32 {}

// i64
impl<'a> AsBnyValueType<'a> for i64 {
    const TYPE: BnyValueType = BnyValueType::Int64;
    const REQUIRED: bool = true;

    fn from_processed_value(value: Option<&'a ProcessedValue>) -> Result<Self, ValueError> {
        match value.as_ref() {
            Some(ProcessedValue::Integer(i)) => Ok(i64::try_from(*i)?),
            _ => Err(ValueError::WrongType),
        }
    }
}
impl private::Sealed for i64 {}

// u64
impl<'a> AsBnyValueType<'a> for u64 {
    const TYPE: BnyValueType = BnyValueType::Uint64;
    const REQUIRED: bool = true;

    fn from_processed_value(value: Option<&'a ProcessedValue>) -> Result<Self, ValueError> {
        match value.as_ref() {
            Some(ProcessedValue::Integer(i)) => Ok(u64::try_from(*i)?),
            _ => Err(ValueError::WrongType),
        }
    }
}

impl private::Sealed for u64 {}

// Option<T> types
impl<'a, T> AsBnyValueType<'a> for Option<T>
where
    T: AsBnyValueType<'a>,
{
    const TYPE: BnyValueType = T::TYPE;

    const REQUIRED: bool = false;

    fn from_processed_value(value: Option<&'a ProcessedValue>) -> Result<Self, ValueError> {
        match value.as_ref() {
            Some(s) => Ok(Some(T::from_processed_value(Some(s))?)),
            None => Ok(None),
        }
    }
}
impl<T> private::Sealed for Option<T> {}
