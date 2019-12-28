//! Version part module.
//!
//! A module that provides the `VersionPart` enum, with the specification of all available version
//! parts. Each version string is broken down into these version parts when being parsed to a
//! `Version`.

use std::cmp::Ordering;
use std::fmt;

use serde::Deserialize;

use super::custom_parts::pep440::PEP440String;
use std::fmt::{Debug, Display};

#[derive(Deserialize, Clone)]
pub enum VersionPart {
    Epoch(i16),
    Integer(i32),
    LexicographicString(String),
    PEP440String(PEP440String),
    Empty,
}

pub trait ProvideEmptyImpl{
    fn get_empty(&self) -> VersionPart;
}

impl ProvideEmptyImpl for VersionPart {
    fn get_empty(&self) -> VersionPart {
        match self {
            VersionPart::Epoch(_i) => VersionPart::Epoch(0),
            VersionPart::Integer(_i) => VersionPart::Integer(0),
            VersionPart::LexicographicString(_i) => VersionPart::LexicographicString(String::new()),
            VersionPart::PEP440String(_i) => VersionPart::PEP440String(PEP440String::empty()),
            VersionPart::Empty => VersionPart::Empty
        }
    }
}

impl Debug for VersionPart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VersionPart::Epoch(_i) => write!(f, "Epoch({})", _i),
            VersionPart::Integer(_i) => write!(f, "Integer({})", _i),
            VersionPart::LexicographicString(_i) => write!(f, "LexicographicString({})", _i),
            VersionPart::PEP440String(_i) => write!(f, "PEP440String({})", _i),
            VersionPart::Empty => write!(f, "Empty"),
        }
    }
}

impl Display for VersionPart {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl PartialOrd for VersionPart {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (VersionPart::Epoch(a), VersionPart::Epoch(b)) => a.partial_cmp(b),
            (VersionPart::Integer(a), VersionPart::Integer(b)) => a.partial_cmp(b),
            (VersionPart::LexicographicString(a), VersionPart::LexicographicString(b)) => a.partial_cmp(b),
            (VersionPart::PEP440String(a), VersionPart::PEP440String(b)) => a.partial_cmp(b),
            // Match simple position in the list, but reverse it because things at the top are higher
            _ => Some(match self {
                VersionPart::Epoch(_a) => 0,
                VersionPart::Integer(_a) => 1,
                VersionPart::LexicographicString(_a) => 2,
                VersionPart::PEP440String(_a) => 3,
                VersionPart::Empty => 4,
            }.partial_cmp(
                match other {
                    VersionPart::Epoch(_a) => &0,
                    VersionPart::Integer(_a) => &1,
                    VersionPart::LexicographicString(_a) => &2,
                    VersionPart::PEP440String(_a) => &3,
                    VersionPart::Empty => &4,
                }
            ).unwrap().reverse())
        }
    }
}

impl PartialEq for VersionPart {
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other) == Some(Ordering::Equal)
    }
}

#[cfg_attr(tarpaulin, skip)]
#[cfg(test)]
mod tests {
    use crate::version::VersionPart;

    #[test]
    fn epoch_compare() {
        assert_eq!(VersionPart::Epoch(0), VersionPart::Epoch(0));
        assert!(VersionPart::Epoch(0) < VersionPart::Epoch(1));
        // epoch of any value trumps integer (priority)
        // assert!(VersionPart::Epoch(value: 0) > VersionPart::Integer(value: 1);
        // assert!(Version::Epoch{0} > Version::String{"abc"});
    }

    #[test]
    fn cross_type_compare() {
        assert!(VersionPart::Epoch(0) > VersionPart::Integer(1));
    }
}
