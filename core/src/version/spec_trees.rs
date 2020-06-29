use std::ops::Deref;
use std::fmt;
use regex::Regex;
use std::borrow::Borrow;

use serde::export::TryFrom;

use crate::version::matching::{MatchEnum, MatchFn, get_matcher};
use crate::version::Version;
use crate::version::errors::VersionParsingError;

#[enum_dispatch]
pub trait Spec {
    // properties in Python
    fn raw_value(&self) -> String { self.get_spec() }
    fn exact_value(&self) -> Option<String> {
        if self.is_exact() { Some(self.get_spec()) } else { None } }

    // properties in Python (to be implemented by other things)
    fn get_spec(&self) -> String;
    fn is_exact(&self) -> bool;
    // easier to use, because you can pass text
    fn test_match(&self, other: &str) -> bool;
    // faster, because you don't create Version objects
    fn test_match_version(&self, other: &Version) -> bool;
}

#[enum_dispatch(Spec)]
#[derive(Clone)]
pub enum VersionSpecOrConstraintTree {
    VersionSpec(VersionSpec),
    ConstraintTree(ConstraintTree), // vec is a mix of &str or other vector(s) of str, possibly nested
}

#[derive(Clone)]
pub struct ConstraintTree {
    pub combinator: Combinator,
    pub parts: Vec<VersionSpecOrConstraintTree>,
}

// Not sure tw
impl Spec for ConstraintTree{
    fn get_spec(&self) -> String {
        untreeify(&VersionSpecOrConstraintTree::ConstraintTree(self.clone())).unwrap()
    }
    fn is_exact(&self) -> bool {
        return false
    }
    fn test_match(&self, other: &str) -> bool {
        let v: Version = other.into();
        self.test_match_version(&v)
    }
    fn test_match_version(&self, other: &Version) -> bool {
        return match self.combinator {
            Combinator::And => self.parts.iter().all(|x| x.borrow().test_match_version(other)),
            Combinator::Or => self.parts.iter().any(|x| x.borrow().test_match_version(other)),
            Combinator::None => false,
        }
    }
}

#[derive(PartialEq, Clone)]
pub enum Combinator {
    Or,
    And,
    None
}

impl ConstraintTree {
    fn combine(&self, inand: bool, nested: bool) -> Result<String, String> {
        match self.parts.len() {
            1 => {
                if let VersionSpecOrConstraintTree::VersionSpec(s) = self.parts[0].borrow() {
                    Ok(s.get_spec().to_string())
                } else {
                    Err("Can't combine (stringify) single-element ConstraintTree that isn't just a string".to_string())
                }
            },
            0 => Err("Can't combine (stringify) a zero-element ConstraintTree".to_string()),
            _ => {
                let mut str_parts = vec![];

                for item in &self.parts {
                    str_parts.push(match item {
                        VersionSpecOrConstraintTree::VersionSpec(s) => s.get_spec().to_string(),
                        VersionSpecOrConstraintTree::ConstraintTree(cj) => {
                            cj.combine(self.combinator == Combinator::And, true).unwrap()
                        }
                    });
                }

                let mut res = match self.combinator {
                    Combinator::And => str_parts.join(","),
                    _ => str_parts.join("|")
                };
                if inand || nested {
                    res = format!("({})", res);
                }
                Ok(res)
            }
        }
    }
}

impl TryFrom<&str> for VersionSpecOrConstraintTree {
    type Error = VersionParsingError;
    fn try_from (input: &str) -> Result<VersionSpecOrConstraintTree, Self::Error> {
        lazy_static! { static ref REGEX_SPLIT_RE: Regex = Regex::new( r#".*[()|,^$]"# ).unwrap(); }
        let split_input: Vec<&str> = REGEX_SPLIT_RE.split(input).collect();
        if split_input.len() > 1 {
            match ConstraintTree::try_from(split_input) {
                Ok(v) => Ok(VersionSpecOrConstraintTree::ConstraintTree(v)),
                Err(e) => Err(e)
            }
        } else {
            match VersionSpec::try_from(input) {
                Ok(v) => Ok(VersionSpecOrConstraintTree::VersionSpec(v)),
                Err(e) => Err(e)
            }
        }
    }
}

impl PartialEq for VersionSpecOrConstraintTree {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (VersionSpecOrConstraintTree::VersionSpec(a), VersionSpecOrConstraintTree::VersionSpec(b)) => a == b,
            (VersionSpecOrConstraintTree::ConstraintTree(a), VersionSpecOrConstraintTree::ConstraintTree(b)) => a == b,
            _ => false
        }
    }
}

impl fmt::Debug for VersionSpecOrConstraintTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl PartialEq for ConstraintTree {
    fn eq(&self, other: &Self) -> bool {
        for (a, b) in self.parts.iter().zip(&other.parts) {
            if a.deref() != b.deref() { return false }
        }
        return true
    }
}

impl TryFrom<Vec<&str>> for ConstraintTree
{
    type Error = VersionParsingError;
    fn try_from(input: Vec<&str>) -> Result<Self, Self::Error> {
        let combinator = match input[0].borrow() {
            "," => Combinator::And,
            "|" => Combinator::Or,
            _ => return Err(VersionParsingError::Message(format!("Unknown first value in vec of str used as ConstraintTree")))
        };
        let tree = ConstraintTree {
            combinator,
            parts: input[1..].iter().map(|x| VersionSpecOrConstraintTree::try_from(x.clone()).unwrap()).collect()
        };
        Ok(tree)
    }
}

impl fmt::Debug for ConstraintTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            write!(f, "{:#?}{:#?}", self.combinator, self.parts)
        } else {
            write!(f, "{:?}", self.get_spec())
        }
    }
}

/// Given a single spec or collection of specs, join them together into a string that captures
///   then relationships among specs
///
/// # Examples
///
/// ```
/// use ronda::{untreeify, ConstraintTree, VersionSpec, VersionSpecOrConstraintTree, Combinator};
/// use std::convert::{TryInto, TryFrom};
///
/// let cj123_456: ConstraintTree = vec![",", "1.2.3", "4.5.6"].try_into().unwrap();
/// let v = untreeify(&"1.2.3".into());
/// assert_eq!(v.unwrap(), "1.2.3".to_string());
/// let v = untreeify(&ConstraintTree::try_from(vec![",", "1.2.3", ">4.5.6"]).unwrap().into());
/// assert_eq!(v.unwrap(), "1.2.3,>4.5.6".to_string());
/// let tree: VersionSpecOrConstraintTree = ConstraintTree {
///                               combinator: Combinator::Or,
///                               parts: vec![
///                                     VersionSpecOrConstraintTree::ConstraintTree(cj123_456),
///                                     VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_into("<=7.8.9").unwrap())]}.into();
/// let v = untreeify(&tree);
/// assert_eq!(v.unwrap(), "(1.2.3,4.5.6)|<=7.8.9".to_string());
/// ```
pub fn untreeify(spec: &VersionSpecOrConstraintTree) -> Result<String, String> {
    match spec {
        VersionSpecOrConstraintTree::ConstraintTree(s) => {s.combine(false, false)},
        VersionSpecOrConstraintTree::VersionSpec(s) => Ok(s.get_spec().to_string())
    }
}

impl From<&str> for Combinator {
    fn from(input: &str) -> Combinator {
        match input {
            "," => Combinator::And,
            "|" => Combinator::Or,
            _ => Combinator::None
        }
    }
}

impl fmt::Debug for Combinator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Combinator::And => write!(f, "&"),
            Combinator::Or => write!(f, "|"),
            Combinator::None => write!(f, ""),
        }
    }
}

fn _apply_ops(cstop: &str, output: &mut ConstraintTree, stack: &mut Vec<&str>) -> Result<(), String> {
    // cstop: operators with lower precedence
    while stack.len() > 0 && ! cstop.contains(stack.last().unwrap()) {
        // Fuse expressions with the same operator; e.g.,
        //   ('|', ('|', a, b), ('|', c, d))becomes
        //   ('|', a, b, c d)
        if output.parts.len() < 2 {
            return Err("can't join single expression".to_string())
        }
        let c: Combinator = stack.pop().unwrap().into();
        let mut condensed: Vec<VersionSpecOrConstraintTree> = vec![];

        for _ in 0..2 {
            match output.parts.pop().unwrap() {
                VersionSpecOrConstraintTree::ConstraintTree(a) => {
                    if a.combinator == c {
                        condensed = a.clone().parts.into_iter().chain(condensed.into_iter()).collect();
                    } else {
                        condensed.insert(0,VersionSpecOrConstraintTree::ConstraintTree(a.clone()))
                    }
                },
                VersionSpecOrConstraintTree::VersionSpec(a) => condensed.insert(0,
                                                                           VersionSpecOrConstraintTree::VersionSpec(a.to_owned()))
            }
        }

        let condensed_output = ConstraintTree {
            combinator: c,
            parts: condensed
        };

        if output.parts.len() > 0 {
            output.parts.push(VersionSpecOrConstraintTree::ConstraintTree(condensed_output));
        } else {
            *output = condensed_output;
        }
    }
    return Ok(())
}

fn _treeify(spec_str: String) -> Result<ConstraintTree, String> {
    lazy_static! { static ref VSPEC_TOKENS: Regex = Regex::new(
        r#"\s*\^[^$]*[$]|\s*[()|,]|\s*[^()|,]+"#
    ).unwrap(); }
    //let delimiters: &str = "|,()";
    let mut output: ConstraintTree = ConstraintTree { combinator: Combinator::None, parts: vec![]};
    let mut stack: Vec<&str> =vec![];

    let spec_str_in_parens = format!("({})", spec_str);
    let tokens: Vec<&str> = VSPEC_TOKENS.find_iter(&spec_str_in_parens).map(|x| x.as_str().trim()).collect();

    for item in tokens {
        match item {
            "(" => { stack.push("(") },
            "|" => {
                _apply_ops("(", &mut output, &mut stack)?;
                stack.push("|");
            },
            "," => {
                _apply_ops("|(", &mut output, &mut stack)?;
                stack.push(",");
            },
            ")" => {
                _apply_ops("(", &mut output, &mut stack)?;
                if stack.is_empty() || *stack.last().unwrap() != "(" {
                    return Err("expression must start with \"(\"".to_string());
                }
                stack.pop();
            },
            _ => {
                if output.combinator != Combinator::None {
                    output = ConstraintTree {
                        combinator: Combinator::None,
                        parts: vec![VersionSpecOrConstraintTree::ConstraintTree(output)]};
                }
                output.parts.push(VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_from(item).unwrap()))
            }
        }
    }

    if ! stack.is_empty() { return Err(format!("unable to convert ({}) to expression tree: {:#?}", spec_str, stack)); }
    Ok(output)
}

/// Examples:
/// ```
/// use ronda::{treeify, ConstraintTree, VersionSpec, VersionSpecOrConstraintTree, Combinator};
///
///  let v = treeify(String::from("((1.5|((1.6|1.7), 1.8), 1.9 |2.0))|2.1").unwrap();
///  assert_eq!(v, ConstraintTree {
///                  combinator: Combinator::Or,
///                  parts: vec![
///      VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_from("1.5").unwrap()),
///      VersionSpecOrConstraintTree::ConstraintTree(ConstraintTree {
///                   combinator: Combinator::And,
///                   parts: vec![
///           VersionSpecOrConstraintTree::ConstraintTree(ConstraintTree {
///                       combinator: Combinator::Or,
///                       parts: vec![
///               VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_from("1.6").unwrap()),
///               VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_from("1.7").unwrap()),
///           ]}),
///           VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_from("1.8").unwrap()),
///           VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_from("1.9").unwrap()),
///      ]}),
///      VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_from("2.0").unwrap()),
///      VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_from("2.1").unwrap()),
///  ]});
///  ```
pub fn treeify(spec_str: &str) -> Result<ConstraintTree, String> {
    _treeify(spec_str.to_string())
}

#[derive(Clone)]
pub struct VersionSpec {
    spec_str: String,
    matcher: MatchEnum,
    _is_exact: bool
}

impl PartialEq for VersionSpec {
    fn eq(&self, other: &Self) -> bool {
        return self.spec_str == other.spec_str
    }
}

impl Spec for VersionSpec {
    fn get_spec(&self) -> String { self.spec_str.clone() }
    fn is_exact(&self) -> bool { self._is_exact }
    fn test_match(&self, other: &str) -> bool {
        let v: Version = other.into();
        self.test_match_version(&v)
    }
    fn test_match_version(&self, other: &Version) -> bool {
        self.matcher.test(other)
    }
}

impl TryFrom<&str> for VersionSpec {
    type Error = VersionParsingError;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        let res = get_matcher(input);
        match res {
            Ok((matcher, _is_exact)) => Ok(VersionSpec { spec_str: input.to_string(), matcher, _is_exact }),
            Err(e) => Err(e)
        }
    }
}



#[cfg_attr(tarpaulin, skip)]
#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;
    use rstest::rstest;

    #[test]
    fn untreeify_single() {
        let ct: VersionSpecOrConstraintTree = "1.2.3".try_into().unwrap();
        let v = untreeify(&ct).unwrap();
        assert_eq!(v, "1.2.3");
    }

    #[test]
    fn untreeify_simple_and() {
        let ct: ConstraintTree = vec![",", "1.2.3", ">4.5.6"].try_into().unwrap();
        let v = untreeify(&ct.into()).unwrap();
        assert_eq!(v, "1.2.3,>4.5.6");
    }

    #[test]
    fn untreeify_simple_or() {
        let ct: ConstraintTree = vec!["|", "1.2.3", ">4.5.6"].try_into().unwrap();
        let v = untreeify(&ct.into()).unwrap();
        assert_eq!(v, "1.2.3|>4.5.6");
    }

    #[test]
    fn untreeify_and_joining_inner_or() {
        let inner_or: ConstraintTree = vec!["|", "1.2.3", "4.5.6"].try_into().unwrap();
        let v = untreeify(&ConstraintTree {
            combinator: Combinator::And,
            parts: vec![
                VersionSpecOrConstraintTree::ConstraintTree(inner_or),
                VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_from("<=7.8.9").unwrap()),
            ]
        }.into()).unwrap();
        assert_eq!(v, "(1.2.3|4.5.6),<=7.8.9");
    }

    #[test]
    fn untreeify_nested() {
        let or_6_7: ConstraintTree = vec!["|", "1.6", "1.7"].try_into().unwrap();
        let or_6_7_and_8_9: ConstraintTree = ConstraintTree{
            combinator: Combinator::And,
            parts: vec![
                VersionSpecOrConstraintTree::ConstraintTree(or_6_7),
                VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_from("1.8").unwrap()),
                VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_from("1.9").unwrap()),
            ]};
        let or_with_inner_group: ConstraintTree = ConstraintTree {
            combinator: Combinator::Or,
            parts: vec![
                VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_from("1.5").unwrap()),
                VersionSpecOrConstraintTree::ConstraintTree(or_6_7_and_8_9),
                VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_from("2.0").unwrap()),
                VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_from("2.1").unwrap()),
            ]};
        let v = untreeify(&or_with_inner_group.into()).unwrap();
        assert_eq!(v, "1.5|((1.6|1.7),1.8,1.9)|2.0|2.1");
    }

    #[test]
    fn treeify_single() {
        let v = treeify("1.2.3").unwrap();
        assert_eq!(v, ConstraintTree {
            combinator: Combinator::None,
            parts: vec![
                VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_from("1.2.3").unwrap()),
            ]
        });
    }

    #[test]
    fn treeify_simple_and() {
        let v = treeify("1.2.3,>4.5.6").unwrap();
        assert_eq!(v, ConstraintTree {
            combinator: Combinator::And,
            parts: vec![
                VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_from("1.2.3").unwrap()),
                VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_from(">4.5.6").unwrap()),
            ]
        }, "{:#?}", v);
    }

    #[test]
    fn treeify_and_or_grouping() {
        let v = treeify("1.2.3,4.5.6|<=7.8.9").unwrap();
        assert_eq!(v, ConstraintTree {
            combinator: Combinator::Or,
            parts: vec![
                VersionSpecOrConstraintTree::ConstraintTree(ConstraintTree {
                    combinator: Combinator::And,
                    parts: vec![
                        VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_from("1.2.3").unwrap()),
                        VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_from("4.5.6").unwrap()),
                    ]
                }),
                VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_from("<=7.8.9").unwrap()),
            ]
        }, "{:#?}", v);
    }

    #[test]
    fn treeify_and_with_or_in_parens() {
        let v = treeify("(1.2.3|4.5.6),<=7.8.9").unwrap();
        assert_eq!(v, ConstraintTree {
            combinator: Combinator::And,
            parts: vec![
                VersionSpecOrConstraintTree::ConstraintTree(ConstraintTree {
                    combinator: Combinator::Or,
                    parts: vec![
                        VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_from("1.2.3").unwrap()),
                        VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_from("4.5.6").unwrap()),
                    ]
                }),
                VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_from("<=7.8.9").unwrap()),
            ]
        }, "{:#?}", v);
    }

    #[test]
    fn treeify_nest_or_in_parens() {
        let v = treeify("((1.5|((1.6|1.7), 1.8), 1.9 |2.0))|2.1").unwrap();

        assert_eq!(v, ConstraintTree {
            combinator: Combinator::Or,
            parts: vec![
                VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_from("1.5").unwrap()),
                VersionSpecOrConstraintTree::ConstraintTree(ConstraintTree {
                    combinator: Combinator::And,
                    parts: vec![
                        VersionSpecOrConstraintTree::ConstraintTree(ConstraintTree {
                            combinator: Combinator::Or,
                            parts: vec![
                                VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_from("1.6").unwrap()),
                                VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_from("1.7").unwrap()),
                            ]}),
                        VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_from("1.8").unwrap()),
                        VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_from("1.9").unwrap()),
                    ]}),
                VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_from("2.0").unwrap()),
                VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_from("2.1").unwrap()),
            ]}, "{:#?}", v);
    }

    #[test]
    fn treeify_nest_or_in_parens_2() {
        let v = treeify("1.5|(1.6|1.7),1.8,1.9|2.0|2.1").unwrap();
        assert_eq!(v, ConstraintTree {
            combinator: Combinator::Or,
            parts: vec![
                VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_from("1.5").unwrap()),
                VersionSpecOrConstraintTree::ConstraintTree(ConstraintTree {
                    combinator: Combinator::And,
                    parts: vec![
                        VersionSpecOrConstraintTree::ConstraintTree(ConstraintTree {
                            combinator: Combinator::Or,
                            parts: vec![
                                VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_from("1.6").unwrap()),
                                VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_from("1.7").unwrap()),
                            ]}),
                        VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_from("1.8").unwrap()),
                        VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_from("1.9").unwrap()),
                    ]}),
                VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_from("2.0").unwrap()),
                VersionSpecOrConstraintTree::VersionSpec(VersionSpec::try_from("2.1").unwrap()),
            ]}, "{:#?}", v);
    }

    #[test]
    fn test_ver_eval() {
        assert_eq!(VersionSpec::try_from("==1.7").unwrap().test_match("1.7.0"), true);
        assert_eq!(VersionSpec::try_from("<=1.7").unwrap().test_match("1.7.0"), true);
        assert_eq!(VersionSpec::try_from("<1.7").unwrap().test_match("1.7.0"), false);
        assert_eq!(VersionSpec::try_from(">=1.7").unwrap().test_match("1.7.0"), true);
        assert_eq!(VersionSpec::try_from(">1.7").unwrap().test_match("1.7.0"), false);
        assert_eq!(VersionSpec::try_from(">=1.7").unwrap().test_match("1.6.7"), false);
        assert_eq!(VersionSpec::try_from(">2013b").unwrap().test_match("2013a"), false);
        assert_eq!(VersionSpec::try_from(">2013b").unwrap().test_match("2013k"), true);
        assert_eq!(VersionSpec::try_from(">2013b").unwrap().test_match("3.0.0"), false);
        assert_eq!(VersionSpec::try_from(">1.0.0a").unwrap().test_match("1.0.0"), true);
        assert_eq!(VersionSpec::try_from(">1.0.0*").unwrap().test_match("1.0.0"), true);
        assert_eq!(VersionSpec::try_from("1.0*").unwrap().test_match("1.0"), true);
        assert_eq!(VersionSpec::try_from("1.0*").unwrap().test_match("1.0.0"), true);
        assert_eq!(VersionSpec::try_from("1.0.0*").unwrap().test_match("1.0"), true);
        assert_eq!(VersionSpec::try_from("1.0.0*").unwrap().test_match("1.0.1"), false);
        assert_eq!(VersionSpec::try_from("2013a*").unwrap().test_match("2013a"), true);
        assert_eq!(VersionSpec::try_from("2013b*").unwrap().test_match("2013a"), false);
        assert_eq!(VersionSpec::try_from("2013a*").unwrap().test_match("2013ab"), true);
        assert_eq!(VersionSpec::try_from("1.2.4*").unwrap().test_match("1.3.4"), false);
        assert_eq!(VersionSpec::try_from("1.2.3*").unwrap().test_match("1.2.3+4.5.6"), true);
        assert_eq!(VersionSpec::try_from("1.2.3+4*").unwrap().test_match("1.2.3+4.5.6"), true);
        assert_eq!(VersionSpec::try_from("1.2.3+5*").unwrap().test_match("1.2.3+4.5.6"), false);
        assert_eq!(VersionSpec::try_from("1.2.4+5*").unwrap().test_match("1.2.3+4.5.6"), false);
    }

    #[test]
    fn test_ver_eval_errors() {
        // each of these should raise
        VersionSpec::try_from("><2.4.5");
        VersionSpec::try_from("!!2.4.5");
        VersionSpec::try_from("!");
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
    fn test_local_identifier(vspec: &'static str) {
        //"""The separator for the local identifier should be either `.` or `+`"""
        // a valid versionstr should match itself
        let m = VersionSpec::try_from(vspec).unwrap();
        assert!(m.test_match (vspec.into()))
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
        assert_eq!(VersionSpec::try_from("~=1.10").unwrap().test_match("1.11.0"), true);
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