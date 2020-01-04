use std::ops::Deref;
use std::fmt;
use regex::Regex;
use serde::export::TryFrom;
use std::convert::TryInto;

#[derive(Clone)]
pub enum StringOrConstraintTree {
    String(String),
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
    fn combine(&self, inand:bool, nested: bool) -> Result<String, &'static str> {
        let mut res: String = "".to_string();
        match self.parts.len() {
            1 => {
                res = if let StringOrConstraintTree::String(s) = self.parts[0].deref() {
                    s.to_string() } else { Err("Can't combine (stringify) single-element ConstraintTree that isn't just a string") };
            },
            0 => Err("Can't combine (stringify) a zero-element ConstraintTree"),
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

                if self.combinator == Combinator::And {
                    res = str_parts.join(",");
                } else {
                    res = str_parts.join("|");
                }
                if inand || nested {
                    res = format!("({})", res);
                }
            }
        }
        Ok(res)
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
    fn try_from(input: &str) -> Result<Self, Self::Error> {
        let combinator = match s[0]{
            "," => Combinator::And,
            "|" => Combinator::Or,
            _ => return Err("Unknown first value in vec of str used as ConstraintTree")
        };
        Ok(ConstraintTree {
            combinator,
            parts: s[1..].iter().map(|x| Box::new(StringOrConstraintTree::String(x.to_string()))).collect()
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
/// use std::convert::TryInto;
///
/// let cj123_456: ConstraintTree = vec![",", "1.2.3", "4.5.6"].try_into().unwrap();
/// let v = untreeify("1.2.3".try_into().unwrap());
/// assert_eq!(v, "1.2.3");
/// let v = untreeify(vec![",", "1.2.3", ">4.5.6"].try_into().unwrap());
/// assert_eq!(v, "1.2.3,>4.5.6");
/// let tree: ConstraintTree = ConstraintTree {
///                               combinator: Combinator::Or,
///                               parts: vec![
///                                     Box::new(StringOrConstraintTree::ConstraintTree(cj123_456)),
///                                     Box::new(StringOrConstraintTree::String("<=7.8.9".to_string()))]};
/// let v = untreeify(tree);
/// assert_eq!(v, "(1.2.3,4.5.6)|<=7.8.9");
/// ```
pub fn untreeify(spec: &ConstraintTree) -> Result<String, &'static str> {
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

fn _apply_ops(cstop: &str, output: &mut ConstraintTree, stack: &mut Vec<&str>) -> Result<(), &'static str> {
    // cstop: operators with lower precedence
    while stack.len() > 0 && ! cstop.contains(stack.last().unwrap()) {
        // Fuse expressions with the same operator; e.g.,
        //   ('|', ('|', a, b), ('|', c, d))becomes
        //   ('|', a, b, c d)
        if output.parts.len() < 2 {
            return Err("can't join single expression")
        }
        let c: Combinator = stack.pop().unwrap().into();
        let mut condensed: Vec<Box<StringOrConstraintTree>> = vec![];

        for _ in 0..2 {
            match output.parts.pop().unwrap().deref() {
                StringOrConstraintTree::ConstraintTree(a) => {
                    if a.combinator == c {
                        condensed = a.clone().parts.into_iter().chain(condensed.into_iter()).collect();
                    } else {
                        condensed.insert(0,(Box::new(StringOrConstraintTree::ConstraintTree(a.clone()))))
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
pub fn treeify(spec_str: &str) -> Result<ConstraintTree, &'static str> {
    lazy_static! { static ref VSPEC_TOKENS: Regex = Regex::new(
        r#"\s*\^[^$]*[$]|\s*[()|,]|\s*[^()|,]+"#
    ).unwrap(); }
    let delimiters: &str = "|,()";
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
                    return Err("expression must start with \"(\"");
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

    if ! stack.is_empty() { return Err(&format!("unable to convert ({}) to expression tree: {:#?}", spec_str, stack)); }
    Ok(output)
}


#[cfg_attr(tarpaulin, skip)]
#[cfg(test)]
mod tests {
    use super::*;

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
}