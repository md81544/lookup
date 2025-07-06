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
fn test_lookup_with_wildcard() {
    let words = vec![
        "arc".to_string(),
        "arch".to_string(),
        "archimedes".to_string(),
    ];
    let results = lookup("arch%", &words, "");
    assert_eq!(results.len(), 2); // should match "arch" and "archimedes" but not shorter words
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

#[test]
fn test_regex_lookup() {
    let words = vec![
        "knelt".to_string(),
        "dodge".to_string(),
        "dryer".to_string(),
        "druid".to_string(),
        "wryly".to_string(),
    ];
    let mut results: Vec<String> = regex_lookup("d", &words);
    assert!(results.len() == 3);
    results = regex_lookup("k", &words);
    assert!(results.len() == 1);
    results = regex_lookup("..d..", &words);
    assert!(results.len() == 1);
    assert_eq!(results[0], "dodge");
    results = regex_lookup("^..y..$", &words);
    assert!(results.len() == 2);
}

#[test]
fn test_reverse() {
    let result = reverse("clock");
    assert!(result.len() == 1);
    assert!(result[0] == "kcolc");
}

#[test]
fn test_regular_patterns() {
    let mut result = regular_patterns("BROSNEERS", false);
    assert!(result.len() == 2);
    assert!(result[0] == "BONES");
    assert!(result[1] == "RSER");
    result = regular_patterns("BROSNEERS", true); // reverse regular patterns
    assert!(result.len() == 2);
    assert!(result[0] == "SENOB");
    assert!(result[1] == "RESR");
    // Check that spaces are ignored in the input
    result = regular_patterns("BRO SNEERS", false);
    assert!(result.len() == 2);
    assert!(result[0] == "BONES");
    assert!(result[1] == "RSER");
}

#[test]
fn test_remove_found_mismatches() {
    let words = vec![
        "knelt".to_string(),
        "dodge".to_string(),
        "dryer".to_string(),
        "druid".to_string(),
        "wryly".to_string(),
        "abc def".to_string(),
        "abcxdef".to_string(),
    ];
    let mut found = "d...e".to_string();
    let mut results = remove_found_mismatches(&words, found, false);
    assert!(results.len() == 1);
    found = "ab...ef".to_string();
    results = remove_found_mismatches(&words, found, false);
    assert!(results.len() == 2);
    found = "ab...ef".to_string();
    results = remove_found_mismatches(&words, found, true); // ignore phrases
    assert!(results.len() == 1);
}

#[test]
fn test_remove_wrong_sized_words() {
    let words = vec![
        "a".to_string(),
        "be".to_string(),
        "cat".to_string(),
        "four".to_string(),
        "table".to_string(),
        "avails".to_string(),
        "flashes".to_string(),
        "walkover".to_string(),
        "subnormal".to_string(),
        "spherality".to_string(),
    ];
    let results = remove_wrong_sized_words(&words, 6);
    assert!(results.len() == 1);
    assert!(results[0] == "avails");
}

#[test]
fn test_expand_numbers() {
    let result = expand_numbers("3e4");
    assert!(result == "___e____");
    let result2 = expand_numbers("14x");
    assert!(result2 == "______________x");
    let result3 = expand_numbers("15");
    assert!(result3 == "_______________");
    let result4 = expand_numbers("1024");
    assert!(result4.len() == 1024);
}
