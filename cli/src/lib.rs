// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

pub mod build;

// TODO: Add proper codes for our app xD
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    Normal = 0,
    Abnormal,
}

impl From<ExitCode> for i32 {
    fn from(val: ExitCode) -> Self {
        val as i32
    }
}
