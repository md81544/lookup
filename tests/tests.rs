use std::collections::HashMap;

use lookup::*;

#[test]
fn test_sort_word() {
    assert_eq!("aaeeilmort", sort_word("ameliorate"));
}

#[test]
fn test_spellingbee() {
    use lookup::spellingbee;
    let words = vec![
        "doctoral".to_string(),
        "cartload".to_string(),
        "frobnish".to_string(),
    ];
    let results = spellingbee("roldact", &words, false);
    assert_eq!(results.len(), 2); // should match "doctoral" and "cartload"
}

#[test]
fn test_panagram() {
    let words = vec!["cartload".to_string(), "plaintiff".to_string()];
    let mut anagrams: HashMap<String, Vec<usize>> = HashMap::new();
    anagrams.insert("aacdlort".to_string(), vec![0usize]);
    anagrams.insert("affiilnpt".to_string(), vec![1usize]);
    let results = panagram("infaflipt", &words, &anagrams);
    assert_eq!(results.len(), 1); // should match "plaintiff"
}

#[test]
fn test_anagram_search() {
    let words = vec!["cartload".to_string(), "plaintiff".to_string()];
    let mut anagrams: HashMap<String, Vec<usize>> = HashMap::new();
    anagrams.insert("aacdlort".to_string(), vec![0usize]);
    anagrams.insert("affiilnpt".to_string(), vec![1usize]);
    let results = anagram_search("infaflipt", &words, &anagrams);
    assert_eq!(results.len(), 1); // should match "plaintiff"
    let results2 = anagram_search("frobnish", &words, &anagrams);
    assert_eq!(results2.len(), 0); // should not match any
}

#[test]
fn test_lookup() {
    let words = vec![
        "doctoral".to_string(),
        "cartload".to_string(),
        "frobnish".to_string(),
        "frazzled".to_string(),
        "not care".to_string(),
    ];
    let results = lookup("f_o_ni__", &words, "");
    assert_eq!(results.len(), 1); // should match "frobnish"
    let results2 = lookup("s__v", &words, "");
    assert_eq!(results2.len(), 0); // should not match anything
    let results3 = lookup("fra_____", &words, "z");
    assert_eq!(results3.len(), 0); // should not match anything
    let results4 = lookup("not/c___", &words, "z");
    assert_eq!(results4.len(), 1); // should match "not care"
}
#[test]

fn test_lookup_phrase() {
    let words = vec![
        "i feel fine".to_string(),
        "a fine mess".to_string(),
        "a dead duck".to_string(),
        "a dandelion".to_string(),
    ];
    let results = lookup("a d___ ___k", &words, "");
    assert_eq!(results.len(), 1); // should match "a dead duck"
    let results2 = lookup("a d________", &words, "");
    assert_eq!(results2.len(), 1); // should only match "a dandelion", not "a dead duck"
}

#[test]
fn test_wordle() {
    let words = vec![
        "knelt".to_string(),
        "dodge".to_string(),
        "dryer".to_string(),
        "druid".to_string(),
        "wryly".to_string(),
    ];
    // We are specifically testing that wordle() finds two Ys in the results, and
    // not simply matching both against the green letter
    let results = wordle("_ry__", &words, "", "y"); // exclude, include
    assert_eq!(results.len(), 1); // should only match "wryly"

    let results2 = wordle("_____", &words, "", "er");
    assert_eq!(results2.len(), 1); // should only match "dryer"
    assert_eq!(results2[0], "dryer");

    let results3 = wordle("dr___", &words, "y", "");
    assert_eq!(results3.len(), 1); // should only match "druid" because we exclude y

    // What if the use includes a letter that is already "green"? This signifies
    // that there's ANOTHER yellow d
    let results4 = wordle("d____", &words, "", "d");
    assert_eq!(results4.len(), 2); // should only match "druid", and "dodge"
}

#[test]
fn test_wordle_exclude_green() {
    let words = vec!["adult".to_string()];
    // Case where the user might have excluded a letter which is also in the search
    // string (i.e. is "green"). This should exclude words that have the excluded letter
    // in any position OTHER than the supplied green one.
    let results = wordle("a___t", &words, "a", ""); // exclude, include
    assert_eq!(results.len(), 1); // should match
}

#[test]
fn test_yellow_check() {
    assert_eq!(true, check_yellow_letters_exist("dryer", "__y__", "er"));
    assert_eq!(false, check_yellow_letters_exist("dryer", "__y__", "ery")); // no second y
    assert_eq!(true, check_yellow_letters_exist("dryer", "d___r", "")); // no yellow letters
}

#[test]
fn test_number_expansion() {
    let mut ss1 = "3f3".to_string();
    ss1 = expand_numbers(&ss1);
    assert_eq!(ss1, "___f___");
    let mut ss2 = "3/x5".to_string();
    ss2 = expand_numbers(&ss2);
    assert_eq!(ss2, "___/x_____");
    let mut ss3 = "11/z4".to_string();
    ss3 = expand_numbers(&ss3);
    assert_eq!(ss3, "___________/z____");
}
