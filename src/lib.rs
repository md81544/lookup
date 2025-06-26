use std::{
    collections::{HashMap, HashSet},
    process::exit,
};

use itertools::Itertools;

pub fn sort_word(word: &str) -> String {
    // Strip all whitespace
    let no_space: String = word.chars().filter(|c| !c.is_whitespace()).collect();
    no_space.chars().sorted().collect::<String>()
}

pub fn spellingbee(search_string: &str, word_list: &Vec<String>, debug: bool) -> Vec<String> {
    let mut results: Vec<String> = Vec::new();
    let mut included_chars = "".to_string();
    let mut excluded_chars = "".to_string();
    for i in 97u8..=122 {
        // 'a' to 'z'
        let t = i as char;
        let c = t.to_string();
        if !search_string.contains(&c) {
            excluded_chars += c.as_str();
        } else {
            included_chars += c.as_str();
        }
    }
    if debug {
        println!("Excluded characters: [{}]", excluded_chars);
        println!("Included characters: [{}]", included_chars);
    }
    let min_len = 4;
    for word in word_list {
        let mut invalid = false;
        if debug {
            print!("\"{}\" : ", word);
        }
        if word.len() < min_len {
            if debug {
                println!("is too short");
            }
            continue;
        }
        for c in word.chars() {
            if excluded_chars.contains(c) {
                if debug {
                    println!("contains excluded char '{}'", c);
                }
                invalid = true;
                break;
            }
        }
        if invalid {
            continue;
        }
        // We now just have to ensure the word contains the mandatory letter
        // which should be the first letter of the search string
        let c = search_string.chars().next().unwrap();
        if !word.contains(c) {
            continue;
        }
        // If we get here, we haven't failed any checks, so it's a match
        if debug {
            println!(" *** match ***");
        } else {
            results.push(word.to_string());
        }
    }
    results
}

pub fn panagram(
    search_string: &str,
    word_list: &[String],
    anagrams: &HashMap<String, Vec<usize>>,
) -> Vec<String> {
    let mut results: Vec<String> = Vec::new();
    if search_string.len() != 9 {
        println!("Error: search string must have nine letters");
        exit(3);
    }
    let mut chars: Vec<_> = search_string.chars().collect();
    let required_letter = chars[0];
    chars.remove(0);
    let mut lookups = HashSet::new();
    for i in 3..9 {
        for word in chars.iter().permutations(i).unique() {
            let w1: String = required_letter.to_string();
            let w2: String = word.into_iter().collect();
            let word_str: String = sort_word(&format!("{}{}", w1, w2));
            lookups.insert(word_str);
        }
    }
    for word in lookups {
        if let Some(indices) = anagrams.get(&word) {
            for idx in indices {
                results.push(word_list[*idx].to_string());
            }
        }
    }
    results
}

pub fn anagram_search(
    search_string: &str,
    word_list: &[String],
    anagrams: &HashMap<String, Vec<usize>>,
) -> Vec<String> {
    let mut results: Vec<String> = Vec::new();
    let search_string = sort_word(search_string);
    if let Some(indices) = anagrams.get(&search_string) {
        for idx in indices {
            results.push(word_list[*idx].to_string());
        }
    }
    results
}

pub fn lookup(search_string: &str, word_list: &[String], exclude: &str) -> Vec<String> {
    let mut results: HashSet<String> = HashSet::new();
    for word in word_list {
        let mut matched = true;
        if word.len() != search_string.len() && !search_string.contains('%') {
            continue;
        }
        for i in 0..word.len() {
            let c = word.as_bytes()[i] as char;
            let mut search_char = search_string.as_bytes()[i] as char;
            if search_char == '/' {
                search_char = ' ';
            }
            // Only exclude characters if they aren't explicitly at this position in the
            // search string, meaning "a___t -x a" would still match "avast", for example
            if c != search_char && exclude.contains(c) {
                matched = false;
                break;
            }
            if search_char == '_' || search_char == '.' {
                // wildcard - we only pass this if the character we're comparing
                // is not a space (i.e. we wouldn't want "__ _____" to match "AA AA AA")
                if word.as_bytes()[i] == b' ' {
                    matched = false;
                    break;
                }
                continue;
            }
            if search_char == '%' {
                // match any word past this point
                break;
            }
            if search_char != word.as_bytes()[i] as char {
                matched = false;
                break;
            }
        }
        if matched {
            results.insert(word.to_string());
        }
    }
    results.into_iter().collect()
}

pub fn wordle(
    search_string: &str,
    word_list: &[String],
    exclude: &str,
    include: &str,
) -> Vec<String> {
    // First we do a lookup using just the "green" letters
    // (i.e. those supplied in the search string), excluding the exclude letters:
    let results = lookup(search_string, word_list, exclude);
    // Now we can go through the results and weed out items that don't have the "yellow" letters
    let mut matches: Vec<String> = Vec::new();
    for word in &results {
        if check_yellow_letters_exist(word, search_string, include) {
            matches.push(word.clone());
        }
    }
    matches
}

pub fn check_yellow_letters_exist(w: &str, search_string: &str, yellow_letters: &str) -> bool {
    // check that all "yellow" letters in the search_string exist in the word
    // BUT not at their position in the search string
    // we can also ignore any matches at positions which are "green"
    // To simplify the logic we remove any "green" letters from the word first
    let mut word = w.to_string();
    for i in 0..5 {
        if search_string.as_bytes()[i].is_ascii_alphabetic() {
            word.replace_range(i..=i, "."); // replace with an arbitrary non alpha character
        }
    }
    // Now we can just check all of the yellow letters exist
    for i in 0..yellow_letters.len() {
        let c = yellow_letters.as_bytes()[i] as char;
        if !word.contains(c) {
            return false;
        }
    }
    true
}

pub fn expand_numbers(search_string: &str) -> String {
    let mut res = "".to_string();
    let mut num = 0;
    for c in search_string.chars() {
        if c.is_numeric() {
            if num != 0 {
                num *= 10;
            }
            num += c.to_digit(10).unwrap();
        } else {
            if num != 0 {
                for _ in 0..num {
                    res.push('_');
                }
            }
            if c == ' ' {
                res.push('/');
            } else {
                res.push(c);
            }
            num = 0;
        }
    }
    if num != 0 {
        for _ in 0..num {
            res.push('_');
        }
    }
    res
}
