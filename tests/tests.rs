use wildflower::Pattern;

// TODO unicode tests

#[test]
fn test_basic() {
    assert!(pattern("").matches(""));
    assert!(pattern("a").matches("a"));
    assert!(pattern("abc").matches("abc"));

    assert!(!pattern("abc").matches("xyz"));
    assert!(!pattern("abc").matches("a"));
    assert!(!pattern("abc").matches("ab"));
    assert!(!pattern("abc").matches("abx"));
}

#[test]
fn test_single_wildcard() {
    assert!(pattern("?").matches("a"));
    assert!(pattern("??").matches("aa"));
    assert!(pattern("a?").matches("aa"));
    assert!(pattern("abc???").matches("abcxyz"));
    assert!(pattern("a?aa").matches("abaa"));
    assert!(pattern("?a?a?").matches("baaax"));

    assert!(!pattern("?").matches(""));
    assert!(!pattern("?").matches("ab"));
    assert!(!pattern("??").matches("a"));
    assert!(!pattern("a?").matches("ba"));
    assert!(!pattern("a?").matches("a"));
    assert!(!pattern("abc???").matches("abcxy"));
    assert!(!pattern("a?aa").matches("aaa"));
    assert!(!pattern("?a?a?").matches("abcde"));
}

#[test]
fn test_many_wildcard_single() {
    assert!(pattern("*").matches(""));
    assert!(pattern("*").matches("a"));
    assert!(pattern("*").matches("abcdef"));
}

#[test]
fn test_many_wildcard_many() {
    assert!(pattern("***").matches(""));
    assert!(pattern("***").matches("a"));
    assert!(pattern("***").matches("abcdef"));
}

#[test]
fn test_many_wildcard_leading() {
    assert!(pattern("*a").matches("a"));
    assert!(pattern("*fast").matches("breakfast"));

    assert!(!pattern("*a").matches("ab"));
    assert!(!pattern("*fast").matches("break"));
}

#[test]
fn test_many_wildcard_trailing() {
    assert!(pattern("a*").matches("a"));
    assert!(pattern("break*").matches("breakfast"));

    assert!(!pattern("a*").matches("ba"));
    assert!(!pattern("break*").matches("fast"));
}

#[test]
fn test_many_wildcard_inner() {
    assert!(pattern("a*b").matches("ab"));
    assert!(pattern("a*b").matches("aXb"));
    assert!(pattern("a*b").matches("aXYZb"));

    assert!(!pattern("a*b").matches("aX"));
    assert!(!pattern("a*b").matches("Xb"));
}

#[test]
fn test_many_wildcard_mixed() {
    assert!(pattern("*a*b").matches("ab"));
    assert!(pattern("*a*b").matches("Xab"));
    assert!(pattern("*a*b").matches("aXb"));
    assert!(pattern("*a*b").matches("XaYb"));

    assert!(!pattern("*a*b").matches("a"));
    assert!(!pattern("*a*b").matches("b"));

    assert!(pattern("a*b*").matches("ab"));
    assert!(pattern("a*b*").matches("abX"));
    assert!(pattern("a*b*").matches("aXb"));
    assert!(pattern("a*b*").matches("aXbY"));

    assert!(!pattern("a*b*").matches("a"));
    assert!(!pattern("a*b*").matches("b"));

    assert!(pattern("*a*b*").matches("ab"));
    assert!(pattern("*a*b*").matches("Xab"));
    assert!(pattern("*a*b*").matches("aXb"));
    assert!(pattern("*a*b*").matches("XaYb"));
    assert!(pattern("*a*b*").matches("abX"));
    assert!(pattern("*a*b*").matches("XabY"));
    assert!(pattern("*a*b*").matches("XaYbZ"));

    assert!(!pattern("*a*b*").matches("a"));
    assert!(!pattern("*a*b*").matches("b"));

    assert!(pattern("a*X*b").matches("aXb"));
    assert!(pattern("a*X*b").matches("aYXb"));
    assert!(pattern("a*X*b").matches("aXYb"));
    assert!(pattern("a*X*b").matches("aYXYb"));

    assert!(!pattern("a*X*b").matches("ab"));
    assert!(!pattern("a*X*b").matches("aX"));
    assert!(!pattern("a*X*b").matches("Yb"));
    assert!(!pattern("a*X*b").matches("aYb"));
    assert!(!pattern("a*X*b").matches("aZYZb"));
}

#[test]
fn test_mixed_wildcards() {
    assert!(pattern("?*").matches("h"));
    assert!(pattern("?*").matches("hi!"));
    assert!(pattern("h?ll*!").matches("hello world!"));
    assert!(pattern("h?ll*!").matches("hollow!"));
    assert!(pattern("h?ll*!").matches("hell!"));
    assert!(pattern("??*").matches("ab"));
    assert!(pattern("??*").matches("abc"));
    assert!(pattern("??*").matches("abcd"));

    assert!(!pattern("?*").matches(""));
    assert!(!pattern("h?ll*!").matches("hllo world!"));
    assert!(!pattern("h?ll*!").matches("hell"));
    assert!(!pattern("??*").matches("a"));
}

#[test]
fn test_escapes() {
    assert!(pattern(r"\\").matches(r"\"));
    assert!(pattern(r"\\\\").matches(r"\\"));
    assert!(pattern(r"\?").matches(r"?"));
    assert!(pattern(r"\*").matches(r"*"));
    assert!(pattern(r"a\bc").matches(r"abc"));
    assert!(pattern(r"\?\*\a").matches("?*a"));
    assert!(pattern(r"h?\?").matches("hi?"));
    assert!(pattern(r"\??????").matches("? okay"));
    assert!(pattern(r"\**").matches("*.*"))
}

fn pattern(pattern: &str) -> Pattern<'_> {
    Pattern::new(pattern)
}
