#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(std))]
extern crate alloc;

#[cfg(not(std))]
use alloc::{vec, vec::Vec};

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
/// let pattern = Pattern::new("*flow?r? it's *!");
/// assert!(pattern.matches("wildflower: it's fast!"));
/// ```
pub struct Pattern<'a> {
    elements: Vec<PatternElement<'a>>,
}

impl<'a> Pattern<'a> {
    /// Create a new pattern from a source string.
    pub fn new(source: &'a str) -> Pattern<'a> {
        Compiler::from_source(source).compile()
    }

    /// Test whether or not this pattern matches a given string.
    pub fn matches(&self, string: &str) -> bool {
        use PatternElement::*;

        // current view of the string
        let mut slice_start = 0;

        let mut elems = self.elements.iter();
        while let Some(elem) = elems.next() {
            let slice = if slice_start < string.len() {
                &string[slice_start..]
            } else {
                ""
            };

            match elem {
                Substring(value) => {
                    if !slice.starts_with(value) {
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
                                if let Some(idx) = &string[slice_start..].find(s) {
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

enum PatternElement<'a> {
    Substring(&'a str),
    Wildcard(Wildcard),
}

#[derive(Copy, Clone, Default)]
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
    pub fn from_source(source: &'a str) -> Compiler<'a> {
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
    pub fn compile(mut self) -> Pattern<'a> {
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

        Pattern {
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
