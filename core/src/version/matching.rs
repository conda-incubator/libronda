use super::spec_trees::*;
use regex::Regex;
use crate::{Version, CompOp};
use crate::version::errors::VersionParsingError;
use std::collections::HashSet;


pub(crate) fn create_match_enum_from_operator_str(input: &str) -> Result<(MatchEnum, bool), String> {
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
pub enum MatchEnum {
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

pub fn get_matcher(input: &str) -> Result<(MatchEnum, bool), VersionParsingError> {
    lazy_static! { static ref REGEX_SPLIT_RE: Regex = Regex::new( r#".*[()|,^$]"# ).unwrap(); }
    lazy_static! { static ref OPERATOR_START: HashSet<&'static str> = ["=", "<", ">", "!", "~"].iter().cloned().collect(); }
    let _is_exact = false;
    let matcher: MatchEnum;
    let mut _is_exact = false;
    if input.starts_with("^") || input.ends_with("$") {
        if ! input.starts_with("^") || ! input.ends_with("$") {
            return Err(VersionParsingError::Message(format!("regex specs must start with '^' and end with '$' - spec '{}' is incorrect", input)))
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
    return Ok((matcher, _is_exact))
}

#[enum_dispatch(MatchEnum)]
trait MatchFn {
    fn test(&self, other: &Version) -> bool;
}

#[derive(Clone)]
pub struct MatchAny {
    pub tree: ConstraintTree,
}
impl MatchFn for MatchAny {
    fn test(&self, other: &Version) -> bool {
        return self.tree.parts.iter().any(|x| x.test_match(other))
    }
}

#[derive(Clone)]
pub struct MatchAll {
    pub tree: ConstraintTree,
}
impl MatchFn for MatchAll {
    fn test(&self, other: &Version) -> bool {
        return self.tree.parts.iter().all(|x| x.test_match(other))
    }
}

#[derive(Clone)]
pub struct MatchRegex {
    pub expression: Regex
}
impl MatchFn for MatchRegex {
    fn test(&self, _other: &Version) -> bool {
        panic!("Not implemented")
    }
}

#[derive(Clone)]
pub struct MatchOperator {
    pub operator: CompOp,
    pub version: Version,
}
impl MatchFn for MatchOperator {
    fn test(&self, other: &Version) -> bool {
        self.version.compare_to_version(other, &self.operator)
    }
}

#[derive(Clone)]
pub struct MatchAlways {}
impl MatchFn for MatchAlways {
    fn test(&self, _other: &Version) -> bool {
        true
    }
}

#[derive(Clone)]
pub struct MatchNever {}
impl MatchFn for MatchNever {
    fn test(&self, _other: &Version) -> bool {
        false
    }
}

#[derive(Clone)]
pub struct MatchExact {
    pub spec: String
}
impl MatchFn for MatchExact {
    fn test(&self, other: &Version) -> bool {
        other.version == self.spec
    }
}


#[cfg_attr(tarpaulin, skip)]
#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn test_ver_eval() {
        assert_eq!(VersionSpec::try_from("==1.7").unwrap().test_match("1.7.0"), true);
        assert_eq!(VersionSpec::try_from("<=1.7").unwrap().test_match("1.7.0"), true);
        assert_eq!(VersionSpec::try_from("<1.7").unwrap().test_match("1.7.0"), false);
        assert_eq!(VersionSpec::try_from(">=1.7").unwrap().test_match("1.7.0"), true);
        assert_eq!(VersionSpec::try_from(">1.7").unwrap().test_match("1.7.0"), false);
        assert_eq!(VersionSpec::try_from(">=1.7").unwrap().test_match("1.6.7"), false);
        assert_eq!(VersionSpec::try_from("2013a").unwrap().test_match(">2013b"), false);
        assert_eq!(VersionSpec::try_from("2013k").unwrap().test_match(">2013b"), true);
        assert_eq!(VersionSpec::try_from("3.0.0").unwrap().test_match(">2013b"), false);
        assert_eq!(VersionSpec::try_from("1.0.0").unwrap().test_match(">1.0.0a"), true);
        assert_eq!(VersionSpec::try_from("1.0.0").unwrap().test_match(">1.0.0*"), true);
        assert_eq!(VersionSpec::try_from("1.0").unwrap().test_match("1.0*"), true);
        assert_eq!(VersionSpec::try_from("1.0.0").unwrap().test_match("1.0*"), true);
        assert_eq!(VersionSpec::try_from("1.0").unwrap().test_match("1.0.0*"), true);
        assert_eq!(VersionSpec::try_from("1.0.1").unwrap().test_match("1.0.0*"), false);
        assert_eq!(VersionSpec::try_from("2013a").unwrap().test_match("2013a*"), true);
        assert_eq!(VersionSpec::try_from("2013a").unwrap().test_match("2013b*"), false);
        assert_eq!(VersionSpec::try_from("2013ab").unwrap().test_match("2013a*"), true);
        assert_eq!(VersionSpec::try_from("1.3.4").unwrap().test_match("1.2.4*"), false);
        assert_eq!(VersionSpec::try_from("1.2.3+4.5.6").unwrap().test_match("1.2.3*"), true);
        assert_eq!(VersionSpec::try_from("1.2.3+4.5.6").unwrap().test_match("1.2.3+4*"), true);
        assert_eq!(VersionSpec::try_from("1.2.3+4.5.6").unwrap().test_match("1.2.3+5*"), false);
        assert_eq!(VersionSpec::try_from("1.2.3+4.5.6").unwrap().test_match("1.2.4+5*"), false);
    }

    #[test]
    fn test_ver_eval_errors() {
        // each of these should raise
        VersionSpec::try_from("3.0.0").unwrap().test_match("><2.4.5");
        VersionSpec::try_from("3.0.0").unwrap().test_match("!!2.4.5");
        VersionSpec::try_from("3.0.0").unwrap().test_match("!");
    }

    #[test]
    fn test_version_spec_1() {
        let v1 = VersionSpec::try_from("1.7.1").unwrap();
        let v2 = VersionSpec::try_from("1.7.1*").unwrap();
        let v3 = VersionSpec::try_from("1.7.1").unwrap();
        assert!(v1.is_exact());
        assert_ne!(v2.is_exact(), true);
        assert!(v3.is_exact());
        // right now, VersionSpec instance are not orderable nor equal by value. Versions are, though.
        // assert_eq!(v1, v3);
        // assert_ne!(v1, v2);
        // assert_ne!(v3, v2);
        // assert_ne!(v1, 1.0);
        // pointer tests here are testing caching - are equal values created as just one object?
        // https://users.rust-lang.org/t/is-any-way-to-know-references-are-referencing-the-same-object/9716/6
        assert_eq!(&v1 as *const _, &v3 as *const _);
        assert_ne!(&v1 as *const _, &v2 as *const _);
    }

    #[test]
    fn test_version_spec_2() {
        //let v1 = VersionSpec::try_from("( (1.5|((1.6|1.7), 1.8), 1.9 |2.0))|2.1").unwrap();
        //assert_eq!(v1.spec, "1.5|1.6|1.7,1.8,1.9|2.0|2.1");
        match VersionSpec::try_from("(1.5"){
            Ok(_) => panic!(),
            _ => true
        };
        match VersionSpec::try_from("1.5)"){
            Ok(_) => panic!(),
            _ => true
        };
        match VersionSpec::try_from("1.5||1.6"){
            Ok(_) => panic!(),
            _ => true
        };
        match VersionSpec::try_from("^1.5"){
            Ok(_) => panic!(),
            _ => true
        };
    }

    #[test]
    fn test_version_spec_3(){
        let v1 = VersionSpec::try_from("1.7.1*").unwrap();
        let v2 = VersionSpec::try_from("1.7.1.*").unwrap();
        assert_eq!(v1.is_exact(), false);
        assert_eq!(v2.is_exact(), false);
        // right now, VersionSpec instance are not orderable nor equal by value. Versions are, though.
        // assert_eq!(v1, v2);
        // assert_eq!(v1 != v2, false);
        assert_eq!(&v1 as *const _, &v2 as *const _);
    }

    #[test]
    fn test_version_spec_4() {
        let v1 = VersionSpec::try_from("1.7.1*,1.8.1*").unwrap();
        let v2 = VersionSpec::try_from("1.7.1.*,1.8.1.*").unwrap();
        let v3 = VersionSpec::try_from("1.7.1*,1.8.1.*").unwrap();
        assert_eq!(v1.is_exact(), false);
        assert_eq!(v2.is_exact(), false);
        // right now, VersionSpec instance are not orderable nor equal by value. Versions are, though.
        // assert!((v1 == v2) && (v2 == v3));
        // assert_eq!(v1 != v2, false);
        assert_eq!(&v1 as *const _, &v2 as *const _);
        assert_eq!(&v1 as *const _, &v3 as *const _);

    }

    // case("1.8/*|1.9.*', false),  what was this supposed to be?
    // case("^1\.8.*$", false),     invalid escape in rust

    #[rstest(vspec, res,
        case("1.7.*", true),
        case("1.7.1", true),
        case("1.7.0", false),
        case("1.7", false),
        case("1.5.*", false),
        case(">=1.5", true),
        case("!=1.5", true),
        case("!=1.7.1", false),
        case("==1.7.1", true),
        case("==1.7", false),
        case("==1.7.2", false),
        case("==1.7.1.0", true),
        case("1.7.*|1.8.*", true),
        case(">1.7,<1.8", true),
        case(">1.7.1,<1.8", false),
        case("^1.7.1$", true),
        case(r"^1\.7\.1$", true),
        case(r"^1\.7\.[0-9]+$", true),
        case(r"^1\.[5-8]\.1$", true),
        case(r"^[^1].*$", false),
        case(r"^[0-9+]+\.[0-9+]+\.[0-9]+$", true),
        case("^$", false),
        case("^.*$", true),
        case("1.7.*|^0.*$", true),
        case("1.6.*|^0.*$", false),
        case("1.6.*|^0.*$|1.7.1", true),
        case("^0.*$|1.7.1", true),
        case(r"1.6.*|^.*\.7\.1$|0.7.1", true),
        case("*", true),
        case("1.*.1", true),
        case("1.5.*|>1.7,<1.8", true),
        case("1.5.*|>1.7,<1.7.1", false)
    )]
    fn test_match(vspec: &str, res: bool) {
        let m = VersionSpec::try_from(vspec).unwrap();
        //assert VersionSpec(m) is m
        //assert str(m) == vspec
        //assert repr(m) == "VersionSpec('%s')" % vspec
        assert_eq!(m.test_match("1.7.1"), res);
    }

    #[rstest(vspec,
        case("1.7.0"),
        case("1.7.0.post123"),
        case("1.7.0.post123.gabcdef9"),
        case("1.7.0.post123 + gabcdef9")
    )]
    fn test_local_identifier(vspec: &str) {
        //"""The separator for the local identifier should be either `.` or `+`"""
        // a valid versionstr should match itself
        let m = VersionSpec::try_from(vspec).unwrap();
        assert!(m.test_match (vspec))
    }

    #[test]
    fn test_not_eq_star() {
        assert_eq!(VersionSpec::try_from("=3.3").unwrap().test_match("3.3.1"), true);
        assert_eq!(VersionSpec::try_from("=3.3").unwrap().test_match("3.3"), true);
        assert_eq!(VersionSpec::try_from("=3.3").unwrap().test_match("3.4"), false);

        assert_eq!(VersionSpec::try_from("3.3.*").unwrap().test_match("3.3.1"), true);
        assert_eq!(VersionSpec::try_from("3.3.*").unwrap().test_match("3.3"), true);
        assert_eq!(VersionSpec::try_from("3.3.*").unwrap().test_match("3.4"), false);

        assert_eq!(VersionSpec::try_from("=3.3.*").unwrap().test_match("3.3.1"), true);
        assert_eq!(VersionSpec::try_from("=3.3.*").unwrap().test_match("3.3"), true);
        assert_eq!(VersionSpec::try_from("=3.3.*").unwrap().test_match("3.4"), false);

        assert_eq!(VersionSpec::try_from("!=3.3.*").unwrap().test_match("3.3.1"), false);
        assert_eq!(VersionSpec::try_from("!=3.3.*").unwrap().test_match("3.4"), true);
        assert_eq!(VersionSpec::try_from("!=3.3.*").unwrap().test_match("3.4.1"), true);

        assert_eq!(VersionSpec::try_from("!=3.3").unwrap().test_match("3.3.1"), true);
        assert_eq!(VersionSpec::try_from("!=3.3").unwrap().test_match("3.3.0.0"), false);
        assert_eq!(VersionSpec::try_from("!=3.3.*").unwrap().test_match("3.3.0.0"), false);
    }

    #[test]
    fn test_compound_versions() {
        let vs = VersionSpec::try_from(">=2.7, !=3.0.*, !=3.1.*, !=3.2.*, !=3.3.*").unwrap();
        assert_eq!(vs.test_match("2.6.8"), false);
        assert!(vs.test_match("2.7.2"));
        assert_eq!(vs.test_match("3.3"), false);
        assert_eq!(vs.test_match("3.3.4"), false);
        assert!(vs.test_match("3.4"));
        assert!(vs.test_match("3.4a"));
    }

    #[test]
    fn test_invalid_version_specs() {
        match VersionSpec::try_from("~") {
            Ok(_) => panic!(),
            _ => true
        };
        match VersionSpec::try_from("^") {
            Ok(_) => panic!(),
            _ => true
        };
    }

    #[test]
    fn test_compatible_release_versions() {
        assert_eq!(VersionSpec::try_from("~=1.10") .unwrap().test_match("1.11.0"), true);
        assert_eq!(VersionSpec::try_from("~=1.10.0").unwrap().test_match("1.11.0"), false);

        assert_eq!(VersionSpec::try_from("~=3.3.2").unwrap().test_match("3.4.0"), false);
        assert_eq!(VersionSpec::try_from("~=3.3.2").unwrap().test_match("3.3.1"), false);
        assert_eq!(VersionSpec::try_from("~=3.3.2").unwrap().test_match("3.3.2.0"), true);
        assert_eq!(VersionSpec::try_from("~=3.3.2").unwrap().test_match("3.3.3"), true);

        assert_eq!(VersionSpec::try_from("~=3.3.2|==2.2").unwrap().test_match("2.2.0"), true);
        assert_eq!(VersionSpec::try_from("~=3.3.2|==2.2").unwrap().test_match("3.3.3"), true);
        assert_eq!(VersionSpec::try_from("~=3.3.2|==2.2").unwrap().test_match("2.2.1"), false);

        match VersionSpec::try_from("~=3.3.2.*") {
            Ok(_) => panic!(),
            _ => true
        };
    }

    #[test]
    fn test_pep_440_arbitrary_equality_operator() {
        // We're going to leave the not implemented for now.
        match VersionSpec::try_from("===3.3.2.*") {
            Ok(_) => panic!(),
            _ => true
        };
     }
}