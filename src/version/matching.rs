
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

fn matcher_for_tuple(vspec) {
//    vspec_tree = vspec
//    _matcher = self.any_match if vspec_tree[0] == '|' else self.all_match
//    tup = tuple(VersionSpec(s) for s in vspec_tree[1:])
//    vspec_str = untreeify((vspec_tree[0],) + tuple(t.spec for t in tup))
//    self.tup = tup
//    matcher = _matcher
//    is_exact = False
//    return vspec_str, matcher, is_exact
}

impl Spec {
    fn is_exact(&self) -> bool {

    }
    fn regex_match(&self, spec_str: &str) -> bool {

    }
    fn operator_match(&self, spec_str: &str) -> bool {

    }
    fn any_match(&self, spec_str: &str) -> bool {

    }
    fn all_match(&self, spec_str: &str) -> bool {

    }
    fn exact_match(&self, spec_str: &str) -> bool {

    }
    fn always_true_match(&self, _spec_str: &str) -> bool {
        true
    }

}

pub enum VspecInputTypes {
    String,
    Tuple,
}

fn _apply_ops(cstop: &str, output: &mut Vec<&str>, stack: &mut Vec<&str>) {
    // cstop: operators with lower precedence
    while stack.len() > 0 && ! cstop.contains(stack[-1]) {
//        if output.len() < 2:
//            raise InvalidVersionSpec(spec_str, "cannot join single expression")
        let c = stack.pop().unwrap();
        let r = output.pop().unwrap();
        // Fuse expressions with the same operator; e.g.,
        //   ('|', ('|', a, b), ('|', c, d))becomes
        //   ('|', a, b, c d)
        // We're playing a bit of a trick here. Instead of checking
        // if the left or right entries are tuples, we're counting
        // on the fact that if we _do_ see a string instead, its
        // first character cannot possibly be equal to the operator.
        let r = match (r[0] == c) {
            true => r[1:],
        _ => (r, ),
    }
    let mut left = output.pop().unwrap().to_string();
    left = left.[1:] if left[0] == c else (left,);
    output.push((c,)+left+r)
}
}

pub fn treeify(spec_str: &str) {
    tokens = re.findall(VSPEC_TOKENS, '(%s)' % spec_str)
    output = []
    stack = []
    for item in tokens:
        item = item.strip()
        if item == '|':
            apply_ops('(')
            stack.append('|')
        elif item == ',':
            apply_ops('|(')
            stack.append(',')
        elif item == '(':
            stack.append('(')
        elif item == ')':
            apply_ops('(')
            if not stack or stack[-1] != '(':
                raise InvalidVersionSpec(spec_str, "expression must start with '('")
            stack.pop()
        else:
            output.append(item)
    if stack:
        raise InvalidVersionSpec(spec_str, "unable to convert to expression tree: %s" % stack)
    return output[0]
}

fn untreeify(spec: &Vec<&str>, _inand: bool, depth: usize) {
    if isinstance(spec, tuple):
        if spec[0] == '|':
            res = '|'.join(map(lambda x: untreeify(x, depth=depth + 1), spec[1:]))
            if _inand or depth > 0:
                res = '(%s)' % res
        else:
            res = ','.join(map(lambda x: untreeify(x, _inand=True, depth=depth + 1), spec[1:]))
            if depth > 0:
                res = '(%s)' % res
        return res
    return spec
}

pub struct VersionSpec {}

impl Spec for VersionSpec {
    pub fn merge(&self, other: &VersionSpec) {

    }
}