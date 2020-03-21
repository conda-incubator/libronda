use std::ops::Deref;
use std::fmt;
use regex::Regex;
use serde::export::TryFrom;
use std::collections::HashSet;


use crate::CompOp;
use crate::version::matching::*;
use std::borrow::Borrow;


#[derive(Clone)]
pub enum StringOrConstraintTree {
    String(String),   // a string representation of a matchspec
    ConstraintTree(ConstraintTree), // vec is a mix of &str or other vector(s) of str, possibly nested
}

#[derive(Clone)]
pub struct ConstraintTree {
    pub combinator: Combinator,
    pub parts: Vec<Box<StringOrConstraintTree>>,
}

#[derive(PartialEq, Clone)]
pub enum Combinator {
    Or,
    And,
    None
}

impl ConstraintTree {
    fn combine(&self, inand:bool, nested: bool) -> Result<String, String> {
        match self.parts.len() {
            1 => {
                if let StringOrConstraintTree::String(s) = self.parts[0].deref() {
                    Ok(s.to_string())
                } else {
                    Err("Can't combine (stringify) single-element ConstraintTree that isn't just a string".to_string())
                }
            },
            0 => Err("Can't combine (stringify) a zero-element ConstraintTree".to_string()),
            _ => {
                let mut str_parts = vec![];

                for item in &self.parts {
                    str_parts.push(match item.deref() {
                        StringOrConstraintTree::String(s) => s.deref().to_string(),
                        StringOrConstraintTree::ConstraintTree(cj) => {
                            cj.combine(self.combinator == Combinator::And, true)?
                        }
                    });
                }

                let mut res = match self.combinator {
                    Combinator::And => str_parts.join(","),
                    _ =>  str_parts.join("|")
                };
                if inand || nested {
                    res = format!("({})", res);
                }
                Ok(res)
            }
        }
    }

    pub(crate) fn evaluate(&self, other: &str) -> bool {
        fn _eval_part(a: &StringOrConstraintTree, b: &str) -> bool {
            return match a {
                StringOrConstraintTree::String(val) => VersionSpec::try_from(val.borrow()).unwrap().test_match(b),
                StringOrConstraintTree::ConstraintTree(val) => val.evaluate(b)
            }
        }
        return match self.combinator {
            Combinator::And => self.parts.iter().all(|x| _eval_part(x.borrow(), other)),
            Combinator::Or => self.parts.iter().any(|x| _eval_part(x.borrow(), other)),
            _ => panic!()
        }
    }
}

impl From<&str> for StringOrConstraintTree {
    fn from (s: &str) -> StringOrConstraintTree {
        StringOrConstraintTree::String(s.to_string())
    }
}

impl PartialEq for StringOrConstraintTree {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (StringOrConstraintTree::String(a), StringOrConstraintTree::String(b)) => a == b,
            (StringOrConstraintTree::ConstraintTree(a), StringOrConstraintTree::ConstraintTree(b)) => a == b,
            _ => false
        }
    }
}

impl fmt::Debug for StringOrConstraintTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StringOrConstraintTree::String(a) => write!(f, "{:#?}", a),
            StringOrConstraintTree::ConstraintTree(a) => write!(f, "{:#?}", a),
        }
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

impl From<&str> for ConstraintTree {
    fn from (s: &str) -> ConstraintTree {
        ConstraintTree {
            combinator: Combinator::None,
            parts: vec![Box::new(StringOrConstraintTree::String(s.to_string()))]}
    }
}

impl From<StringOrConstraintTree> for ConstraintTree {
    fn from (scj: StringOrConstraintTree) -> ConstraintTree {
        match scj {
            StringOrConstraintTree::ConstraintTree(_scj) => _scj,
            _ => ConstraintTree { combinator: Combinator::None, parts: vec![Box::new(scj)] },
        }
    }
}

impl TryFrom<Vec<&str>> for ConstraintTree
{
    type Error = &'static str;
    fn try_from(input: Vec<&str>) -> Result<Self, Self::Error> {
        let combinator = match input[0]{
            "," => Combinator::And,
            "|" => Combinator::Or,
            _ => return Err("Unknown first value in vec of str used as ConstraintTree")
        };
        Ok(ConstraintTree {
            combinator,
            parts: input[1..].iter().map(|x| Box::new(StringOrConstraintTree::String(x.to_string()))).collect()
        })
    }
}



impl fmt::Debug for ConstraintTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            write!(f, "{:#?}{:#?}", self.combinator, self.parts)
        } else {
            write!(f, "{:?}{:?}", self.combinator, self.parts)
        }
    }
}

/// Given a single spec or collection of specs, join them together into a string that captures
///   then relationships among specs
///
/// # Examples
///
/// ```
/// use ronda::{untreeify, ConstraintTree, StringOrConstraintTree, Combinator};
/// use std::convert::{TryInto, TryFrom};
///
/// let cj123_456: ConstraintTree = vec![",", "1.2.3", "4.5.6"].try_into().unwrap();
/// let v = untreeify(&"1.2.3".into());
/// assert_eq!(v.unwrap(), "1.2.3".to_string());
/// let v = untreeify(&ConstraintTree::try_from(vec![",", "1.2.3", ">4.5.6"]).unwrap());
/// assert_eq!(v.unwrap(), "1.2.3,>4.5.6".to_string());
/// let tree: ConstraintTree = ConstraintTree {
///                               combinator: Combinator::Or,
///                               parts: vec![
///                                     Box::new(StringOrConstraintTree::ConstraintTree(cj123_456)),
///                                     Box::new(StringOrConstraintTree::String("<=7.8.9".to_string()))]};
/// let v = untreeify(&tree);
/// assert_eq!(v.unwrap(), "(1.2.3,4.5.6)|<=7.8.9".to_string());
/// ```
pub fn untreeify(spec: &ConstraintTree) -> Result<String, String> {
    spec.combine(false, false)
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
        let mut condensed: Vec<Box<StringOrConstraintTree>> = vec![];

        for _ in 0..2 {
            match output.parts.pop().unwrap().deref() {
                StringOrConstraintTree::ConstraintTree(a) => {
                    if a.combinator == c {
                        condensed = a.clone().parts.into_iter().chain(condensed.into_iter()).collect();
                    } else {
                        condensed.insert(0,Box::new(StringOrConstraintTree::ConstraintTree(a.clone())))
                    }
                },
                StringOrConstraintTree::String(a) => condensed.insert(0,
                                                                      Box::new(StringOrConstraintTree::String(a.to_string())))
            }
        }

        let condensed_output = ConstraintTree {
            combinator: c,
            parts: condensed
        };

        if output.parts.len() > 0 {
            output.parts.push(Box::new(StringOrConstraintTree::ConstraintTree(condensed_output)));
        } else {
            *output = condensed_output;
        }
    }
    return Ok(())
}

/// Examples:
/// ```
/// use ronda::{treeify, ConstraintTree, StringOrConstraintTree, Combinator};
///
///  let v = treeify("((1.5|((1.6|1.7), 1.8), 1.9 |2.0))|2.1").unwrap();
///  assert_eq!(v, ConstraintTree {
///                  combinator: Combinator::Or,
///                  parts: vec![
///      Box::new(StringOrConstraintTree::String("1.5".to_string())),
///      Box::new(StringOrConstraintTree::ConstraintTree(ConstraintTree {
///                   combinator: Combinator::And,
///                   parts: vec![
///           Box::new(StringOrConstraintTree::ConstraintTree(ConstraintTree {
///                       combinator: Combinator::Or,
///                       parts: vec![
///               Box::new(StringOrConstraintTree::String("1.6".to_string())),
///               Box::new(StringOrConstraintTree::String("1.7".to_string())),
///           ]})),
///           Box::new(StringOrConstraintTree::String("1.8".to_string())),
///           Box::new(StringOrConstraintTree::String("1.9".to_string())),
///      ]})),
///      Box::new(StringOrConstraintTree::String("2.0".to_string())),
///      Box::new(StringOrConstraintTree::String("2.1".to_string())),
///  ]});
///  ```
pub fn treeify(spec_str: &str) -> Result<ConstraintTree, String> {
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
                        parts: vec![Box::new(StringOrConstraintTree::ConstraintTree(output))]};
                }
                output.parts.push(Box::new(StringOrConstraintTree::String(item.to_string())))
            }
        }
    }

    if ! stack.is_empty() { return Err(format!("unable to convert ({}) to expression tree: {:#?}", spec_str, stack)); }
    Ok(output)
}



#[derive(Clone)]
pub(crate) struct VersionSpec {
    spec_str: String,
    tree: Option<ConstraintTree>,
    matcher: MatchEnum,
    _is_exact: bool
}

impl Spec for VersionSpec {
    fn merge(&self, _other: &Self) -> Self { panic!("Not implemented") }
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
        let matcher: MatchEnum;
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

#[cfg_attr(tarpaulin, skip)]
#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;
    use rstest::rstest;

    #[test]
    fn untreeify_single() {
        let v = untreeify(&"1.2.3".try_into().unwrap()).unwrap();
        assert_eq!(v, "1.2.3");
    }

    #[test]
    fn untreeify_simple_and() {
        let v = untreeify(&vec![",", "1.2.3", ">4.5.6"].try_into().unwrap()).unwrap();
        assert_eq!(v, "1.2.3,>4.5.6");
    }

    #[test]
    fn untreeify_simple_or() {
        let v = untreeify(&vec!["|", "1.2.3", ">4.5.6"].try_into().unwrap()).unwrap();
        assert_eq!(v, "1.2.3|>4.5.6");
    }

    #[test]
    fn untreeify_and_joining_inner_or() {
        let inner_or: ConstraintTree = vec!["|", "1.2.3", "4.5.6"].try_into().unwrap();
        let v = untreeify(&ConstraintTree {
            combinator: Combinator::And,
            parts: vec![
                Box::new(StringOrConstraintTree::ConstraintTree(inner_or)),
                Box::new(StringOrConstraintTree::String("<=7.8.9".to_string())),
            ]
        }).unwrap();
        assert_eq!(v, "(1.2.3|4.5.6),<=7.8.9");
    }

    #[test]
    fn untreeify_nested() {
        let or_6_7: ConstraintTree = vec!["|", "1.6", "1.7"].try_into().unwrap();
        let or_6_7_and_8_9: ConstraintTree = ConstraintTree{
            combinator: Combinator::And,
            parts: vec![
                Box::new(StringOrConstraintTree::ConstraintTree(or_6_7)),
                Box::new(StringOrConstraintTree::String("1.8".to_string())),
                Box::new(StringOrConstraintTree::String("1.9".to_string())),
            ]};
        let or_with_inner_group: ConstraintTree = ConstraintTree {
            combinator: Combinator::Or,
            parts: vec![
                Box::new(StringOrConstraintTree::String("1.5".to_string())),
                Box::new(StringOrConstraintTree::ConstraintTree(or_6_7_and_8_9)),
                Box::new(StringOrConstraintTree::String("2.0".to_string())),
                Box::new(StringOrConstraintTree::String("2.1".to_string())),
            ]};
        let v = untreeify(&or_with_inner_group).unwrap();
        assert_eq!(v, "1.5|((1.6|1.7),1.8,1.9)|2.0|2.1");
    }

    #[test]
    fn treeify_single() {
        let v = treeify("1.2.3").unwrap();
        assert_eq!(v, ConstraintTree {
            combinator: Combinator::None,
            parts: vec![
                Box::new(StringOrConstraintTree::String("1.2.3".to_string()))]
        });
    }

    #[test]
    fn treeify_simple_and() {
        let v = treeify("1.2.3,>4.5.6").unwrap();
        assert_eq!(v, ConstraintTree {
            combinator: Combinator::And,
            parts: vec![
                Box::new(StringOrConstraintTree::String("1.2.3".to_string())),
                Box::new(StringOrConstraintTree::String(">4.5.6".to_string())),
            ]
        }, "{:#?}", v);
    }

    #[test]
    fn treeify_and_or_grouping() {
        let v = treeify("1.2.3,4.5.6|<=7.8.9").unwrap();
        assert_eq!(v, ConstraintTree {
            combinator: Combinator::Or,
            parts: vec![
                Box::new(StringOrConstraintTree::ConstraintTree(ConstraintTree {
                    combinator: Combinator::And,
                    parts: vec![
                        Box::new(StringOrConstraintTree::String("1.2.3".to_string())),
                        Box::new(StringOrConstraintTree::String("4.5.6".to_string())),
                    ]
                })),
                Box::new(StringOrConstraintTree::String("<=7.8.9".to_string())),
            ]
        }, "{:#?}", v);
    }

    #[test]
    fn treeify_and_with_or_in_parens() {
        let v = treeify("(1.2.3|4.5.6),<=7.8.9").unwrap();
        assert_eq!(v, ConstraintTree {
            combinator: Combinator::And,
            parts: vec![
                Box::new(StringOrConstraintTree::ConstraintTree(ConstraintTree {
                    combinator: Combinator::Or,
                    parts: vec![
                        Box::new(StringOrConstraintTree::String("1.2.3".to_string())),
                        Box::new(StringOrConstraintTree::String("4.5.6".to_string())),
                    ]
                })),
                Box::new(StringOrConstraintTree::String("<=7.8.9".to_string())),
            ]
        }, "{:#?}", v);
    }

    #[test]
    fn treeify_nest_or_in_parens() {
        let v = treeify("((1.5|((1.6|1.7), 1.8), 1.9 |2.0))|2.1").unwrap();

        assert_eq!(v, ConstraintTree {
            combinator: Combinator::Or,
            parts: vec![
                Box::new(StringOrConstraintTree::String("1.5".to_string())),
                Box::new(StringOrConstraintTree::ConstraintTree(ConstraintTree {
                    combinator: Combinator::And,
                    parts: vec![
                        Box::new(StringOrConstraintTree::ConstraintTree(ConstraintTree {
                            combinator: Combinator::Or,
                            parts: vec![
                                Box::new(StringOrConstraintTree::String("1.6".to_string())),
                                Box::new(StringOrConstraintTree::String("1.7".to_string())),
                            ]})),
                        Box::new(StringOrConstraintTree::String("1.8".to_string())),
                        Box::new(StringOrConstraintTree::String("1.9".to_string())),
                    ]})),
                Box::new(StringOrConstraintTree::String("2.0".to_string())),
                Box::new(StringOrConstraintTree::String("2.1".to_string())),
            ]}, "{:#?}", v);
    }

    #[test]
    fn treeify_nest_or_in_parens_2() {
        let v = treeify("1.5|(1.6|1.7),1.8,1.9|2.0|2.1").unwrap();
        assert_eq!(v, ConstraintTree {
            combinator: Combinator::Or,
            parts: vec![
                Box::new(StringOrConstraintTree::String("1.5".to_string())),
                Box::new(StringOrConstraintTree::ConstraintTree(ConstraintTree {
                    combinator: Combinator::And,
                    parts: vec![
                        Box::new(StringOrConstraintTree::ConstraintTree(ConstraintTree {
                            combinator: Combinator::Or,
                            parts: vec![
                                Box::new(StringOrConstraintTree::String("1.6".to_string())),
                                Box::new(StringOrConstraintTree::String("1.7".to_string())),
                            ]})),
                        Box::new(StringOrConstraintTree::String("1.8".to_string())),
                        Box::new(StringOrConstraintTree::String("1.9".to_string())),
                    ]})),
                Box::new(StringOrConstraintTree::String("2.0".to_string())),
                Box::new(StringOrConstraintTree::String("2.1".to_string())),
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