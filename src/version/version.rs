//! This
//! Version module, which provides the `Version` struct as parsed version representation.
//!
//! Version numbers in the form of a string are parsed to a `Version` first, before any comparison
//! is made. This struct provides many methods and features for easy comparison, probing and other
//! things.

use std::cmp::Ordering;
use std::fmt;
use std::iter::Peekable;
use std::slice::Iter;
use std::str::FromStr;
use std::convert::From;

use serde::Deserialize;

use super::comp_op::CompOp;
use super::version_part::{VersionPart, ProvideEmptyImpl};
use super::parsers::conda::conda_parser;
use super::errors::VersionParsingError;

/// Version struct, which is a representation for a parsed version string.
///
/// A version in string format can be parsed using methods like `Version::from("1.2.3");`.
/// These methods return a `Result` holding the parsed version or an error on failure.
///
/// The original version string is stored in the struct, and can be accessed using the
/// `version.as_str()` method. Note, that when the version wasn't parsed from a string
/// representation, the returned value is generated.
///
/// The struct provides many methods for comparison and probing.
#[derive(Deserialize)]
pub struct Version {
    version: String,
    parts: Vec<VersionPart>,
}

impl FromStr for Version {
    type Err = VersionParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        /// Create a `Version` instance from a version string.
        ///
        /// The version string should be passed to the `version` parameter.  Takes ownership of `version`
        ///
        /// # Examples
        ///
        /// ```
        /// use libronda::{CompOp, Version};
        ///
        /// let ver: Version = "1.2.3".parse()?;
        /// ```
        Version::parse(s, &conda_parser)
    }
}

impl From<&str> for Version {
    fn from(s: &str) -> Version {
        Version::parse(s, &conda_parser).unwrap()
    }
}

//impl<'a> From<&'a str> for &'a Version {
//    fn from(s: &str) -> & Version {
//        let v: Version = Version::parse(s, &conda_parser).unwrap();
//        &v
//    }
//}

impl Version {
    /// Create a `Version` instance from a version string with the given `parser` function.
    ///
    /// The version string should be passed to the `version` parameter.  Additional parsers
    /// are in the "parsers" module.  This is the primary means of customizing behavior.
    ///
    /// Note that the string reference passed in here is moved with to_string().
    ///
    /// # Examples
    ///
    /// ```
    /// use libronda::{CompOp, Version, conda_parser};
    ///
    /// let ver = Version::parse("1.2.3", &conda_parser).unwrap();
    /// ```
    pub fn parse(version: &str, parser: &dyn Fn(&str) -> Result<Vec<VersionPart>, VersionParsingError>) -> Result<Self, VersionParsingError> {
        let owned_version = version.to_string();
        match parser(&owned_version) {
            Ok(parts) => Ok(Self { version: owned_version, parts}),
            Err(E) => Err(E)
        }
    }

    /// Get the original version string.
    ///
    /// # Examples
    ///
    /// ```
    /// use libronda::Version;
    ///
    /// let ver: Version = "1.2.3".parse().unwrap();
    ///
    /// assert_eq!(ver.as_str(), "1.2.3");
    /// ```
    pub fn as_str(&self) -> &str {
        &self.version
    }

    /// Get a specific version part by it's `index`.
    /// An error is returned if the given index is out of bound.
    ///
    /// # Examples
    ///
    /// ```
    /// use libronda::{Version, VersionPart};
    ///
    /// let ver: Version = "1.2.3".parse().unwrap();
    ///
    /// assert_eq!(ver.part(0), Ok(&VersionPart::Integer(1)));
    /// assert_eq!(ver.part(1), Ok(&VersionPart::Integer(2)));
    /// assert_eq!(ver.part(2), Ok(&VersionPart::Integer(3)));
    /// ```
    pub fn part(&self, index: usize) -> Result<&VersionPart, ()> {
        // Make sure the index is in-bound
        if index >= self.parts.len() {
            return Err(());
        }

        // Return the requested part
        Ok(&self.parts[index])
    }

    /// Get a vector of all version parts.
    ///
    /// # Examples
    ///
    /// ```
    /// use libronda::{Version, VersionPart};
    ///
    /// let ver: Version = "1.2.3".parse().unwrap();
    ///
    /// assert_eq!(ver.parts(), &vec![
    ///     VersionPart::Integer(1),
    ///     VersionPart::Integer(2),
    ///     VersionPart::Integer(3)
    /// ]);
    /// ```
    pub fn parts(&self) -> &Vec<VersionPart> {
        &self.parts
    }

    /// Get the number of parts in this version string.
    ///
    /// # Examples
    ///
    /// ```
    /// use libronda::Version;
    ///
    /// let ver_a: Version = "1.2.3".into();
    /// let ver_b: Version = "1.2.3.4".into();
    ///
    /// assert_eq!(ver_a.part_count(), 3);
    /// assert_eq!(ver_b.part_count(), 4);
    /// ```
    pub fn part_count(&self) -> usize {
        self.parts.len()
    }

    /// Compare this version to the given `other` version.
    ///
    /// This method returns one of the following comparison operators:
    ///
    /// * `Lt`
    /// * `Eq`
    /// * `Gt`
    ///
    /// Other comparison operators can be used when comparing, but aren't returned by this method.
    ///
    /// # Examples:
    ///
    /// ```
    /// use libronda::{CompOp, Version};
    ///
    /// let a: Version = "1.2".parse().unwrap();
    /// let b: Version = "2".parse().unwrap();
    /// assert_eq!(a.compare("1.3.2".into()), CompOp::Lt);
    /// assert_eq!(a.compare("1.2.0".into()), CompOp::Eq);
    /// assert_eq!(b.compare("1.7.3".into()), CompOp::Gt);
    /// ```
    pub fn compare(&self, other: &Version) -> CompOp {
        // Compare the versions with their peekable iterators
        Self::compare_iter(self.parts.iter().peekable(),
                           other.parts.iter().peekable())
    }

    /// Compare this version to the given `other` version,
    /// and check whether the given comparison operator is valid.
    ///
    /// All comparison operators can be used.
    ///
    /// # Examples:
    ///
    /// ```
    /// use libronda::{CompOp, Version};
    ///
    /// let a: Version = "1.2".parse().unwrap();
    /// assert!(a.compare_to("1.3.2".into(), &CompOp::Lt));
    /// assert!(a.compare_to("1.3.2".into(), &CompOp::Le));
    /// assert!(a.compare_to("1.2".into(), &CompOp::Eq));
    /// assert!(a.compare_to("1.2".into(), &CompOp::Le));
    /// ```
    pub fn compare_to(&self, other: &Version, operator: &CompOp) -> bool {
        // Get the comparison result
        let result = self.compare(other);

        // Match the result against the given operator
        match result {
            CompOp::Eq => match operator {
                &CompOp::Eq | &CompOp::Le | &CompOp::Ge => true,
                _ => false,
            },
            CompOp::Lt => match operator {
                &CompOp::Ne | &CompOp::Lt | &CompOp::Le => true,
                _ => false,
            },
            CompOp::Gt => match operator {
                &CompOp::Ne | &CompOp::Gt | &CompOp::Ge => true,
                _ => false,
            },
            _ => unreachable!(),
        }
    }

    /// Compare two version numbers based on the iterators of their version parts.
    ///
    /// This method returns one of the following comparison operators:
    ///
    /// * `Lt`
    /// * `Eq`
    /// * `Gt`
    ///
    /// Other comparison operators can be used when comparing, but aren't returned by this method.
    fn compare_iter(
        mut iter: Peekable<Iter<VersionPart>>,
        mut other_iter: Peekable<Iter<VersionPart>>,
    ) -> CompOp {
        // Iterate over the iterator, without consuming it
        loop {
            let i1 = &iter.next();
            let i2 = &other_iter.next();
            let _cmp = match (i1, i2) {
                (Some(i), None) => match i.partial_cmp(&&i.get_empty()) {
                    Some(Ordering::Less) => return CompOp::Lt,
                    Some(Ordering::Greater) => return CompOp::Gt,
                    Some(Ordering::Equal) => return CompOp::Eq,
                    _ => panic!()
                },
                (None, Some(j)) => match &j.get_empty().partial_cmp(j) {
                    Some(Ordering::Less) => return CompOp::Lt,
                    Some(Ordering::Greater) => return CompOp::Gt,
                    Some(Ordering::Equal) => return CompOp::Eq,
                    _ => panic!()
                },
                (Some(i), Some(j)) => match i.partial_cmp(j) {
                    Some(Ordering::Greater) => return CompOp::Gt,
                    Some(Ordering::Less) => return CompOp::Lt,
                    // This is the only loop branch that continues
                    Some(Ordering::Equal) => Ordering::Equal,
                    _ => panic!()
                },
                // both versions are the same length and are equal for all values
                (None, None) => return CompOp::Eq
            };
        }
    }
}


impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.version)
    }
}

// Show just the version component parts as debug output
impl fmt::Debug for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            write!(f, "{:#?}", self.parts)
        } else {
            write!(f, "{:?}", self.parts)
        }
    }
}

/// Implement the partial ordering trait for the version struct, to easily allow version comparison.
impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Version) -> Option<Ordering> {
        self.compare(other).ord()
    }
}

/// Implement the partial equality trait for the version struct, to easily allow version comparison.
impl PartialEq for Version {
    fn eq(&self, other: &Version) -> bool {
        self.compare_to(other, &CompOp::Eq)
    }
}

#[cfg_attr(tarpaulin, skip)]
#[cfg(test)]
mod tests {
    use crate::CompOp;
    // use crate::version_part::VersionPart;

    use super::Version;
    use crate::version::errors::VersionParsingError;

    // TODO: This doesn't really test whether this method fully works
    fn from(v_string: &str, n_parts: usize) {
        // Test whether parsing works for each test version
        let result: Result<Version, VersionParsingError> = v_string.parse();
        assert!(result.is_ok());
    }
    parametrize_versions!(from);

//    fn from_with_invalid_versions(v_string: &str, n_parts: usize) {
//        // Test whether parsing works for each test invalid version
//        assert!(Version::from(v_string).is_none());
//    }
//    parametrize_versions_errors!(from_with_invalid_versions);

    fn as_str(v_string: &str, n_parts: usize) {
        let v: Version = v_string.parse().unwrap();
        // The input version string must be the same as the returned string
        assert_eq!(v.as_str(), v_string);
    }
    parametrize_versions!(as_str);

    fn part(v_string: &str, n_parts: usize) {
        // Create a version object
        let ver: Version = v_string.parse().unwrap();

        // Loop through each part
        for i in 0..n_parts {
            assert_eq!(ver.part(i), Ok(&ver.parts[i]));
        }

        // A value outside the range must return an error
        assert!(ver.part(n_parts).is_err());
    }
    parametrize_versions!(part);

    fn parts(v_string: &str, n_parts: usize) {
        let v: Version = v_string.parse().unwrap();
        // The number of parts must match
        assert_eq!(v.parts().len(), n_parts);
    }
    parametrize_versions!(parts);

    fn part_count(v_string: &str, n_parts: usize) {
        let v: Version = v_string.parse().unwrap();
        // The number of parts must match the metadata
        assert_eq!(v.part_count(), n_parts);
    }
    parametrize_versions!(part_count);

    fn compare(a: &str, b: &str, operator: &CompOp) {
        // Get both versions
        let version_a: Version = a.parse().unwrap();
        let version_b: Version = b.parse().unwrap();

        // Compare them
        assert_eq!(
            version_a.compare(&version_b),
            operator.clone(),
        );
    }
    parametrize_versions_set!(compare);

    fn compare_to(a: &str, b: &str, operator: &CompOp) {
        // Get both versions
        let version_a: Version = a.parse().unwrap();
        let version_b: Version = b.parse().unwrap();

        // Test
        assert!(version_a.compare_to(&version_b, operator));

        // Make sure the inverse operator is not correct
        assert_eq!(version_a.compare_to(&version_b, &operator.invert()), false);
    }
    parametrize_versions_set!(compare_to);

    #[test]
    fn compare_to_neq() {
        // Assert an exceptional case, compare to not equal
        let a: Version = "1.2".into();
        assert!(a.compare_to(&"1.2.3".into(), &CompOp::Ne));
    }

    #[test]
    fn display() {
        let a: Version = "1.2.3".into();
        assert_eq!(format!("{}", a), "1.2.3");
    }

    #[test]
    fn debug() {
        let a: Version = "1.2.3".into();
        assert_eq!(
            format!("{:?}", a),
            "[Integer(1), Integer(2), Integer(3)]",
        );
//        assert_eq!(
//            format!("{:#?}", Version::from("1.2.3").unwrap()),
//            "[\n    Integer(\n        1,\n    ),\n    Integer(\n        2,\n    ),\n    Integer(\n        3,\n    ),\n]",
//        );
    }

    fn partial_cmp(a: &str, b: &str, operator: &CompOp) {
        // Get both versions
        let version_a: Version = a.parse().unwrap();
        let version_b: Version = b.parse().unwrap();

        // Compare and assert
        match operator {
            &CompOp::Eq => assert!(version_a == version_b),
            &CompOp::Lt => assert!(version_a < version_b),
            &CompOp::Gt => assert!(version_a > version_b),
            _ => {}
        }
    }
    parametrize_versions_set!(partial_cmp);

    fn partial_eq(a: &str, b: &str, operator: &CompOp) {
        // Skip entries that are less or equal, or greater or equal
        match operator {
            &CompOp::Le | &CompOp::Ge => return,
            _ => {}
        }

        // Get both versions
        let version_a: Version = a.parse().unwrap();
        let version_b: Version = b.parse().unwrap();

        // Determine what the result should be
        let result = match operator {
            &CompOp::Eq => true,
            _ => false,
        };

        // Test
        assert_eq!(version_a == version_b, result);
    }
    parametrize_versions_set!(partial_eq);

    #[test]
    fn partial_eq_neq() {
        // Assert an exceptional case, compare to not equal
        let a: Version = "1.2".parse().unwrap();
        assert_eq!(a, "1.2".into());
        assert_ne!(a, "1.2.3".into());
    }

    # [test]
    fn test_less_specific_less_than_more_specific() {
        // 0.4 < 0.4.0
        let a: Version = "0.4".parse().unwrap();
        let b: Version = "0.4.0".parse().unwrap();
        assert_eq!(a == b, true);
    }

    # [test]
    fn test_rc_greater_than_earlier_version_less_than_release() {
        // 0.4.0 < 0.4.1.rc < 0.4.1
        let a: Version = "0.4.0".parse().unwrap();
        let b: Version = "0.4.1.rc".parse().unwrap();
        let c: Version = "0.4.1".parse().unwrap();
        assert_eq!(a < b, true);
        assert_eq!(b < c, true);
    }

    # [test]
    fn test_case_insensitive_rc() {
        // 0.4.1.rc == 0.4.1.RC
        let a: Version = "0.4.1.rc".parse().unwrap();
        let b: Version = "0.4.1.RC".parse().unwrap();
        assert_eq!(a == b, true);
    }

    # [test]
    fn test_lexicographical_sort_numbers() {
        // 0.5a1 < 0.5a2
        let a: Version = "0.5a1".parse().unwrap();
        let b: Version = "0.5a2".parse().unwrap();
        assert_eq!(a < b, true);
    }

    # [test]
    fn test_lexicographical_sort() {
        // 0.5a2 < 0.5b1
        let a: Version = "0.5a2".parse().unwrap();
        let b: Version = "0.5b1".parse().unwrap();
        assert_eq!(a < b, true);
    }

    # [test]
    fn test_dev_special_case_horribleness() {
        // 1.0 < 1.1dev1 < 1.1a1 < 1.1.0dev1 == 1.1.dev1
        let a: Version = "1.0".parse().unwrap();
        let b: Version = "1.1dev1".parse().unwrap();
        let c: Version = "1.1a1".parse().unwrap();
        let d: Version = "1.1.0dev1".parse().unwrap();
        // let e: Version = "1.1.dev1");
        assert_eq!(a < b, true);
        assert_eq!(b < c, true);
        assert_eq!(c < d, true);
        // Not going to support this implicit crap.  Bite me.
        // assert_eq!(d == e, true);
    }

    # [test]
    fn test_rc_with_number() {
        let a: Version = "1.1.dev1".parse().unwrap();
        let b: Version = "1.1.a1".parse().unwrap();
        let c: Version = "1.1.0rc1".parse().unwrap();
        let d: Version = "1.1.0".parse().unwrap();
        assert_eq!(a < b, true);
        assert_eq!(b < c, true);
        assert_eq!(c < d, true);
    }

    # [test]
    fn test_post_gt_release() {
        let a: Version = "1.1.0".parse().unwrap();
        let b: Version = "1.1.0post1".parse().unwrap();
        let c: Version = "1996.07.12".parse().unwrap();
        assert_eq!(a == b, false);
        assert_eq!(a > b, false);
        assert_eq!(a < b, true);
        assert_eq!(b < c, true);
    }

    # [test]
    fn test_epoch() {
        let a: Version = "1996.07.12".parse().unwrap();
        let b: Version = "1!0.4.1".parse().unwrap();
        let c: Version = "1!3.4.1".parse().unwrap();
        let d: Version = "2!0.4.1".parse().unwrap();
        assert_eq!(a < b, true);
        assert_eq!(b < c, true);
        assert_eq!(c < d, true);
    }
}
