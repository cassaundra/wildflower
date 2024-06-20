#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(std))]
extern crate alloc;
extern crate core;

use core::ops::Deref;

#[cfg(not(std))]
use alloc::{vec, vec::Vec};
use core::fmt;
use core::fmt::{Display, Formatter};

use stable_deref_trait::StableDeref;
use yoke::{Yoke, Yokeable};

/// The escape character, `\`.
pub const ESCAPE_CHAR: char = '\\';

/// The *single* wildcard character, `?`.
pub const WILDCARD_SINGLE_CHAR: char = '?';

/// The *many* wildcard character, `*`.
pub const WILDCARD_MANY_CHAR: char = '*';

/// A wildcard pattern to be matched against strings.
///
/// In general, instances of a pattern should be reused wherever possible to
/// avoid the cost imposed by recompiling.
///
/// See the [crate documentation](crate) for more documentation and examples.
///
/// ```
/// # use wildflower::Pattern;
/// let pattern = Pattern::new(r"*flow?r? it's *!");
/// assert!(pattern.matches("wildflower: it's fast!"));
/// ```
pub struct Pattern<S> {
    inner: Yoke<PatternInner<'static>, S>,
}

impl<S> Pattern<S>
where
    S: StableDeref,
    <S as Deref>::Target: 'static + AsRef<str>,
{
    /// Create a new pattern from a source string.
    pub fn new(source: S) -> Self {
        let inner = Yoke::attach_to_cart(source, |s| Compiler::from_source(s.as_ref()).compile());
        Pattern { inner }
    }

    /// Test whether or not this pattern matches a given string.
    pub fn matches(&self, string: &str) -> bool {
        use PatternElement::*;

        // current view of the string
        let mut slice_start = 0;

        let mut elems = self.inner.get().elements.iter();
        while let Some(elem) = elems.next() {
            let slice = if slice_start < string.len() {
                &string[slice_start..]
            } else {
                ""
            };

            match elem {
                Substring(value) => {
                    if !slice.starts_with(&**value) {
                        return false;
                    }

                    slice_start += value.len();
                }
                Wildcard(wc) => {
                    if wc.minimum > 0 {
                        // try to take the minimum
                        if let Some((idx, c)) = slice.char_indices().nth(wc.minimum - 1) {
                            slice_start += idx + c.len_utf8();
                        } else {
                            return false;
                        }
                    }

                    if wc.is_many {
                        // look ahead
                        match elems.next() {
                            // substring: consume until substring is found
                            Some(Substring(s)) => {
                                if let Some(idx) = &string[slice_start..].find(&**s) {
                                    slice_start += idx + s.len();
                                } else {
                                    return false;
                                }
                            }

                            // end of pattern: implicitly consume the rest of string
                            None => return true,

                            // per the optimization rules, no wildcards should follow
                            _ => unreachable!("invalid pattern"),
                        }
                    }
                }
            }
        }

        // we have succeeded if we have successfully matched all characters
        slice_start == string.len()
    }
}

impl<S> Display for Pattern<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        for p in self.inner.get().elements.iter() {
            match p {
                PatternElement::Substring(ss) => {
                    for c in ss.chars() {
                        match c {
                            ESCAPE_CHAR => {
                                write!(f, r"\\")?;
                            }
                            WILDCARD_SINGLE_CHAR => {
                                write!(f, r"\?")?;
                            }
                            WILDCARD_MANY_CHAR => {
                                write!(f, r"\*")?;
                            }
                            _ => {
                                write!(f, "{}", c)?;
                            }
                        }
                    }
                }
                PatternElement::Wildcard(wc) => {
                    if wc.is_many {
                        write!(f, "*")?;
                    } else {
                        for _ in 0..wc.minimum {
                            write!(f, "?")?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl<S> From<S> for Pattern<S>
where
    S: StableDeref,
    <S as Deref>::Target: 'static + AsRef<str>,
{
    fn from(source: S) -> Self {
        Pattern::new(source)
    }
}

impl<S> PartialEq for Pattern<S> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.get() == other.inner.get()
    }
}
impl<S> Eq for Pattern<S> {}

#[derive(Yokeable, PartialEq, Eq)]
struct PatternInner<'a> {
    elements: Vec<PatternElement<'a>>,
}

#[derive(PartialEq, Eq)]
enum PatternElement<'a> {
    Substring(&'a str),
    Wildcard(Wildcard),
}

#[derive(Copy, Clone, Default, Eq, PartialEq)]
struct Wildcard {
    minimum: usize,
    is_many: bool,
}

impl Wildcard {
    pub fn single() -> Self {
        Wildcard {
            minimum: 1,
            is_many: false,
        }
    }

    pub fn many() -> Self {
        Wildcard {
            minimum: 0,
            is_many: true,
        }
    }

    pub fn push_single(mut self) -> Self {
        self.minimum += 1;
        self
    }

    pub fn push_many(mut self) -> Self {
        self.is_many = true;
        self
    }
}

/// A pattern "compiler" which transform a pattern string into its efficient
/// internal representation, a [Pattern].
///
/// # Algorithm
///
/// The underlying algorithm is fairly simple:
/// 1. Construct the longest possible substrings of non-wildcard characters.
/// 2. Pack consecutive wildcards into "wildcard groups" which keep track of 1)
///    the number of single wildcards (`?`) in the substring, and 2) whether or
///    not at least one many wildcard (`*`) is present within the substring.
///
/// The constructed internal representation is a sequence of these substrings
/// and wildcard groups.
struct Compiler<'a> {
    source: &'a str,
    elements: Vec<PatternElement<'a>>,
    slice_start: usize,
    slice_end: usize,
    escape: bool,
}

impl<'a> Compiler<'a> {
    pub fn from_source(source: &'a str) -> Self {
        Compiler {
            source,
            elements: vec![],
            slice_start: 0,
            slice_end: 0,
            escape: false,
        }
    }

    /// Parse and optimize a pattern from a source string in a single step,
    /// returning a list of the optimized pattern's constituent elements.
    ///
    /// This function is infallible.
    pub fn compile(mut self) -> PatternInner<'a> {
        for c in self.source.chars() {
            match c {
                ESCAPE_CHAR if !self.escape => {
                    self.flush();
                    self.reset_after(ESCAPE_CHAR);
                    self.escape = true;
                }
                WILDCARD_SINGLE_CHAR if !self.escape => {
                    self.flush();
                    self.reset_after(WILDCARD_SINGLE_CHAR);
                    self.push_wildcard(false);
                }
                WILDCARD_MANY_CHAR if !self.escape => {
                    self.flush();
                    self.reset_after(WILDCARD_MANY_CHAR);
                    self.push_wildcard(true);
                }
                _ => {
                    self.slice_end += c.len_utf8();
                    self.escape = false;
                }
            }
        }

        self.flush();

        PatternInner {
            elements: self.elements,
        }
    }

    /// Push the current substring and advance.
    fn flush(&mut self) {
        if self.slice_start < self.slice_end {
            self.elements.push(PatternElement::Substring(
                &self.source[self.slice_start..self.slice_end],
            ));
        }

        self.slice_start = self.slice_end;
    }

    /// Reset the substring pointer to just after the current character.
    fn reset_after(&mut self, c: char) {
        self.slice_start = self.slice_end + c.len_utf8();
        self.slice_end = self.slice_start;
    }

    /// Add a wildcard to the end of the pattern, merging it with the previous
    /// if one exists.
    fn push_wildcard(&mut self, is_many: bool) {
        if let Some(PatternElement::Wildcard(wildcard)) = self.elements.last_mut() {
            *wildcard = if is_many {
                wildcard.push_many()
            } else {
                wildcard.push_single()
            };
        } else {
            let wildcard = if is_many {
                Wildcard::many()
            } else {
                Wildcard::single()
            };
            self.elements.push(PatternElement::Wildcard(wildcard));
        }
    }
}

#[cfg(feature = "serde")]
use serde::de::{Deserialize, Deserializer, Error as DeError, Visitor};
#[cfg(feature = "serde")]
use serde::ser::{Serialize, Serializer};

#[cfg(feature = "serde")]
impl<D> Serialize for Pattern<D> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Pattern<String> {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PatternVisitor;

        impl<'de> Visitor<'de> for PatternVisitor {
            type Value = Pattern<String>;

            #[inline]
            fn expecting(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
                f.write_str("a CIDR string")
            }

            #[inline]
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(Pattern::from(v.to_string()))
            }
        }

        deserializer.deserialize_str(PatternVisitor)
    }
}
