pub fn remove_trailing_spaces(s: &str) -> String {
    s.replace("\n", "").split(' ').filter(|&e| e != "").collect::<Vec<&str>>().join(" ")
}

pub fn split_by_whitespace_trimmed(s: &str) -> Vec<String> {
    s.replace("\n", "").split(' ').filter(|&e| e != "").map(|e| e.to_string()).collect()
}

pub fn first_until_whitespace(s: &str) -> String {
    s.trim().split(' ').next().unwrap_or("").to_string()
} 

#[test]
fn test()
{
    let tester = remove_trailing_spaces("   a   b       c  ");
    assert_eq!(tester.as_str(), "a b c");

    let tester = split_by_whitespace_trimmed("   a   bb       c  ");
    assert_eq!(tester, vec!["a", "bb", "c"]);

    let tester = first_until_whitespace("   a   bb       c  ");
    assert_eq!(tester.as_str(), "a");

    let tester = first_until_whitespace("test   bb       c  ");
    assert_eq!(tester.as_str(), "test");

    let tester = first_until_whitespace("");
    assert_eq!(tester.as_str(), "");
}