use std::ops::Deref;
use std::iter::Iterator;
use regex::Regex;
use std::borrow::BorrowMut;

pub trait Spec {
    // properties in Python
    fn spec(&self) -> &str;
    fn raw_value(&self) -> &str;
    fn exact_value(&self) -> Option<&str>;

    fn is_exact(&self) -> bool;
    fn regex_match(&self, spec_str: &str) -> bool;
    fn operator_match(&self, spec_str: &str) -> bool;
    fn any_match(&self, spec_str: &str) -> bool;
    fn all_match(&self, spec_str: &str) -> bool;
    fn exact_match(&self, spec_str: &str) -> bool;
    fn always_true_match(&self, _spec_str: &str) -> bool;

    // To be implemented by other things
    fn merge(&self, other: &impl Spec) -> Self;
}

//fn matcher_for_tuple(vspec) {
////    vspec_tree = vspec
////    _matcher = self.any_match if vspec_tree[0] == '|' else self.all_match
////    tup = tuple(VersionSpec(s) for s in vspec_tree[1:])
////    vspec_str = untreeify((vspec_tree[0],) + tuple(t.spec for t in tup))
////    self.tup = tup
////    matcher = _matcher
////    is_exact = False
////    return vspec_str, matcher, is_exact
//}

//impl Spec {
//    fn is_exact(&self) -> bool {
//
//    }
//    fn regex_match(&self, spec_str: &str) -> bool {
//
//    }
//    fn operator_match(&self, spec_str: &str) -> bool {
//
//    }
//    fn any_match(&self, spec_str: &str) -> bool {
//
//    }
//    fn all_match(&self, spec_str: &str) -> bool {
//
//    }
//    fn exact_match(&self, spec_str: &str) -> bool {
//
//    }
//    fn always_true_match(&self, _spec_str: &str) -> bool {
//        true
//    }
//
//}

pub enum VspecInputTypes {
    String,
    Tuple,
}

pub enum StringOrConstraintTree {
    String(String),
    ConstraintTree(ConstraintTree), // vec is a mix of &str or other vector(s) of str, possibly nested
}

pub struct ConstraintTree {
    pub parts: Vec<Box<StringOrConstraintTree>>,
}

impl ConstraintTree {
    fn isand(&self) -> bool {
        self.parts.len() > 0 && match self.parts[0].deref() {
            StringOrConstraintTree::String(s) => *s == ",",
            _ => false
        }
    }

    fn combine(&self, inand:bool, nested: bool) -> String {
        let mut res: String;
        match self.parts.len() {
            1 => {
                res = if let StringOrConstraintTree::String(s) = self.parts[0].deref() {
                    s.to_string()
                } else {panic!()};
                },
            0 => panic!(),
            _ => {
                let mut str_parts = vec![];

                for item in &self.parts[1..] {
                    str_parts.push(match item.deref() {
                        StringOrConstraintTree::String(s) => s.deref().to_string(),
                        StringOrConstraintTree::ConstraintTree(cj) => cj.combine(self.isand(), true)
                    });
                }

                if self.isand() {
                    res = str_parts.join(",");
                } else {
                    res = str_parts.join("|");
                }
                if inand || nested {
                    res = format!("({})", res);
                }
            }
        }
        res
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
        ConstraintTree { parts: vec![Box::new(StringOrConstraintTree::String(s.to_string()))]}
    }
}

impl From<StringOrConstraintTree> for ConstraintTree {
    fn from (scj: StringOrConstraintTree) -> ConstraintTree {
        match scj {
            StringOrConstraintTree::ConstraintTree(_scj) => _scj,
            _ => ConstraintTree { parts: vec![Box::new(scj)] },
        }
    }
}

impl From<Vec<&str>> for ConstraintTree
{
    fn from(s: Vec<&str>) -> ConstraintTree {
        ConstraintTree {
            parts: s.iter().map(|x| Box::new(StringOrConstraintTree::String(x.to_string()))).collect()
        }
    }
}

impl From<Vec<Box<StringOrConstraintTree>>> for ConstraintTree
{
    fn from (parts: Vec<Box<StringOrConstraintTree>>) -> ConstraintTree {
        ConstraintTree { parts }
    }
}

/// Given a single spec or collection of specs, join them together into a string that captures
///   then relationships among specs
///
/// # Examples
///
/// ```
/// use ronda::{untreeify, ConstraintTree, StringOrConstraintTree};
///
/// let cj123_456: ConstraintTree = vec![",", "1.2.3", "4.5.6"].into();
/// let v = untreeify("1.2.3".into());
/// assert_eq!(v, "1.2.3");
/// let v = untreeify(vec![",", "1.2.3", ">4.5.6"].into());
/// assert_eq!(v, "1.2.3,>4.5.6");
/// let tree: ConstraintTree = ConstraintTree {parts: vec![Box::new(StringOrConstraintTree::String("|".to_string())),
///                                                          Box::new(StringOrConstraintTree::ConstraintTree(cj123_456)),
///                                                          Box::new(StringOrConstraintTree::String("<=7.8.9".to_string()))]};
/// let v = untreeify(tree);
/// assert_eq!(v, "(1.2.3,4.5.6)|<=7.8.9");
/// ```
pub fn untreeify(spec: ConstraintTree) -> String {
    spec.combine(false, false)
}

fn _apply_ops(cstop: &str, output: &mut ConstraintTree, stack: &mut Vec<&str>) {
    // cstop: operators with lower precedence
    while stack.len() > 0 && ! cstop.contains(stack.last().unwrap()) {
        let c = stack.pop().unwrap();
        // Fuse expressions with the same operator; e.g.,
        //   ('|', ('|', a, b), ('|', c, d))becomes
        //   ('|', a, b, c d)
        // We're playing a bit of a trick here. Instead of checking
        // if the left or right entries are tuples, we're counting
        // on the fact that if we _do_ see a string instead, its
        // first character cannot possibly be equal to the operator.
        let mut condensed_parts: Vec<Box<StringOrConstraintTree>> = vec![];
        for part in output.parts {
            match part.deref() {
                StringOrConstraintTree::ConstraintTree(a) => {
                    if let StringOrConstraintTree::String(inner_a) = &a.parts[0].deref() {
                        if inner_a == c {
                            condensed_parts.append(&mut a.parts[1..].to_vec());
                        } else {
                            condensed_parts.push(part);
                        }
                    } else {
                        condensed_parts.push(part);
                    }
                },
                _ => condensed_parts.push(part),
            };
        };

        condensed_parts.insert(0, Box::new(StringOrConstraintTree::String(c.to_string())));
        *output = ConstraintTree { parts: condensed_parts };
    }
}


/// Examples:
/// ```
/// use ronda::{treeify, ConstraintTree, StringOrConstraintTree};
///
///  let v = treeify("((1.5|((1.6|1.7), 1.8), 1.9 |2.0))|2.1");
///  assert_eq!(v, ConstraintTree { parts: vec![
///      Box::new(StringOrConstraintTree::String("|".to_string())),
///      Box::new(StringOrConstraintTree::String("1.5".to_string())),
///      Box::new(StringOrConstraintTree::ConstraintTree(ConstraintTree {parts: vec![
///           Box::new(StringOrConstraintTree::String(",".to_string())),
///           Box::new(StringOrConstraintTree::ConstraintTree(ConstraintTree {parts: vec![
///               Box::new(StringOrConstraintTree::String("|".to_string())),
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
pub fn treeify(spec_str: &str) -> ConstraintTree {
    lazy_static! { static ref VSPEC_TOKENS: Regex = Regex::new(
        r"('\s*\^[^$]*[$]|\s*[()|,]|[^()|,]+"
    ).unwrap(); }
    let mut output: ConstraintTree = ConstraintTree { parts: vec![]};
    let mut stack: Vec<&str> =vec![];

    for item in VSPEC_TOKENS.find_iter(&format!("({})", spec_str)) {
        let item = item.as_str().trim();
        match item {
            "|" => { _apply_ops("(", &mut output, &mut stack); stack.push("|"); },
            "," => { _apply_ops("|(", &mut output, &mut stack); stack.push(","); },
            "(" => { stack.push("(") },
            ")" => {
                _apply_ops("(", &mut output, &mut stack);
                if stack.is_empty() || *stack.last().unwrap() != "(" {
                    panic!("expression must start with \"(\"");
                }
                stack.pop();
            },
            _ => { output.parts.push(Box::new(StringOrConstraintTree::String(item.to_string()))); }
        }
    }

    if ! stack.is_empty() { panic!(format!("unable to convert to expression tree: {:#?}", stack)); }
    output
}

pub struct VersionSpec {}

//impl Spec for VersionSpec {
//    pub fn merge(&self, other: &VersionSpec) {
//
//    }
//}

#[cfg_attr(tarpaulin, skip)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn untreeify_single() {
        let v = untreeify("1.2.3".into());
        assert_eq!(v, "1.2.3");
    }

    #[test]
    fn untreeify_simple_and() {
        let v = untreeify(vec![",", "1.2.3", ">4.5.6"].into());
        assert_eq!(v, "1.2.3,>4.5.6");
    }

    #[test]
    fn untreeify_simple_or() {
        let v = untreeify(vec!["|", "1.2.3", ">4.5.6"].into());
        assert_eq!(v, "1.2.3|>4.5.6");
    }

    #[test]
    fn untreeify_and_joining_inner_or() {
        let inner_or: ConstraintTree = vec!["|", "1.2.3", "4.5.6"].into();
        let v = untreeify(vec![
            Box::new(StringOrConstraintTree::String(",".to_string())),
            Box::new(StringOrConstraintTree::ConstraintTree(inner_or)),
            Box::new(StringOrConstraintTree::String("<=7.8.9".to_string())),
            ].into());
        assert_eq!(v, "(1.2.3|4.5.6),<=7.8.9");
    }

    #[test]
    fn untreeify_nested() {
        let or_6_7: ConstraintTree = vec!["|", "1.6", "1.7"].into();
        let or_6_7_and_8_9: ConstraintTree = vec![
            Box::new(StringOrConstraintTree::String(",".to_string())),
            Box::new(StringOrConstraintTree::ConstraintTree(or_6_7)),
            Box::new(StringOrConstraintTree::String("1.8".to_string())),
            Box::new(StringOrConstraintTree::String("1.9".to_string())),
        ].into();
        let or_with_inner_group: ConstraintTree = vec![
            Box::new(StringOrConstraintTree::String("|".to_string())),
            Box::new(StringOrConstraintTree::String("1.5".to_string())),
            Box::new(StringOrConstraintTree::ConstraintTree(or_6_7_and_8_9)),
            Box::new(StringOrConstraintTree::String("2.0".to_string())),
            Box::new(StringOrConstraintTree::String("2.1".to_string())),
        ].into();
        let v = untreeify(or_with_inner_group);
        assert_eq!(v, "1.5|((1.6|1.7),1.8,1.9)|2.0|2.1");
    }

    #[test]
    fn treeify_single() {
        let v = treeify("1.2.3");
        assert!(v == ConstraintTree { parts: vec![
            Box::new(StringOrConstraintTree::String("1.2.3".to_string()))]});
    }

//    #[test]
//    fn treeify_simple_and() {
//        let v = treeify("1.2.3,>4.5.6");
//        assert_eq!(v, (",", "1.2.3", ">4.5.6"));
//    }

//    #[test]
//    fn treeify_and_or_grouping() {
//        let v = treeify("1.2.3,4.5.6|<=7.8.9");
//        assert_eq!(v, ('|', (',', '1.2.3', '4.5.6'), '<=7.8.9'))));
//    }
//
//    #[test]
//    fn treeify_and_with_or_in_parens() {
//        let v = treeify("(1.2.3|4.5.6),<=7.8.9");
//        assert_eq!(v, (',', ('|', '1.2.3', '4.5.6'), '<=7.8.9'))));
//    }

    //  treeify("((1.5|((1.6|1.7), 1.8), 1.9 |2.0))|2.1")
    //  ('|', '1.5', (',', ('|', '1.6', '1.7'), '1.8', '1.9'), '2.0', '2.1')
    //  treeify("1.5|(1.6|1.7),1.8,1.9|2.0|2.1")
    //  ('|', '1.5', (',', ('|', '1.6', '1.7'), '1.8', '1.9'), '2.0', '2.1')
}