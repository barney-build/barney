// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

use thiserror::Error;

#[derive(Debug, Error)]
pub enum OccurrenceError {
    #[error("requires minimum occurrence of {0}, found {1}")]
    MinimumNotMet(u16, u16),

    #[error("maximum occurrence of {0}, {1} found")]
    MaximumExceeded(u16, u16),

    #[error("exact occurrence of {0}, {1} found")]
    ExactlyOnly(u16, u16),

    #[error("expected between {0} and {1} occurrences, found {2}")]
    RangeOnly(u16, u16, u16),
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Occurrence(OccurrenceKind);

#[derive(Debug, Clone, Copy, Default)]
enum OccurrenceKind {
    #[default]
    Unbounded,
    Exactly(u16),
    Minimum(u16),
    Between(u16, u16),
    Maximum(u16),
}

impl Occurrence {
    pub const fn unbounded() -> Self {
        Self(OccurrenceKind::Unbounded)
    }

    pub const fn exactly(n: u16) -> Self {
        Self(OccurrenceKind::Exactly(n))
    }

    pub const fn minimum(n: u16) -> Self {
        Self(OccurrenceKind::Minimum(n))
    }

    pub const fn between(mn: u16, mx: u16) -> Self {
        assert!(mn < mx, "minimum must be below maximum");
        Self(OccurrenceKind::Between(mn, mx))
    }

    pub const fn maximum(n: u16) -> Self {
        Self(OccurrenceKind::Maximum(n))
    }

    pub fn validate(&self, count: u8) -> Result<(), OccurrenceError> {
        let count = count as u16;
        match &self.0 {
            OccurrenceKind::Unbounded => Ok(()),
            OccurrenceKind::Exactly(n) => {
                (count == *n).ok_or(OccurrenceError::ExactlyOnly(*n, count))
            }
            OccurrenceKind::Minimum(m) => {
                (count >= *m).ok_or(OccurrenceError::MinimumNotMet(*m, count))
            }
            OccurrenceKind::Between(mn, mx) => {
                (count >= *mn && count <= *mx).ok_or(OccurrenceError::RangeOnly(*mn, *mx, count))
            }
            OccurrenceKind::Maximum(mx) => {
                (count <= *mx).ok_or(OccurrenceError::MaximumExceeded(*mx, count))
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::Occurrence;

    #[test]
    fn test_occurrence() {
        let rng1 = Occurrence::exactly(1);
        assert!(rng1.validate(12).is_err());
        assert!(rng1.validate(1).is_ok());

        let rng2 = Occurrence::minimum(2);
        assert!(rng2.validate(1).is_err());
        assert!(rng2.validate(2).is_ok());
        assert!(rng2.validate(56).is_ok());

        let rng3 = Occurrence::between(0, 4);
        assert!(rng3.validate(3).is_ok());
        assert!(rng3.validate(0).is_ok());
        assert!(rng3.validate(4).is_ok());
        assert!(rng3.validate(5).is_err());

        let rng4 = Occurrence::maximum(1);
        assert!(rng4.validate(0).is_ok());
        assert!(rng4.validate(1).is_ok());
        assert!(rng4.validate(2).is_err());

        let rng5 = Occurrence::unbounded();
        assert!(rng5.validate(0).is_ok());
        assert!(rng5.validate(240).is_ok());
    }
}
