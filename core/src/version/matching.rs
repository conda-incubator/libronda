use super::spec_trees::*;
use regex::Regex;
use std::collections::HashSet;

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
    fn merge(&self, other: &Self) -> Self { panic!() }
    fn is_exact(&self) -> bool { self._is_exact }
}

impl VersionSpec {}

impl From<&str> for VersionSpec {
    fn from(input: &str) -> VersionSpec {
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

fn create_version_spec_from_operator_str(input: &str) -> VersionSpec {
    m = version_relation_re.match(vspec_str)
    if m is None:
        raise InvalidVersionSpec(vspec_str, "invalid operator")
    operator_str, vo_str = m.groups()
    if vo_str[-2:] == '.*':
    if operator_str in ("=", ">="):
        vo_str = vo_str[:-2]
    elif operator_str == "!=":
        vo_str = vo_str[:-2]
    operator_str = "!=startswith"
    elif operator_str == "~=":
        raise InvalidVersionSpec(vspec_str, "invalid operator with '.*'")
    else:
    log.warning("Using .* with relational operator is superfluous and deprecated "
    "and will be removed in a future version of conda. Your spec was "
    "{}, but conda is ignoring the .* and treating it as {}"
        .format(vo_str, vo_str[:-2]))
    vo_str = vo_str[:-2]
    try:
        self.operator_func = OPERATOR_MAP[operator_str]
    except KeyError:
        raise InvalidVersionSpec(vspec_str, "invalid operator: %s" % operator_str)
    self.matcher_vo = VersionOrder(vo_str)
    matcher = self.operator_match
    is_exact = operator_str == "=="
    VersionSpec {}
}

fn match_any<T: Spec>(spec: &T, spec_str: &str) -> bool {
    spec.iter().any(|x| x == spec_str)
}

fn match_all<T: Spec>(spec: &T, spec_str: &str) -> bool {
    spec.iter().all(|x| x == spec_str)
}

fn regex_match<T: Spec>(spec: &T, spec_str: &str, pattern: Regex) -> bool {
    panic!()
}

fn operator_match<T: Spec>(spec: &T, spec_str: &str) -> bool {
    panic!()
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