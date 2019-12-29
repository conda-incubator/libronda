use super::spec_trees::*;
use regex::Regex;
use std::collections::HashSet;
use std::convert::{TryFrom, TryInto};
use crate::version::matching::Operator::Ne;
use core::panicking::panic_fmt;
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
    matcher: Box<Fn(&Self, &Self) -> bool>,
    _is_exact: bool
}

impl Spec for VersionSpec {
    fn spec(&self) -> &str { &self.spec_str }
    fn merge(&self, other: &Self) -> Self { panic!("Not implemented") }
    fn is_exact(&self) -> bool { self._is_exact }
}

impl VersionSpec {}

impl TryFrom<&str> for VersionSpec {
    type Error = &'static str;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        lazy_static! { static ref REGEX_SPLIT_RE: Regex = Regex::new( r#".*[()|,^$]"# ).unwrap(); }
        lazy_static! { static ref OPERATOR_START: HashSet<&'static str> = ["=", "<", ">", "!", "~"].iter().cloned().collect(); }
        if REGEX_SPLIT_RE.split(input).collect().len() > 0 {
            treeify(input).into()
        } else if OPERATOR_START.contains(input[0]) {
            create_version_spec_from_operator_str(input)
        } else if input == "*" {
            VersionSpec { spec_str: input.to_string(), tree: None,
                matcher: Box::new(always_match), _is_exact: false }
        } else if input.trim_end_matches("*").contains("*") {
            let rx = input.replace(".", r"\.").replace("+", r"\+").replace("*", r".*")
            let Regex: rx = Regex::new(&format!(r"^(?:{})$", rx));
            VersionSpec { spec_str: input.to_string(), tree: None,
                matcher: Box::new(|spec: &VersionSpec, spec_str: &str| regex_match(spec, spec_str, re)),
                _is_exact: false }
        } else if input.ends_with("*") {
            if ! input.ends_with(".*") {
                let input = input[..-1] + ".*"
            }

            vo_str = vspec_str.rstrip('*').rstrip('.')
            self.operator_func = VersionOrder.startswith
            self.matcher_vo = VersionOrder(vo_str)
            matcher = self.operator_match
            is_exact = False
        } else if ! input.contains("@") {
            self.operator_func = OPERATOR_MAP["=="]
            self.matcher_vo = VersionOrder(vspec_str)
            matcher = self.operator_match
            is_exact = True
        } else {
            VersionSpec { spec_str: input.to_string(), tree: None, matcher: Box::new(exact_match), _is_exact: true }
        }
    }
}

impl From<ConstraintTree> for VersionSpec {
    fn from(tree: ConstraintTree) -> VersionSpec {
        let matcher = match vspec.combinator {
            Combinator::Or => Box::new(any_match),
            _ => Box::new(all_match)
        };
        let spec_str = untreeify(vspec);
        // ConstraintTree matches are never exact
        let _is_exact = false;
        VersionSpec { spec_str, tree: Some(tree), matcher, _is_exact }
    }
}

fn create_version_spec_from_operator_str(input: &str) -> Result<VersionSpec, &'static str> {
    lazy_static! { static ref VERSION_RELATION_RE: Regex = Regex::new( r#"^(=|==|!=|<=|>=|<|>|~=)(?![=<>!~])(\S+)$"# ).unwrap(); }

    let (operator_str: &str, v_str: &str) = match VERSION_RELATION_RE.captures(input) {
        None => return Err(&format!("invalid operator in string {}", input)),
        Some(caps) => (caps.get(1).as_str(), caps.get(2).as_str())
    };

    if v_str.endswith(".*") {
        if operator_str == "!=" {
            let operator_str = "!=startswith";
        } else if operator_str == "~=" {
            return Err(&format!("invalid operator (~=) with '.*' in spec string: {}", input));
        }
        let v_str = v_str[..-2];
    }
    let matcher = Box::New(|x: &str| operator_match(spec: x, operator: operator_str.into(), version: v_str.into()));
    Ok(VersionSpec {spec_str: input.to_string(), tree: None, matcher, _is_exact: operator_str == "=="})
}

fn match_any<T: Spec>(spec: &T, spec_str: &str) -> bool {
    spec.iter().any(|x| x == spec_str)
}

fn match_all<T: Spec>(spec: &T, spec_str: &str) -> bool {
    spec.iter().all(|x| x == spec_str)
}

fn regex_match<T: Spec>(spec: &T, spec_str: &str, pattern: Regex) -> bool {
    panic!("Not implemented")
}

impl From<&str> for Operator {
    fn from(s: &str) -> Operator {
        match s {
            "==" => CompOp::Eq,
            "!=" => CompOp::Ne,
            "<=" => CompOp::Le,
            ">=" => CompOp::Ge,
            "<" => CompOp:Lt,
            ">" => CompOp:Gt,
            "=" => CompOp:Startswith,
            "!=startswith" => CompOp::NotStartswith,
            "~=" => CompOp::Compatible,
            _ => panic_fmt!("unrecognized operator string: {}", s)
        }
    }
}

// the operator and version are what is stored relative to the Spec, so the Spec doesn't need to be an extra arg here.
fn operator_match(spec_str: &str, operator: CompOp, version: &Version) -> bool {
    panic!("Not implemented")
}

fn always_true_match<T: Spec>(_spec: &T, _spec_str: &str) -> bool {
    true
}

fn exact_match<T: Spec>(spec: &T, spec_str: &str) -> bool {
    spec_str == spec.spec()
}

enum MatchFn {
    Any(|spec: &VersionSpec, spec_str: &str| match_any(spec, spec_str)),
}