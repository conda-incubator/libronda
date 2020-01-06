use super::spec_trees::*;
use regex::Regex;
use std::collections::HashSet;
use std::convert::TryFrom;
use crate::{Version, CompOp};

pub trait Spec {
    // properties in Python
    fn raw_value(&self) -> &str { self.get_spec() }
    fn exact_value(&self) -> Option<&str> {
        if self.is_exact() { Some(self.get_spec()) } else { None } }

    // To be implemented by other things
    fn merge(&self, other: &Self) -> Self;

    // properties in Python (to be implemented by other things)
    fn get_spec(&self) -> &str;
    fn get_matcher(&self) -> &MatchEnum;
    fn is_exact(&self) -> bool;
    fn test_match(&self, other: &str) -> bool { self.get_matcher().test(other) }
}

#[derive(Clone)]
struct VersionSpec {
    spec_str: String,
    tree: Option<ConstraintTree>,
    matcher: MatchEnum,
    _is_exact: bool
}

impl Spec for VersionSpec {
    fn merge(&self, other: &Self) -> Self { panic!("Not implemented") }
    fn get_spec(&self) -> &str { &self.spec_str }
    fn get_matcher(&self) -> &MatchEnum { &self.matcher }
    fn is_exact(&self) -> bool { self._is_exact }
}

impl TryFrom<&str> for VersionSpec {
    type Error = String;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        lazy_static! { static ref REGEX_SPLIT_RE: Regex = Regex::new( r#".*[()|,^$]"# ).unwrap(); }
        lazy_static! { static ref OPERATOR_START: HashSet<&'static str> = ["=", "<", ">", "!", "~"].iter().cloned().collect(); }
        let _is_exact = false;
        let split_input: Vec<&str> = REGEX_SPLIT_RE.split(input).collect();
        if split_input.len() > 0 {
            let tree = treeify(input)?;
            return Ok(tree.into());
        }
        let mut matcher: MatchEnum = Default::default();
        let mut _is_exact = false;
        if input.starts_with("^") || input.ends_with("$") {
            if ! input.starts_with("^") || ! input.ends_with("$") {
                return Err(format!("regex specs must start with '^' and end with '$' - spec '{}' is incorrect", input))
            }
            matcher = MatchRegex { expression: Regex::new(input).unwrap() }.into();
            _is_exact = false;
        } else if OPERATOR_START.contains(&input[..1]) {
            let (_m, _e) = create_match_enum_from_operator_str(input)?;
            matcher = _m;
            _is_exact = _e;
        } else if input == "*" {
            matcher = MatchAlways {}.into();
            _is_exact = false;
        } else if input.trim_end_matches("*").contains("*") {
            let rx = input.replace(".", r"\.").replace("+", r"\+").replace("*", r".*");
            let rx: Regex = Regex::new(&format!(r"^(?:{})$", rx)).unwrap();
            matcher = MatchRegex { expression: rx }.into();
            _is_exact = false;
        } else if input.ends_with("*") {
            matcher = MatchOperator {
                operator: CompOp::StartsWith,
                version: input.trim_end_matches(|c| c=='*' || c=='.').into() }.into();
            _is_exact = false;
        } else if ! input.contains("@") {
            matcher = MatchOperator {operator: CompOp::Eq, version: input.into()}.into();
            _is_exact = true;
        } else {
            matcher = MatchExact { spec: input.to_string() }.into();
            _is_exact = true;
        }
        Ok(VersionSpec { spec_str: input.to_string(), tree: None, matcher, _is_exact })
    }
}

impl From<ConstraintTree> for VersionSpec {
    fn from(tree: ConstraintTree) -> VersionSpec {
        let matcher = match tree.combinator {
            Combinator::Or => MatchAny { tree: tree.clone() }.into(),
            _ => MatchAll { tree: tree.clone() }.into()
        };
        let spec_str = untreeify(&tree).unwrap();
        // ConstraintTree matches are never exact
        VersionSpec { spec_str, tree: Some(tree), matcher, _is_exact: false }
    }
}

fn create_match_enum_from_operator_str(input: &str) -> Result<(MatchEnum, bool), String> {
    lazy_static! { static ref VERSION_RELATION_RE: Regex = Regex::new( r#"^(=|==|!=|<=|>=|<|>|~=)(?![=<>!~])(\S+)$"# ).unwrap(); }

    let (mut operator_str, mut v_str) = match VERSION_RELATION_RE.captures(input) {
        None => return Err(format!("invalid operator in string {}", input)),
        Some(caps) => (caps.get(1).map_or("", |m| m.as_str()), caps.get(2).map_or("", |m| m.as_str()))
    };

    if v_str.ends_with(".*") {
        if operator_str == "!=" {
            operator_str = "!=startswith";
        } else if operator_str == "~=" {
            return Err(format!("invalid operator (~=) with '.*' in spec string: {}", input));
        }
        v_str = &v_str[..v_str.len()-2];
    }
    let matcher = MatchOperator { operator: CompOp::from_sign(operator_str).unwrap(), version: v_str.into() };
    let _is_exact = operator_str == "==";
    Ok((matcher.into(), _is_exact))
}

#[enum_dispatch]
#[derive(Clone)]
enum MatchEnum {
    MatchAny,
    MatchAll,
    MatchRegex,
    MatchOperator,
    MatchAlways,
    MatchExact,
    MatchNever,
}

impl Default for MatchEnum {
    fn default() -> Self { MatchNever{}.into() }
}

#[enum_dispatch(MatchEnum)]
trait MatchFn {
    fn test(&self, other: &str) -> bool;
}

#[derive(Clone)]
struct MatchAny {
    tree: ConstraintTree,
}
impl MatchFn for MatchAny {
    fn test(&self, other: &str) -> bool {
        // We probably need to convert each individual string of a ConstraintTree into a
        // MatchOperator, and then have the "other" match with each of those individually.
        panic!("Not implemented.  Not sure how tuple of VersionSpec matches with ConstraintTree")
        // self.tree.parts.iter().any(|x| x == other)
    }
}

#[derive(Clone)]
struct MatchAll {
    tree: ConstraintTree,
}
impl MatchFn for MatchAll {
    fn test(&self, other: &str) -> bool {
        // We probably need to convert each individual string of a ConstraintTree into a
        // MatchOperator, and then have the "other" match with each of those individually.
        panic!("Not implemented.  Not sure how tuple of VersionSpec matches with ConstraintTree")
        // self.tree.parts.iter().all(|x| x == other)
    }
}

#[derive(Clone)]
struct MatchRegex {
    expression: Regex
}
impl MatchFn for MatchRegex {
    fn test(&self, other: & str) -> bool {
        panic!("Not implemented")
    }
}

#[derive(Clone)]
struct MatchOperator {
    operator: CompOp,
    version: Version,
}
impl MatchFn for MatchOperator {
    fn test(&self, other: & str) -> bool {
        self.version.compare_to_str(other, &self.operator)
    }
}

#[derive(Clone)]
struct MatchAlways {}
impl MatchFn for MatchAlways {
    fn test(&self, _other: & str) -> bool {
        true
    }
}

#[derive(Clone)]
struct MatchNever {}
impl MatchFn for MatchNever {
    fn test(&self, _other: & str) -> bool {
        false
    }
}

#[derive(Clone)]
struct MatchExact {
    spec: String
}
impl MatchFn for MatchExact {
    fn test(&self, other: & str) -> bool {
        other == self.spec
    }
}
