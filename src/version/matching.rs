use std::ops::Deref;

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

//fn _apply_ops(cstop: &str, output: &mut Vec<&str>, stack: &mut Vec<&str>) {
//    // cstop: operators with lower precedence
//    while stack.len() > 0 && ! cstop.contains(stack[-1]) {
////        if output.len() < 2:
////            raise InvalidVersionSpec(spec_str, "cannot join single expression")
//        let c = stack.pop().unwrap();
//        let r = output.pop().unwrap();
//        // Fuse expressions with the same operator; e.g.,
//        //   ('|', ('|', a, b), ('|', c, d))becomes
//        //   ('|', a, b, c d)
//        // We're playing a bit of a trick here. Instead of checking
//        // if the left or right entries are tuples, we're counting
//        // on the fact that if we _do_ see a string instead, its
//        // first character cannot possibly be equal to the operator.
//        let r = match (r[0] == c) {
//            true => r[1:],
//        _ => (r, ),
//    }
//    let mut left = output.pop().unwrap().to_string();
//    left = left.[1:] if left[0] == c else (left,);
//    output.push((c,)+left+r)
//}
//}

//pub fn treeify(spec_str: &str) {
//    tokens = re.findall(VSPEC_TOKENS, "(%s)" % spec_str)
//    output = []
//    stack = []
//    for item in tokens:
//        item = item.strip()
//        if item == '|':
//            apply_ops('(')
//            stack.append('|')
//        elif item == ',':
//            apply_ops('|(')
//            stack.append(',')
//        elif item == '(':
//            stack.append('(')
//        elif item == ')':
//            apply_ops('(')
//            if not stack or stack[-1] != '(':
//                raise InvalidVersionSpec(spec_str, "expression must start with '('")
//            stack.pop()
//        else:
//            output.append(item)
//    if stack:
//        raise InvalidVersionSpec(spec_str, "unable to convert to expression tree: %s" % stack)
//    return output[0]
//}

pub enum StringOrConstraintJoint {
    String(String),
    ConstraintJoint(ConstraintJoint), // vec is a mix of &str or other vector(s) of str, possibly nested
}

pub struct ConstraintJoint {
    pub parts: Vec<Box<StringOrConstraintJoint>>,
}

impl ConstraintJoint {
    fn isand(&self) -> bool {
        self.parts.len() > 0 && match self.parts[0].deref() {
            StringOrConstraintJoint::String(s) => *s == ",",
            _ => false
        }
    }

    fn combine(&self, inand:bool, nested: bool) -> String {
        let mut res: String;
        match self.parts.len() {
            1 => {
                res = if let StringOrConstraintJoint::String(s) = self.parts[0].deref() {
                    s.to_string()
                } else {panic!()};
                },
            0 => panic!(),
            _ => {
                let mut str_parts = vec![];

                for item in &self.parts[1..] {
                    str_parts.push(match item.deref() {
                        StringOrConstraintJoint::String(s) => s.deref().to_string(),
                        StringOrConstraintJoint::ConstraintJoint(cj) => cj.combine(self.isand(), true)
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

impl From<&str> for StringOrConstraintJoint {
    fn from (s: &str) -> StringOrConstraintJoint {
        StringOrConstraintJoint::String(s.to_string())
    }
}

impl From<&str> for ConstraintJoint {
    fn from (s: &str) -> ConstraintJoint {
        ConstraintJoint { parts: vec![Box::new(StringOrConstraintJoint::String(s.to_string()))]}
    }
}

impl From<StringOrConstraintJoint> for ConstraintJoint {
    fn from (scj: StringOrConstraintJoint) -> ConstraintJoint {
        match scj {
            StringOrConstraintJoint::ConstraintJoint(_scj) => _scj,
            _ => ConstraintJoint { parts: vec![Box::new(scj)] },
        }
    }
}

impl From<Vec<&str>> for ConstraintJoint
{
    fn from(s: Vec<&str>) -> ConstraintJoint {
        ConstraintJoint{
            parts: s.iter().map(|x| Box::new(StringOrConstraintJoint::String(x.to_string()))).collect()
        }
    }
}

impl From<Vec<Box<StringOrConstraintJoint>>> for ConstraintJoint
{
    fn from (parts: Vec<Box<StringOrConstraintJoint>>) -> ConstraintJoint {
        ConstraintJoint{ parts }
    }
}

/// Given a single spec or collection of specs, join them together into a string that captures
///   then relationships among specs
///
/// # Examples
///
/// ```
/// use ronda::{untreeify, ConstraintJoint, StringOrConstraintJoint};
///
/// let cj123_456: ConstraintJoint = vec![",", "1.2.3", "4.5.6"].into();
/// let v = untreeify("1.2.3".into());
/// assert_eq!(v, "1.2.3");
/// let v = untreeify(vec![",", "1.2.3", ">4.5.6"].into());
/// assert_eq!(v, "1.2.3,>4.5.6");
/// let tree: ConstraintJoint = ConstraintJoint {parts: vec![Box::new(StringOrConstraintJoint::String("|".to_string())),
///                                                          Box::new(StringOrConstraintJoint::ConstraintJoint(cj123_456)),
///                                                          Box::new(StringOrConstraintJoint::String("<=7.8.9".to_string()))]};
/// let v = untreeify(tree);
/// assert_eq!(v, "(1.2.3,4.5.6)|<=7.8.9");
/// ```
pub fn untreeify(spec: ConstraintJoint) -> String {
    spec.combine(false, false)
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
    use super::{untreeify, ConstraintJoint, StringOrConstraintJoint};

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
        let inner_or: ConstraintJoint = vec!["|", "1.2.3", "4.5.6"].into();
        let v = untreeify(vec![
            Box::new(StringOrConstraintJoint::String(",".to_string())),
            Box::new(StringOrConstraintJoint::ConstraintJoint(inner_or)),
            Box::new(StringOrConstraintJoint::String("<=7.8.9".to_string())),
            ].into());
        assert_eq!(v, "(1.2.3|4.5.6),<=7.8.9");
    }

    #[test]
    fn untreeify_nested() {
        let or_6_7: ConstraintJoint = vec!["|", "1.6", "1.7"].into();
        let or_6_7_and_8_9: ConstraintJoint = vec![
            Box::new(StringOrConstraintJoint::String(",".to_string())),
            Box::new(StringOrConstraintJoint::ConstraintJoint(or_6_7)),
            Box::new(StringOrConstraintJoint::String("1.8".to_string())),
            Box::new(StringOrConstraintJoint::String("1.9".to_string())),
        ].into();
        let or_with_inner_group: ConstraintJoint = vec![
            Box::new(StringOrConstraintJoint::String("|".to_string())),
            Box::new(StringOrConstraintJoint::String("1.5".to_string())),
            Box::new(StringOrConstraintJoint::ConstraintJoint(or_6_7_and_8_9)),
            Box::new(StringOrConstraintJoint::String("2.0".to_string())),
            Box::new(StringOrConstraintJoint::String("2.1".to_string())),
        ].into();
        let v = untreeify(or_with_inner_group);
        assert_eq!(v, "1.5|((1.6|1.7),1.8,1.9)|2.0|2.1");
    }
}