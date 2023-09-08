use wildflower::Pattern;

#[test]
fn test_basic() {
    assert!(pattern("").matches(""));
    assert!(pattern("a").matches("a"));
    assert!(pattern("🐈").matches("🐈"));
    assert!(pattern("abc").matches("abc"));

    assert_eq!(pattern("").to_string(), "".to_string());
    assert_eq!(pattern("a").to_string(), "a".to_string());
    assert_eq!(pattern("🐈").to_string(), "🐈".to_string());
    assert_eq!(pattern("abc").to_string(), "abc".to_string());

    assert!(!pattern("a").matches(""));
    assert!(!pattern("").matches("a"));
    assert!(!pattern("abc").matches("xyz"));
    assert!(!pattern("abc").matches("a"));
    assert!(!pattern("abc").matches("ab"));
    assert!(!pattern("abc").matches("abx"));
}

#[test]
fn test_single_wildcard() {
    assert!(pattern("?").matches("a"));
    assert!(pattern("??").matches("aa"));
    assert!(pattern("??").matches("αß"));
    assert!(pattern("??").matches("你好"));
    assert!(pattern("a?").matches("aa"));
    assert!(pattern("abc???").matches("abcx🐜z"));
    assert!(pattern("a?aa").matches("abaa"));
    assert!(pattern("?a?a?").matches("baaax"));

    assert_eq!(pattern("?").to_string(), "?".to_string());
    assert_eq!(pattern("??").to_string(), "??".to_string());
    assert_eq!(pattern("a?").to_string(), "a?".to_string());
    assert_eq!(pattern("abc???").to_string(), "abc???".to_string());
    assert_eq!(pattern("a?aa").to_string(), "a?aa".to_string());
    assert_eq!(pattern("?a?a?").to_string(), "?a?a?".to_string());

    assert!(!pattern("?").matches(""));
    assert!(!pattern("?").matches("ab"));
    assert!(!pattern("??").matches("a"));
    assert!(!pattern("??").matches("ß"));
    assert!(!pattern("a?").matches("ba"));
    assert!(!pattern("a?").matches("a"));
    assert!(!pattern("abc???").matches("abcxy"));
    assert!(!pattern("a?aa").matches("aaa"));
    assert!(!pattern("?a?a?").matches("abcde"));
}

#[test]
fn test_many_wildcard_single() {
    assert!(pattern("*").matches(""));
    assert!(pattern("*").matches("♡"));
    assert!(pattern("*").matches("a"));
    assert!(pattern("*").matches("abcdef"));

    assert_eq!(pattern("*").to_string(), "*".to_string());
}

#[test]
fn test_many_wildcard_many() {
    assert!(pattern("***").matches(""));
    assert!(pattern("***").matches("a"));
    assert!(pattern("***").matches("abcdef"));

    assert_eq!(pattern("***").to_string(), "*".to_string());
}

#[test]
fn test_many_wildcard_leading() {
    assert!(pattern("*a").matches("a"));
    assert!(pattern("*fast").matches("breakfast"));

    assert_eq!(pattern("*a").to_string(), "*a".to_string());
    assert_eq!(pattern("*fast").to_string(), "*fast".to_string());

    assert!(!pattern("*a").matches("ab"));
    assert!(!pattern("*fast").matches("break"));
}

#[test]
fn test_many_wildcard_trailing() {
    assert!(pattern("a*").matches("a"));
    assert!(pattern("break*").matches("breakfast"));

    assert_eq!(pattern("a*").to_string(), "a*".to_string());
    assert_eq!(pattern("break*").to_string(), "break*".to_string());

    assert!(!pattern("a*").matches("ba"));
    assert!(!pattern("break*").matches("fast"));
}

#[test]
fn test_many_wildcard_inner() {
    assert!(pattern("a*b").matches("ab"));
    assert!(pattern("a*b").matches("aXb"));
    assert!(pattern("a*b").matches("aXYZb"));

    assert_eq!(pattern("a*b").to_string(), "a*b".to_string());

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

    assert_eq!(pattern("*a*b").to_string(), "*a*b".to_string());

    assert!(pattern("a*b*").matches("ab"));
    assert!(pattern("a*b*").matches("abX"));
    assert!(pattern("a*b*").matches("aXb"));
    assert!(pattern("a*b*").matches("aXbY"));

    assert!(!pattern("a*b*").matches("a"));
    assert!(!pattern("a*b*").matches("b"));

    assert_eq!(pattern("a*b*").to_string(), "a*b*".to_string());

    assert!(pattern("*a*b*").matches("ab"));
    assert!(pattern("*a*b*").matches("Xab"));
    assert!(pattern("*a*b*").matches("aXb"));
    assert!(pattern("*a*b*").matches("XaYb"));
    assert!(pattern("*a*b*").matches("abX"));
    assert!(pattern("*a*b*").matches("XabY"));
    assert!(pattern("*a*b*").matches("XaYbZ"));

    assert!(!pattern("*a*b*").matches("a"));
    assert!(!pattern("*a*b*").matches("b"));

    assert_eq!(pattern("*a*b*").to_string(), "*a*b*".to_string());

    assert!(pattern("a*X*b").matches("aXb"));
    assert!(pattern("a*X*b").matches("aYXb"));
    assert!(pattern("a*X*b").matches("aXYb"));
    assert!(pattern("a*X*b").matches("aYXYb"));

    assert!(!pattern("a*X*b").matches("ab"));
    assert!(!pattern("a*X*b").matches("aX"));
    assert!(!pattern("a*X*b").matches("Yb"));
    assert!(!pattern("a*X*b").matches("aYb"));
    assert!(!pattern("a*X*b").matches("aZYZb"));

    assert_eq!(pattern("a*X*b").to_string(), "a*X*b".to_string());
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

    assert_eq!(pattern("?*").to_string(), "*".to_string());
    assert_eq!(pattern("h?ll*!").to_string(), "h?ll*!".to_string());
    assert_eq!(pattern("??*").to_string(), "*".to_string());

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
    assert!(pattern(r"\**").matches("*.*"));

    assert_eq!(pattern(r"\\").to_string(), r"\\".to_string());
    assert_eq!(pattern(r"\\\\").to_string(), r"\\\\".to_string());
    assert_eq!(pattern(r"\?").to_string(), r"\?".to_string());
    assert_eq!(pattern(r"\*").to_string(), r"\*".to_string());
    assert_eq!(pattern(r"a\bc").to_string(), r"abc".to_string());
    assert_eq!(pattern(r"\?\*\a").to_string(), r"\?\*a".to_string());
    assert_eq!(pattern(r"h?\?").to_string(), r"h?\?".to_string());
    assert_eq!(pattern(r"\??????").to_string(), r"\??????".to_string());
    assert_eq!(pattern(r"\**").to_string(), r"\**".to_string());
}

#[test]
fn test_whitespace() {
    assert!(pattern("\n").matches("\n"));
    assert!(pattern("?").matches("\n"));
    assert!(pattern("\t*\n").matches("\t\t\n"));
    assert!(!pattern(" ").matches("\n"));
    assert!(!pattern(" ").matches("\t"));

    assert_eq!(pattern("\n").to_string(), "\n".to_string());
    assert_eq!(pattern("?").to_string(), "?".to_string());
    assert_eq!(pattern("\t*\n").to_string(), "\t*\n".to_string());
    assert_eq!(pattern(" ").to_string(), " ".to_string());
}

#[test]
fn test_issue_3() {
    assert!(!pattern("??*?!?").matches("hello!"));
    assert!(!pattern("hel*???!?**+").matches("hello!"));
    assert!(!pattern("?*??ll*??*w\n").matches("hello!"));

    assert_eq!(pattern("??*?!?").to_string(), "*!?".to_string());
    assert_eq!(pattern("hel*???!?**+").to_string(), "hel*!*+".to_string());
    assert_eq!(pattern("?*??ll*??*w\n").to_string(), "*ll*w\n".to_string());
}

#[test]
fn test_issue_4() {
    assert!(!pattern("??*``*").matches(r"``\ȣ?"));

    assert_eq!(pattern("??*``*").to_string(), "*``*".to_string());
}

fn pattern(pattern: &str) -> Pattern<&'_ str> {
    Pattern::new(pattern)
}
