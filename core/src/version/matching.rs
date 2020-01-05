use super::spec_trees::*;
use regex::Regex;
use std::collections::HashSet;
use std::convert::TryFrom;
use crate::version::matching::Operator::Ne;
use crate::{Version, CompOp};

pub trait Spec {
    // properties in Python
    fn raw_value(&self) -> &str { self.spec() }
    fn exact_value(&self) -> Option<&str> {
        if self.is_exact() { Some(self.spec()) } else { None } }

    // To be implemented by other things
    fn merge(&self, other: &Self) -> Self;

    // properties in Python (to be implemented by other things)
    fn spec(&self) -> &str;
    fn is_exact(&self) -> bool;
}

#[derive(Clone)]
struct VersionSpec {
    spec_str: String,
    tree: Some(ConstraintTree),
    matcher: MatchEnum,
    _is_exact: bool
}

impl Spec for VersionSpec {
    fn spec(&self) -> &str { &self.spec_str }
    fn merge(&self, other: &Self) -> Self { panic!("Not implemented") }
    fn is_exact(&self) -> bool { self._is_exact }
}

impl TryFrom<&str> for VersionSpec {
    type Error = &'static str;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        lazy_static! { static ref REGEX_SPLIT_RE: Regex = Regex::new( r#".*[()|,^$]"# ).unwrap(); }
        lazy_static! { static ref OPERATOR_START: HashSet<&'static str> = ["=", "<", ">", "!", "~"].iter().cloned().collect(); }
        if REGEX_SPLIT_RE.split(input).collect().len() > 0 {
            treeify(input).into();
            panic!(" this is not completely implemented")
        } else if OPERATOR_START.contains(input[0]) {
            create_version_spec_from_operator_str(input)
        } else if input == "*" {
            let matcher = MatchAlways {};
            let _is_exact = false;
        } else if input.trim_end_matches("*").contains("*") {
            let rx = input.replace(".", r"\.").replace("+", r"\+").replace("*", r".*");
            let rx: Regex = Regex::new(&format!(r"^(?:{})$", rx))?;
            let matcher = MatchRegex { expression: rx };
            let _is_exact = false;
        } else if input.ends_with("*") {
            let matcher = MatchOperator {
                operator: CompOp::StartsWith,
                version: input.trim_end_matches(&['*', '.']).into() };
            let _is_exact = false;
        } else if ! input.contains("@") {
            let matcher = MatchOperator {operator: CompOp::Eq, version: input.into()};
            let _is_exact = true;
        } else {
            let matcher = MatchExact { spec: input.to_string() };
            let _is_exact = true;
        }
        Ok(VersionSpec { spec_str: input.to_string(), tree: None, matcher: matcher.into(), _is_exact })
    }
}

impl From<ConstraintTree> for VersionSpec {
    fn from(tree: ConstraintTree) -> VersionSpec {
        let matcher = match vspec.combinator {
            Combinator::Or => MatchAny { tree: tree.clone() }.into(),
            _ => MatchAll { tree: tree.clone() }.into()
        };
        let spec_str = untreeify(vspec).unwrap();
        // ConstraintTree matches are never exact
        VersionSpec { spec_str, tree: Some(tree), matcher, _is_exact: false }
    }
}

fn create_version_spec_from_operator_str(input: &str) -> Result<VersionSpec, &'static str> {
    lazy_static! { static ref VERSION_RELATION_RE: Regex = Regex::new( r#"^(=|==|!=|<=|>=|<|>|~=)(?![=<>!~])(\S+)$"# ).unwrap(); }

    let (operator_str, v_str) = match VERSION_RELATION_RE.captures(input) {
        None => return Err(&format!("invalid operator in string {}", input)),
        Some(caps) => (caps.get(1).map_or("", |m| m.as_str()), caps.get(2).map_or("", |m| m.as_str()))
    };

    if v_str.ends_with(".*") {
        if operator_str == "!=" {
            let operator_str = "!=startswith";
        } else if operator_str == "~=" {
            return Err(&format!("invalid operator (~=) with '.*' in spec string: {}", input));
        }
        let v_str = &v_str[..-2];
    }
    let matcher = MatchOperator { operator: CompOp::from_sign(operator_str)?, version: v_str.into() };
    Ok(VersionSpec {spec_str: input.to_string(), tree: None, matcher: matcher.into(), _is_exact: operator_str == "=="})
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
        self.tree.parts.iter().any(|x| x == spec_str)
    }
}

#[derive(Clone)]
struct MatchAll {
    tree: ConstraintTree,
}
impl MatchFn for MatchAll {
    fn test(&self, other: &str) -> bool {
        self.tree.parts.iter().all(|x| x == spec_str)
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
        self.version.compare_to_str(spec_str, &self.operator)
    }
}

#[derive(Clone)]
struct MatchAlways {}
impl MatchFn for MatchAlways {
    fn test(&self, other: & str) -> bool {
        true
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
