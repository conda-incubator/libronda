//! Version compare module, with useful static comparison methods.
//!
//! This module provides the `VersionCompare` struct, which provides many static functions, that are
//! useful for version comparison.

use super::comp_op::CompOp;
use super::version::Version;

/// The main library structure, which provides various static methods for easy version comparison.
///
/// This structure uses static methods only, and doesn't need to be constructed.
pub struct VersionCompare {}

impl VersionCompare {
    /// Compare two version number strings to each other.
    /// This compares version `a` to version `b`, and returns whether version `a` is greater, less
    /// or equal to version `b`.
    ///
    /// The two given version numbers must be valid, or an error will be returned.
    ///
    /// One of the following ok results may be returned:
    ///
    /// * `CompOp::Eq`
    /// * `CompOp::Lt`
    /// * `CompOp::Gt`
    ///
    /// # Examples
    ///
    /// ```
    /// use libronda::{CompOp, VersionCompare};
    ///
    /// // Compare version numbers
    /// assert_eq!(VersionCompare::compare("1.2.3", "1.2.3"), Ok(CompOp::Eq));
    /// assert_eq!(VersionCompare::compare("1.2.3", "1.2.4"), Ok(CompOp::Lt));
    /// assert_eq!(VersionCompare::compare("1", "0.1"), Ok(CompOp::Gt));
    /// ```
    pub fn compare(a: &str, b: &str) -> Result<CompOp, ()> {
        // Create version instances
        let a_ver = Version::from(a);
        let b_ver = Version::from(b);

        // Both version numbers must have been parsed
        if a_ver.is_err() || b_ver.is_err() {
            return Err(());
        }

        // Compare and return the result
        Ok(a_ver.unwrap().compare(&b_ver.unwrap()))
    }

    /// Compare two version number strings to each other and check whether the given comparison
    /// `operator` is valid.
    ///
    /// The two given version numbers must be valid, or an error will be returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use libronda::{CompOp, VersionCompare};
    ///
    /// // Compare version numbers
    /// assert!(VersionCompare::compare_to("1.2.3", "1.2.3", &CompOp::Eq).unwrap());
    /// assert!(VersionCompare::compare_to("1.2.3", "1.2.3", &CompOp::Le).unwrap());
    /// assert!(VersionCompare::compare_to("1.2.3", "1.2.4", &CompOp::Lt).unwrap());
    /// assert!(VersionCompare::compare_to("1", "0.1", &CompOp::Gt).unwrap());
    /// assert!(VersionCompare::compare_to("1", "0.1", &CompOp::Ge).unwrap());
    /// ```
    pub fn compare_to(a: &str, b: &str, operator: &CompOp) -> Result<bool, ()> {
        // Create version instances
        let a_ver = Version::from(a);
        let b_ver = Version::from(b);

        // Both version numbers must have been parsed
        if a_ver.is_err() || b_ver.is_err() {
            return Err(());
        }

        // Compare and return the result
        Ok(a_ver.unwrap().compare_to(&b_ver.unwrap(), &operator))
    }
}

#[cfg_attr(tarpaulin, skip)]
#[cfg(test)]
mod tests {
    use crate::CompOp;

    use crate::version::VersionCompare;
    use crate::version::Version;

    fn compare(a: &str, b: &str, operator: &CompOp) {
        match VersionCompare::compare(a, b) == Ok(operator.clone()) {
            false => panic!(),
            _ => {}
        }
    }
    parametrize_versions_set!(compare);

    fn compare_error(a: &str, b: &str, operator: &CompOp) {
        let result = VersionCompare::compare(a, b);

        if result.is_ok() {
            assert!(result != Ok(operator.clone()));
        }
    }
    parametrize_errors_set!(compare_error);

    fn compare_to(a: &str, b: &str, operator: &CompOp) {
        // Test
        assert!(VersionCompare::compare_to(a, b, operator).unwrap());

        // Make sure the inverse operator is not correct
        assert_eq!(
            VersionCompare::compare_to(a, b, &operator.opposite()).unwrap(),
            false
        );
    }
    parametrize_versions_set!(compare_to);

    fn compare_to_errors(a: &str, b: &str, operator: &CompOp) {
        let result = VersionCompare::compare_to(a, b, operator);

        if result.is_ok() {
            assert!(!result.unwrap())
        }
    }
    parametrize_errors_set!(compare_to_errors);
//
//        // Assert an exceptional case, compare to not equal
//        assert!(VersionCompare::compare_to("1.2.3", "1.2", CompOp::Ne).unwrap());
//    }
}
