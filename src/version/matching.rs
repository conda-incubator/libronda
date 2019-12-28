use super::spec_trees::*;

pub trait Spec {
    // default methods
//    fn is_exact(&self) -> bool {}
//    fn regex_match(&self, spec_str: &str) -> bool {}
//    fn operator_match(&self, spec_str: &str) -> bool {}
//    fn any_match(&self, spec_str: &str) -> bool {}
//    fn all_match(&self, spec_str: &str) -> bool {}
    fn always_true_match(&self, _spec_str: &str) -> bool {true}

    // To be implemented by other things
    fn merge(&self, other: &impl Spec) -> Self;
    fn exact_match(&self, spec_str: &str) -> bool;

    // properties in Python (to be implemented by other things)
    fn spec(&self) -> &str;
    fn raw_value(&self) -> &str { self.spec() }
    fn exact_value(&self) -> Option<&str> {
        if self.is_exact() { Some(self.spec()) } else { None } }
}

#[derive(Clone)]
struct VersionSpec<'a> {
    spec_str: &'a str,
    tree: &'a ConstraintTree,
}

impl Spec for VersionSpec {
    fn spec(&self) -> &str { self.spec_str }
    fn merge(&self, other: &impl Spec) -> Self { self.clone() }
    fn exact_match(&self, spec_str: &str) -> bool { false }
}

impl VersionSpec {
//    fn get_matcher(&self, other: &str) -> (String, impl Fn(&Self, &Self) -> bool, bool) {
//    }
    fn get_matcher_tuple(&self, vspec: &ConstraintTree) -> (String, impl Fn(&Self, &Self) -> bool, bool) {
        let _matcher = match vspec.combinator {
            Combinator::Or => |x| self.any_match(x),
            _ => |x| self.all_match(x)
        };
        self.tree = vspec;
        let vspec_str = untreeify(vspec);
        (vspec_str, _matcher, is_exact)
    }
}



fn matcher_for_tuple(vspec: &ConstraintTree) -> (String, impl Fn(&Self, &Self) -> bool, bool) {

    _matcher = self.any_match if vspec.combinator else self.all_match
    tup = tuple(VersionSpec(s) for s in vspec_tree[1:])
    vspec_str = untreeify((vspec_tree[0],) + tuple(t.spec for t in tup))
    self.tup = tup
    matcher = _matcher
    is_exact = False
    return vspec_str, matcher, is_exact
}

