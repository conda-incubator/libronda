use std::cmp::Ordering;
use std::fmt;
use regex::Regex;
use unicase::UniCase;

#[derive(Debug, Copy, Clone)]
pub struct PEP440String<'a> {
    alpha: &'a str,
}

impl<'a> PEP440String<'a> {
    pub fn from(alpha: &'a str) -> PEP440String {
        PEP440String{ alpha }
    }

    pub fn empty() -> PEP440String<'a> {
        PEP440String {alpha: ""}
    }
}

fn compare_pep440_str<'a>(left: &'a str, right: &'a str) -> Option<Ordering> {
    lazy_static! { static ref DEV_RE: Regex = Regex::new("(?i)dev").unwrap(); }
    lazy_static! { static ref POST_RE: Regex = Regex::new("(?i)post").unwrap(); }

    // top on the list is post.  It always wins.  Process it first.
    match (POST_RE.is_match(left), POST_RE.is_match(right)) {
        (true, true) => Some(Ordering::Equal),
        (false, true) => Some(Ordering::Less),
        (true, false) => Some(Ordering::Greater),
        // Empty strings are when no string value is present for one or the other (release versions)
        _ => match (left.is_empty(), right.is_empty()) {
            (true, true) => Some(Ordering::Equal),
            (false, true) => Some(Ordering::Less),
            (true, false) => Some(Ordering::Greater),
            // dev is inverse of post - it always loses
            _ => match (DEV_RE.is_match(left), DEV_RE.is_match(right)) {
                (true, true) => Some(Ordering::Equal),
                (false, true) => Some(Ordering::Greater),
                (true, false) => Some(Ordering::Less),
                // this is the final fallback to lexicographic sorting, if neither
                //   dev nor post are in effect.  Case insensitive comparison here.
                (false, false) => UniCase::new(left).partial_cmp(&UniCase::new(right)),
            }
        }
    }
}

impl<'a> PartialOrd for PEP440String<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        compare_pep440_str(self.alpha, other.alpha)
    }
}

impl<'a> PartialEq for PEP440String<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(&other).unwrap() == Ordering::Equal
    }
}

impl<'a> fmt::Display for PEP440String<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.alpha)
    }
}

#[cfg_attr(tarpaulin, skip)]
#[cfg(test)]
mod tests {
    use super::PEP440String;

    #[test]
    fn compare_dev_less_alpha() {
        assert_eq!(PEP440String::from("dev") < PEP440String::from("alpha"), true);
    }

    #[test]
    fn compare_lexicographic_default() {
        assert_eq!(PEP440String::from("a") < PEP440String::from("d"), true);
    }

    #[test]
    fn compare_post_greater_later() {
        assert_eq!(PEP440String::from("z") < PEP440String::from("post"), true);
    }

    #[test]
    fn compare_post_greater_empty() {
        assert_eq!(PEP440String::from("") < PEP440String::from("post"), true);
    }

    #[test]
    fn compare_empty_greater_alpha() {
        assert_eq!(PEP440String::from("a") < PEP440String::from(""), true);
    }
}