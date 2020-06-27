//! Module with all supported comparison operators.
//!
//! This module provides an enum with all comparison operators that can be used with this library.
//! The enum provides various useful helper functions to inverse or flip an operator.
//!
//! Methods like `CompOp::from_sign(">");` can be used to get a comparison operator by it's logical
//! sign from a string.

use std::cmp::Ordering;

/// Enum of supported comparison operators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompOp {
    /// Equal (`==`, `=`).
    /// When version `A` is equal to `B`.
    Eq,

    /// Not equal (`!=`, `!`, `<>`).
    /// When version `A` is not equal to `B`.
    Ne,

    /// Less than (`<`).
    /// When version `A` is less than `B` but not equal.
    Lt,

    /// Less or equal (`<=`).
    /// When version `A` is less than or equal to `B`.
    Le,

    /// Greater or equal (`>=`).
    /// When version `A` is greater than or equal to `B`.
    Ge,

    /// Greater than (`>`).
    /// When version `A` is greater than `B` but not equal.
    Gt,

    /// StartsWith (`=`).
    /// When version `B` completely matches the first part of `A`
    StartsWith,

    /// NotStartsWith (`!=startswith`).
    /// When the version `B` does not completely match the first part of `A`
    NotStartsWith,

    /// Compatible (`~=`).
    /// PEP 440 compatible release, https://www.python.org/dev/peps/pep-0440/#compatible-release
    /// For V.N,
    /// >=V.N, == V.*
    /// Generally interpreted in Conda as
    /// >=V.N, <{V+1}
    Compatible,

    /// Incompatible (`!~=`).
    /// Opposite of PEP 440 compatible release, https://www.python.org/dev/peps/pep-0440/#compatible-release
    /// For V.N,
    /// <V.N || != V.*
    Incompatible,
}

impl CompOp {
    /// Get a comparison operator by it's sign.
    /// Whitespaces are stripped from the sign string.
    /// An error is returned if the sign isn't recognized.
    ///
    /// The following signs are supported:
    ///
    /// * `==` -> `Eq`
    /// * `!=` -> `Ne`
    /// * `< ` -> `Lt`
    /// * `<=` -> `Le`
    /// * `>=` -> `Ge`
    /// * `>` -> `Gt`
    /// * `=` -> `StartsWith`
    /// * `!=startswith ` -> `NotStartsWith`
    /// * `~=` -> `Compatible`
    /// * `!~=` -> `Incompatible`
    ///
    /// # Examples
    ///
    /// ```
    /// use ronda::CompOp;
    ///
    /// assert_eq!(CompOp::from_sign("=="), Ok(CompOp::Eq));
    /// assert_eq!(CompOp::from_sign("<"), Ok(CompOp::Lt));
    /// assert_eq!(CompOp::from_sign("  >=   "), Ok(CompOp::Ge));
    /// assert!(CompOp::from_sign("*").is_err());
    /// ```
    pub fn from_sign(sign: &str) -> Result<CompOp, ()> {
        match sign.trim().as_ref() {
            "==" => Ok(CompOp::Eq),
            "!=" => Ok(CompOp::Ne),
            "<" => Ok(CompOp::Lt),
            "<=" => Ok(CompOp::Le),
            ">=" => Ok(CompOp::Ge),
            ">" => Ok(CompOp::Gt),
            "=" => Ok(CompOp::StartsWith),
            "!=startswith" => Ok(CompOp::NotStartsWith),
            "~=" => Ok(CompOp::Compatible),
            "!~=" => Ok(CompOp::Incompatible),
            _ => Err(()),
        }
    }

    /// Get a comparison operator by it's name.
    /// Names are case-insensitive, and whitespaces are stripped from the string.
    /// An error is returned if the name isn't recognized.
    ///
    /// # Examples
    ///
    /// ```
    /// use ronda::CompOp;
    ///
    /// assert_eq!(CompOp::from_name("eq"), Ok(CompOp::Eq));
    /// assert_eq!(CompOp::from_name("lt"), Ok(CompOp::Lt));
    /// assert_eq!(CompOp::from_name("  Ge   "), Ok(CompOp::Ge));
    /// assert!(CompOp::from_name("abc").is_err());
    /// ```
    pub fn from_name(sign: &str) -> Result<CompOp, ()> {
        match sign.trim().to_lowercase().as_ref() {
            "eq" => Ok(CompOp::Eq),
            "ne" => Ok(CompOp::Ne),
            "lt" => Ok(CompOp::Lt),
            "le" => Ok(CompOp::Le),
            "ge" => Ok(CompOp::Ge),
            "gt" => Ok(CompOp::Gt),
            "startswith" => Ok(CompOp::StartsWith),
            "notstartswith" => Ok(CompOp::NotStartsWith),
            "compatible" => Ok(CompOp::Compatible),
            "incompatible" => Ok(CompOp::Incompatible),
            _ => Err(()),
        }
    }

    /// Get the comparison operator from Rusts `Ordering` enum.
    ///
    /// The following comparison operators are returned:
    ///
    /// * `Ordering::Less` -> `Lt`
    /// * `Ordering::Equal` -> `Eq`
    /// * `Ordering::Greater` -> `Gt`
    pub fn from_ord(ord: Ordering) -> CompOp {
        match ord {
            Ordering::Less => CompOp::Lt,
            Ordering::Equal => CompOp::Eq,
            Ordering::Greater => CompOp::Gt,
        }
    }

    /// Get the name of this comparison operator.
    ///
    /// # Examples
    ///
    /// ```
    /// use ronda::CompOp;
    ///
    /// assert_eq!(CompOp::Eq.name(), "eq");
    /// assert_eq!(CompOp::Lt.name(), "lt");
    /// assert_eq!(CompOp::Ge.name(), "ge");
    /// ```
    pub fn name(&self) -> &str {
        match self {
            &CompOp::Eq => "eq",
            &CompOp::Ne => "ne",
            &CompOp::Lt => "lt",
            &CompOp::Le => "le",
            &CompOp::Ge => "ge",
            &CompOp::Gt => "gt",
            &CompOp::StartsWith => "startswith",
            &CompOp::NotStartsWith => "notstartswith",
            &CompOp::Compatible => "compatible",
            &CompOp::Incompatible => "incompatible",
        }
    }

    /// Covert to the inverted comparison operator.
    ///
    /// This uses the following bidirectional rules:
    ///
    /// * `Eq` <-> `Ne`
    /// * `Lt` <-> `Ge`
    /// * `Le` <-> `Gt`
    ///
    /// # Examples
    ///
    /// ```
    /// use ronda::CompOp;
    ///
    /// assert_eq!(CompOp::Eq.as_inverted(), CompOp::Ne);
    /// assert_eq!(CompOp::Lt.as_inverted(), CompOp::Ge);
    /// assert_eq!(CompOp::Gt.as_inverted(), CompOp::Le);
    /// ```
    pub fn as_inverted(self) -> Self {
        self.invert()
    }

    /// Get the inverted comparison operator.
    ///
    /// This uses the following bidirectional rules:
    ///
    /// * `Eq` <-> `Ne`
    /// * `Lt` <-> `Ge`
    /// * `Le` <-> `Gt`
    ///
    /// # Examples
    ///
    /// ```
    /// use ronda::CompOp;
    ///
    /// assert_eq!(CompOp::Eq.invert(), CompOp::Ne);
    /// assert_eq!(CompOp::Lt.invert(), CompOp::Ge);
    /// assert_eq!(CompOp::Gt.invert(), CompOp::Le);
    /// ```
    pub fn invert(&self) -> Self {
        match self {
            &CompOp::Eq => CompOp::Ne,
            &CompOp::Ne => CompOp::Eq,
            &CompOp::Lt => CompOp::Ge,
            &CompOp::Le => CompOp::Gt,
            &CompOp::Ge => CompOp::Lt,
            &CompOp::Gt => CompOp::Le,
            &CompOp::StartsWith => CompOp::NotStartsWith,
            &CompOp::NotStartsWith => CompOp::StartsWith,
            &CompOp::Compatible => CompOp::Incompatible,
            &CompOp::Incompatible => CompOp::Compatible,
        }
    }

    /// Convert to the opposite comparison operator.
    ///
    /// This uses the following bidirectional rules:
    ///
    /// * `Eq` <-> `Ne`
    /// * `Lt` <-> `Gt`
    /// * `Le` <-> `Ge`
    ///
    /// # Examples
    ///
    /// ```
    /// use ronda::CompOp;
    ///
    /// assert_eq!(CompOp::Eq.as_opposite(), CompOp::Ne);
    /// assert_eq!(CompOp::Lt.as_opposite(), CompOp::Gt);
    /// assert_eq!(CompOp::Ge.as_opposite(), CompOp::Le);
    /// ```
    pub fn as_opposite(self) -> Self {
        self.opposite()
    }

    /// Get the opposite comparison operator.
    ///
    /// This uses the following bidirectional rules:
    ///
    /// * `Eq` <-> `Ne`
    /// * `Lt` <-> `Gt`
    /// * `Le` <-> `Ge`
    ///
    /// # Examples
    ///
    /// ```
    /// use ronda::CompOp;
    ///
    /// assert_eq!(CompOp::Eq.opposite(), CompOp::Ne);
    /// assert_eq!(CompOp::Lt.opposite(), CompOp::Gt);
    /// assert_eq!(CompOp::Ge.opposite(), CompOp::Le);
    /// ```
    pub fn opposite(&self) -> Self {
        match self {
            &CompOp::Eq => CompOp::Ne,
            &CompOp::Ne => CompOp::Eq,
            &CompOp::Lt => CompOp::Gt,
            &CompOp::Le => CompOp::Ge,
            &CompOp::Ge => CompOp::Le,
            &CompOp::Gt => CompOp::Lt,
            &CompOp::StartsWith => CompOp::NotStartsWith,
            &CompOp::NotStartsWith => CompOp::StartsWith,
            &CompOp::Compatible => CompOp::Incompatible,
            &CompOp::Incompatible => CompOp::Compatible,
        }
    }

    /// Convert to the flipped comparison operator.
    ///
    /// This uses the following bidirectional rules:
    ///
    /// * `Lt` <-> `Gt`
    /// * `Le` <-> `Ge`
    /// * Other operators are returned as is.
    ///
    /// # Examples
    ///
    /// ```
    /// use ronda::CompOp;
    ///
    /// assert_eq!(CompOp::Eq.as_flipped(), CompOp::Eq);
    /// assert_eq!(CompOp::Lt.as_flipped(), CompOp::Gt);
    /// assert_eq!(CompOp::Ge.as_flipped(), CompOp::Le);
    /// ```
    pub fn as_flipped(self) -> Self {
        self.flip()
    }

    /// Get the flipped comparison operator.
    ///
    /// This uses the following bidirectional rules:
    ///
    /// * `Lt` <-> `Gt`
    /// * `Le` <-> `Ge`
    /// * Other operators are returned as is.
    ///
    /// # Examples
    ///
    /// ```
    /// use ronda::CompOp;
    ///
    /// assert_eq!(CompOp::Eq.flip(), CompOp::Eq);
    /// assert_eq!(CompOp::Lt.flip(), CompOp::Gt);
    /// assert_eq!(CompOp::Ge.flip(), CompOp::Le);
    /// ```
    pub fn flip(&self) -> Self {
        match self {
            &CompOp::Lt => CompOp::Gt,
            &CompOp::Le => CompOp::Ge,
            &CompOp::Ge => CompOp::Le,
            &CompOp::Gt => CompOp::Lt,
            _ => self.clone(),
        }
    }

    /// Get the sign for this comparison operator.
    ///
    /// The following signs are returned:
    ///
    /// * `Eq` -> `==`
    /// * `Ne` -> `!=`
    /// * `Lt` -> `< `
    /// * `Le` -> `<=`
    /// * `Ge` -> `>=`
    /// * `Gt` -> `> `
    /// * `StartsWith` -> `=`,
    /// * `NotStartsWith` -> `!=startswith`,
    /// * `Compatible` -> `~=`,
    /// * `Incompatible` -> `!~=`,
    ///
    /// Note: Some comparison operators also support other signs,
    /// such as `=` for `Eq` and `!` for `Ne`,
    /// these are never returned by this method however as the table above is used.
    ///
    /// # Examples
    ///
    /// ```
    /// use ronda::CompOp;
    ///
    /// assert_eq!(CompOp::Eq.sign(), "==");
    /// assert_eq!(CompOp::Lt.sign(), "<");
    /// assert_eq!(CompOp::Ge.flip().sign(), "<=");
    /// ```
    pub fn sign(&self) -> &'static str {
        match self {
            &CompOp::Eq => "==",
            &CompOp::Ne => "!=",
            &CompOp::Lt => "<",
            &CompOp::Le => "<=",
            &CompOp::Ge => ">=",
            &CompOp::Gt => ">",
            &CompOp::StartsWith => "=",
            &CompOp::NotStartsWith => "!=startswith",
            &CompOp::Compatible => "~=",
            &CompOp::Incompatible => "!~=",
        }
    }

    /// Get a factor (number) for this comparison operator.
    /// These factors can be useful for quick calculations.
    ///
    /// The following factor numbers are returned:
    ///
    /// * `Eq` or `Ne` -> ` 0 `
    /// * `Lt` or `Le` -> `-1`
    /// * `Gt` or `Ge` -> ` 1`
    ///
    /// # Examples
    ///
    /// ```
    /// use ronda::Version;
    ///
    /// let ver_a: Version = "1.2.3".into();
    /// let ver_b: Version = "1.3".into();
    ///
    /// assert_eq!(ver_a.compare_version(&ver_b).factor(), -1);
    /// assert_eq!(10 * ver_b.compare_version(&ver_a).factor(), 10);
    /// ```
    pub fn factor(&self) -> i8 {
        match self {
            &CompOp::Eq | &CompOp::Ne => 0,
            &CompOp::Lt | &CompOp::Le => -1,
            &CompOp::Gt | &CompOp::Ge => 1,
            _ => 0,
        }
    }

    /// Get Rust's ordering for this comparison operator.
    ///
    /// The following comparison operators are supported:
    ///
    /// * `Eq` -> `Ordering::Equal`
    /// * `Lt` -> `Ordering::Less`
    /// * `Gt` -> `Ordering::Greater`
    ///
    /// For other comparison operators `None` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use ronda::Version;
    ///
    /// let ver_a: Version = "1.2.3".into();
    /// let ver_b: Version = "1.3".into();
    ///
    /// assert_eq!(ver_a.compare_version(&ver_b).ord().unwrap(), Ordering::Less);
    /// ```
    pub fn ord(&self) -> Option<Ordering> {
        match self {
            &CompOp::Eq => Some(Ordering::Equal),
            &CompOp::Lt => Some(Ordering::Less),
            &CompOp::Gt => Some(Ordering::Greater),
            _ => None,
        }
    }
}

#[cfg_attr(tarpaulin, skip)]
#[cfg(test)]
mod tests {
    use std::cmp::Ordering;
    use super::CompOp;

    #[test]
    fn from_sign() {
        // Normal signs
        assert_eq!(CompOp::from_sign("==").unwrap(), CompOp::Eq);
        assert_eq!(CompOp::from_sign("!=").unwrap(), CompOp::Ne);
        assert_eq!(CompOp::from_sign("<").unwrap(), CompOp::Lt);
        assert_eq!(CompOp::from_sign("<=").unwrap(), CompOp::Le);
        assert_eq!(CompOp::from_sign(">=").unwrap(), CompOp::Ge);
        assert_eq!(CompOp::from_sign(">").unwrap(), CompOp::Gt);
        assert_eq!(CompOp::from_sign("=").unwrap(), CompOp::StartsWith);
        assert_eq!(CompOp::from_sign("!=startswith").unwrap(), CompOp::NotStartsWith);
        assert_eq!(CompOp::from_sign("~=").unwrap(), CompOp::Compatible);
        assert_eq!(CompOp::from_sign("!~=").unwrap(), CompOp::Incompatible);

        // Exceptional cases
        assert_eq!(CompOp::from_sign("  <=  ").unwrap(), CompOp::Le);
        assert!(CompOp::from_sign("*").is_err());
    }

    #[test]
    fn from_name() {
        // Normal names
        assert_eq!(CompOp::from_name("eq").unwrap(), CompOp::Eq);
        assert_eq!(CompOp::from_name("ne").unwrap(), CompOp::Ne);
        assert_eq!(CompOp::from_name("lt").unwrap(), CompOp::Lt);
        assert_eq!(CompOp::from_name("le").unwrap(), CompOp::Le);
        assert_eq!(CompOp::from_name("ge").unwrap(), CompOp::Ge);
        assert_eq!(CompOp::from_name("gt").unwrap(), CompOp::Gt);
        assert_eq!(CompOp::from_name("startswith").unwrap(), CompOp::StartsWith);
        assert_eq!(CompOp::from_name("notstartswith").unwrap(), CompOp::NotStartsWith);
        assert_eq!(CompOp::from_name("compatible").unwrap(), CompOp::Compatible);
        assert_eq!(CompOp::from_name("incompatible").unwrap(), CompOp::Incompatible);

        // Exceptional cases
        assert_eq!(CompOp::from_name("  Le  ").unwrap(), CompOp::Le);
        assert!(CompOp::from_name("abc").is_err());
    }

    #[test]
    fn from_ord() {
        assert_eq!(CompOp::from_ord(Ordering::Less), CompOp::Lt);
        assert_eq!(CompOp::from_ord(Ordering::Equal), CompOp::Eq);
        assert_eq!(CompOp::from_ord(Ordering::Greater), CompOp::Gt);
    }

    #[test]
    fn name() {
        assert_eq!(CompOp::Eq.name(), "eq");
        assert_eq!(CompOp::Ne.name(), "ne");
        assert_eq!(CompOp::Lt.name(), "lt");
        assert_eq!(CompOp::Le.name(), "le");
        assert_eq!(CompOp::Ge.name(), "ge");
        assert_eq!(CompOp::Gt.name(), "gt");
        assert_eq!(CompOp::StartsWith.name(), "startswith");
        assert_eq!(CompOp::NotStartsWith.name(), "notstartswith");
        assert_eq!(CompOp::Compatible.name(), "compatible");
        assert_eq!(CompOp::Incompatible.name(), "incompatible");
    }

    #[test]
    fn as_inverted() {
        assert_eq!(CompOp::Ne.as_inverted(), CompOp::Eq);
        assert_eq!(CompOp::Eq.as_inverted(), CompOp::Ne);
        assert_eq!(CompOp::Ge.as_inverted(), CompOp::Lt);
        assert_eq!(CompOp::Gt.as_inverted(), CompOp::Le);
        assert_eq!(CompOp::Lt.as_inverted(), CompOp::Ge);
        assert_eq!(CompOp::Le.as_inverted(), CompOp::Gt);
        assert_eq!(CompOp::StartsWith.as_inverted(), CompOp::NotStartsWith);
        assert_eq!(CompOp::NotStartsWith.as_inverted(), CompOp::StartsWith);
        assert_eq!(CompOp::Compatible.as_inverted(), CompOp::Incompatible);
        assert_eq!(CompOp::Incompatible.as_inverted(), CompOp::Compatible);
    }

    #[test]
    fn invert() {
        assert_eq!(CompOp::Ne.invert(), CompOp::Eq);
        assert_eq!(CompOp::Eq.invert(), CompOp::Ne);
        assert_eq!(CompOp::Ge.invert(), CompOp::Lt);
        assert_eq!(CompOp::Gt.invert(), CompOp::Le);
        assert_eq!(CompOp::Lt.invert(), CompOp::Ge);
        assert_eq!(CompOp::Le.invert(), CompOp::Gt);
        assert_eq!(CompOp::StartsWith.invert(), CompOp::NotStartsWith);
        assert_eq!(CompOp::NotStartsWith.invert(), CompOp::StartsWith);
        assert_eq!(CompOp::Compatible.invert(), CompOp::Incompatible);
        assert_eq!(CompOp::Incompatible.invert(), CompOp::Compatible);
    }

    #[test]
    fn as_opposite() {
        assert_eq!(CompOp::Ne.as_opposite(), CompOp::Eq);
        assert_eq!(CompOp::Eq.as_opposite(), CompOp::Ne);
        assert_eq!(CompOp::Gt.as_opposite(), CompOp::Lt);
        assert_eq!(CompOp::Ge.as_opposite(), CompOp::Le);
        assert_eq!(CompOp::Le.as_opposite(), CompOp::Ge);
        assert_eq!(CompOp::Lt.as_opposite(), CompOp::Gt);
        assert_eq!(CompOp::StartsWith.as_opposite(), CompOp::NotStartsWith);
        assert_eq!(CompOp::NotStartsWith.as_opposite(), CompOp::StartsWith);
        assert_eq!(CompOp::Compatible.as_opposite(), CompOp::Incompatible);
        assert_eq!(CompOp::Incompatible.as_opposite(), CompOp::Compatible);
    }

    #[test]
    fn opposite() {
        assert_eq!(CompOp::Eq.opposite(), CompOp::Ne);
        assert_eq!(CompOp::Ne.opposite(), CompOp::Eq);
        assert_eq!(CompOp::Lt.opposite(), CompOp::Gt);
        assert_eq!(CompOp::Le.opposite(), CompOp::Ge);
        assert_eq!(CompOp::Ge.opposite(), CompOp::Le);
        assert_eq!(CompOp::Gt.opposite(), CompOp::Lt);
        assert_eq!(CompOp::StartsWith.opposite(), CompOp::NotStartsWith);
        assert_eq!(CompOp::NotStartsWith.opposite(), CompOp::StartsWith);
        assert_eq!(CompOp::Compatible.opposite(), CompOp::Incompatible);
        assert_eq!(CompOp::Incompatible.opposite(), CompOp::Compatible);
    }

    #[test]
    fn as_flipped() {
        assert_eq!(CompOp::Lt.as_flipped(), CompOp::Gt);
        assert_eq!(CompOp::Le.as_flipped(), CompOp::Ge);
        assert_eq!(CompOp::Ge.as_flipped(), CompOp::Le);
        assert_eq!(CompOp::Gt.as_flipped(), CompOp::Lt);
        // Not touched
        assert_eq!(CompOp::Eq.as_flipped(), CompOp::Eq);
        assert_eq!(CompOp::Ne.as_flipped(), CompOp::Ne);
        assert_eq!(CompOp::StartsWith.as_flipped(), CompOp::StartsWith);
        assert_eq!(CompOp::StartsWith.as_flipped(), CompOp::StartsWith);
        assert_eq!(CompOp::Compatible.as_flipped(), CompOp::Compatible);
        assert_eq!(CompOp::Incompatible.as_flipped(), CompOp::Incompatible);
    }

    #[test]
    fn flip() {
        assert_eq!(CompOp::Lt.flip(), CompOp::Gt);
        assert_eq!(CompOp::Le.flip(), CompOp::Ge);
        assert_eq!(CompOp::Ge.flip(), CompOp::Le);
        assert_eq!(CompOp::Gt.flip(), CompOp::Lt);
        // Not touched
        assert_eq!(CompOp::Eq.flip(), CompOp::Eq);
        assert_eq!(CompOp::Ne.flip(), CompOp::Ne);
        assert_eq!(CompOp::StartsWith.flip(), CompOp::StartsWith);
        assert_eq!(CompOp::NotStartsWith.flip(), CompOp::NotStartsWith);
        assert_eq!(CompOp::Compatible.flip(), CompOp::Compatible);
        assert_eq!(CompOp::Incompatible.flip(), CompOp::Incompatible);
    }

    #[test]
    fn sign() {
        assert_eq!(CompOp::Eq.sign(), "==");
        assert_eq!(CompOp::Ne.sign(), "!=");
        assert_eq!(CompOp::Lt.sign(), "<");
        assert_eq!(CompOp::Le.sign(), "<=");
        assert_eq!(CompOp::Ge.sign(), ">=");
        assert_eq!(CompOp::Gt.sign(), ">");
        assert_eq!(CompOp::StartsWith.sign(), "=");
        assert_eq!(CompOp::NotStartsWith.sign(), "!=startswith");
        assert_eq!(CompOp::Compatible.sign(), "~=");
        assert_eq!(CompOp::Incompatible.sign(), "!~=");
    }

    #[test]
    fn factor() {
        assert_eq!(CompOp::Eq.factor(), 0);
        assert_eq!(CompOp::Ne.factor(), 0);
        assert_eq!(CompOp::Lt.factor(), -1);
        assert_eq!(CompOp::Le.factor(), -1);
        assert_eq!(CompOp::Ge.factor(), 1);
        assert_eq!(CompOp::Gt.factor(), 1);
    }

    #[test]
    fn ord() {
        assert_eq!(CompOp::Eq.ord(), Some(Ordering::Equal));
        assert_eq!(CompOp::Ne.ord(), None);
        assert_eq!(CompOp::Lt.ord(), Some(Ordering::Less));
        assert_eq!(CompOp::Le.ord(), None);
        assert_eq!(CompOp::Ge.ord(), None);
        assert_eq!(CompOp::Gt.ord(), Some(Ordering::Greater));
        assert_eq!(CompOp::Compatible.ord(), None);
        assert_eq!(CompOp::Incompatible.ord(), None);
        assert_eq!(CompOp::StartsWith.ord(), None);
        assert_eq!(CompOp::NotStartsWith.ord(), None);
    }
}
