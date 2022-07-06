//! Utilities for matching urls.

use url::Url;

#[derive(Debug, Default)]
/// Match against an URL.
#[must_use]
pub struct Match {
    segments: Vec<Segment>,
    // todo: other things, e.g. query params
}

impl Match {
    /// Construct a new [Match]
    pub fn new() -> Match {
        Default::default()
    }

    /// Add a new segment to the matcher.
    pub fn at(mut self, seg: Segment) -> Match {
        self.segments.push(seg);
        self
    }

    /// Test if this matcher matches the url.
    pub fn does_match(&self, url: &Url) -> bool {
        let mut expected_iter = self.segments.iter();
        let mut actual_iter = if let Some(segments) = url.path_segments() {
            segments
        } else {
            return false;
        };

        loop {
            let expected = if let Some(expected) = expected_iter.next() {
                expected
            } else {
                return actual_iter.next().is_none();
            };

            let actual = if let Some(actual) = actual_iter.next() {
                actual
            } else {
                return false;
            };

            match expected {
                Segment::Static(expected_path) => {
                    if *expected_path != actual {
                        return false;
                    }
                }
                Segment::Param => {}
                Segment::IntParam => {
                    if actual.parse::<i32>().is_err() {
                        return false;
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
/// A unit of a path.
pub enum Segment {
    /// Matches this string exactly
    Static(&'static str),
    /// Anything
    Param,
    /// Any integer
    IntParam,
}

/// Syntactic sugar to construct a [Segment].
pub fn path(path: &'static str) -> Segment {
    Segment::Static(path)
}

/// Syntactic sugar to construct a [Segment].
pub fn anything() -> Segment {
    Segment::Param
}

/// Syntactic sugar to construct a [Segment].
pub fn any_integer() -> Segment {
    Segment::IntParam
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;

    #[lunatic::test]
    fn test_simple() {
        let matcher = Match::new()
            .at(path("home"))
            .at(path("name"))
            .at(anything())
            .at(path("page"))
            .at(any_integer());

        assert!(matcher
            .does_match(&Url::from_str("https://example.com/home/name/someone/page/12").unwrap()));
        assert!(!matcher.does_match(
            &Url::from_str("https://example.com/home/name/someone/page/twelve").unwrap()
        ));
    }
}
