#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(std))]
extern crate alloc;

#[cfg(not(std))]
use alloc::vec;
#[cfg(not(std))]
use alloc::vec::Vec;

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

        let mut slice = string;

        let mut elements = self.elements.iter();

        while let Some(elem) = elements.next() {
            match elem {
                Substring(value) => {
                    if !slice.starts_with(value) {
                        return false;
                    }

                    slice = &slice[value.len()..];
                }
                SingleWildcard(count) => {
                    // take the last char and calculate the next index
                    if let Some((idx, value)) = slice.char_indices().nth(*count - 1) {
                        slice = &slice[idx + value.len_utf8()..];
                    } else {
                        return false;
                    }
                }
                ManyWildcard => {
                    // figure out what needs to be done after this wildcard
                    match elements.next() {
                        // value: we consume zero or more characters until we find the next
                        // substring
                        Some(Substring(value)) => {
                            if let Some(index) = string.find(value) {
                                if index + value.len() >= slice.len() {
                                    slice = "";
                                } else {
                                    slice = &slice[index + value.len()..];
                                }
                            } else {
                                return false;
                            }
                        }

                        // end of pattern: we can implicitly consume the rest of string, and
                        // therefore have finished matching
                        None => return true,

                        // per the optimization rules, we should never find another wildcard after a
                        // many wildcard
                        _ => unreachable!("invalid pattern"),
                    }
                }
            }
        }

        // we have succeeded if we have successfully matched all characters
        slice.is_empty()
    }
}

/// The escape character, `\`.
pub const ESCAPE_CHAR: char = '\\';

/// The *single* wildcard character, `?`.
pub const WILDCARD_SINGLE_CHAR: char = '?';

/// The *many* wildcard character, `*`.
pub const WILDCARD_MANY_CHAR: char = '*';

// TODO better encode optimization rules into types?

#[derive(Debug)]
enum PatternElement<'a> {
    Substring(&'a str),
    SingleWildcard(usize),
    ManyWildcard,
}

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
    ///
    /// ## Optimization
    ///
    /// The optimization function is not bijective---each possible source string
    /// has a corresponding output which this functions deems its optimized
    /// form, but multiple source strings may share this same optimized
    /// form.
    ///
    /// This following optimizations are made (in this order):
    ///
    /// 1. Substrings are maximally large.
    ///
    /// 2. Adjacent many wildcards are merged together.
    ///
    /// 3. Single wildcards are rearranged to precede many wildcards.
    ///
    /// 4. Adjacent single wildcards are merged together.
    pub fn compile(mut self) -> Pattern<'a> {
        use PatternElement::*;

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

                    // optimizations:
                    // 1. flatten repeated single wildcards
                    // 2. ensure that many wildcards are after single wildcards
                    if let Some(SingleWildcard(count)) = self.elements.last_mut() {
                        *count += 1;
                    } else if let Some(ManyWildcard) = self.elements.last() {
                        // try and update the count of a single wildcard before the many wildcard
                        if self.elements.len() > 1 {
                            let len = self.elements.len();
                            if let SingleWildcard(count) = &mut self.elements[len - 2] {
                                *count += 1;
                                continue;
                            }
                        }

                        // otherwise, just insert a new one
                        self.elements
                            .insert(self.elements.len() - 1, SingleWildcard(1));
                    } else {
                        self.elements.push(SingleWildcard(1));
                    }
                }
                WILDCARD_MANY_CHAR if !self.escape => {
                    self.flush();
                    self.reset_after(WILDCARD_MANY_CHAR);

                    // optimization: flatten repeated many wildcards
                    if !matches!(self.elements.last(), Some(ManyWildcard)) {
                        self.elements.push(ManyWildcard);
                    }
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

    fn flush(&mut self) {
        use PatternElement::Substring;

        // flush only if slice is non-empty
        if self.slice_start != self.slice_end {
            let slice = &self.source[self.slice_start..self.slice_end];
            self.elements.push(Substring(slice));
        }

        self.slice_start = self.slice_end;
    }

    fn reset_after(&mut self, c: char) {
        self.slice_start = self.slice_end + c.len_utf8();
        self.slice_end = self.slice_start;
    }
}
