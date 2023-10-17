use clap::{ArgGroup, Parser};
use colored::Colorize;
use itertools::Itertools;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::process::exit;

// Note, word lists are generated from public domain word lists,
// see http://wordlist.aspell.net/12dicts-readme/

// TODO all word lists are expected to be in ASCII but we use String (i.e. UTF-8) throughout
//      so it would _probably_ be an optimisation to use bytes instead
// * Wordle: need an easy way to say that a yellow character shouldn't be in a particular position
//   (not sure how practical this might be!)
// * Wordle: might be an idea to order results by most common letters?
// * (To think about) - maybe some way of marking off words tried in 'panagram' - perhaps
//   an interface where the words disappear when selected? Curses?

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
// The following incantation defines an argument group of mutually-exclusive command flags
#[clap(group(
    ArgGroup::new("lookups")
        .required(true)
        .args(&["wordle", "spellingbee", "panagram", "lookup", "anagram"]),
))]
struct Args {
    /// "Panagram" search (Telegraph Puzzles). Put the mandatory letter first in the search string.
    #[arg(short, long, default_value_t = false)]
    panagram: bool,
    /// "Spelling Bee" search (NYT Puzzles). Put the mandatory letter first in the search string.
    #[arg(short, long, default_value_t = false)]
    spellingbee: bool,
    /// "Wordle" search (NYT Puzzles). Use, e.g. k____ to signify K is "green", use -i to include
    /// "yellow" letters, and -x to exclude "grey" letters
    #[arg(short, long, default_value_t = false)]
    wordle: bool,
    /// Plain anagram solver
    #[arg(short, long, default_value_t = false)]
    anagram: bool,
    /// Letters to include ("yellow") for Wordle search
    #[arg(short, long, default_value = "", requires = "wordle")]
    include: String,
    /// Letters to exclude ("grey") for Wordle search
    #[arg(short = 'x', long, default_value = "", requires = "wordle")]
    exclude: String,
    /// Lookup partial match, e.g. "c_mp_t_r" would yield "computer". You can also look up
    /// phrases, for example "l_k_ m_g_c" would match "like magic".
    #[arg(short, long, default_value_t = false)]
    lookup: bool,
    /// Word obscurity level 1 = everyday, 2 = bigger list, 3 = a lot of weird words
    #[arg(short, long, default_value_t = 1)]
    obscurity: u8,
    /// Wide output
    #[arg(long, default_value_t = false)]
    wide: bool,
    /// Debug output
    #[arg(short, long, default_value_t = false)]
    debug: bool,
    // Search string
    #[arg()]
    //#[arg(index(1))]
    search_string: Vec<String>,
}

fn main() {
    use clap::CommandFactory;
    let mut cmd = Args::command();
    let args = Args::parse();

    if args.obscurity < 1 || args.obscurity > 3 {
        println!("{}", "\nError: Invalid word obscurity level".red());
        let _ = cmd.print_help();
        exit(10);
    }

    let mut phrase_lookup = false;
    // The search string can be multiple words, if it is we infer it's a phrase lookup.
    let mut search_string = "".to_string();
    if args.search_string.len() > 1 {
        phrase_lookup = true;
        for word in args.search_string {
            if !search_string.is_empty() {
                search_string += " ";
            }
            search_string += &word.to_lowercase();
        }
    } else {
        search_string = args.search_string[0].clone();
    }

    let mut file_name = format!("./words_{}.txt", args.obscurity).to_string();
    if args.debug {
        // very small file for testing
        file_name = "./words_debug.txt".to_string();
    }
    if phrase_lookup {
        file_name = "./phrases.txt".to_string();
    }
    let mut anagrams: HashMap<String, Vec<usize>> = HashMap::new();
    let mut word_list: Vec<String> = Vec::new();
    if phrase_lookup && !args.lookup {
        println!(
            "{}",
            "\nError: can only look for multiple words in --lookup mode".red()
        );
        let _ = cmd.print_help();
        exit(11);
    }

    // Word list file must exist in the current path
    let mut vec_index = 0usize;
    if let Ok(lines) = read_lines(&file_name) {
        for word in lines.flatten() {
            if args.wordle {
                // if we're doing a wordle lookup, we're only interested in five-letter words
                // and we don't care about anagrams
                if word.len() == 5 {
                    word_list.push(word.clone());
                }
            } else {
                word_list.push(word.clone());
                let anagram = sort_word(&word);
                // Does this entry already exist? Add to vec if so, else create new vec
                match anagrams.entry(anagram) {
                    Entry::Vacant(e) => {
                        e.insert(vec![vec_index]);
                    }
                    Entry::Occupied(mut e) => {
                        e.get_mut().push(vec_index);
                    }
                }
                vec_index += 1;
            }
        }
    } else {
        println!("Could not read {}", file_name);
        exit(2);
    }

    let mut results: Vec<String> = Vec::new();

    // Check which action flags have been set and act accordingly
    if args.panagram {
        results = panagram(&search_string, &word_list, &anagrams);
    }

    if args.spellingbee {
        results = spellingbee(&search_string, &word_list, args.debug);
    }

    if args.wordle {
        if search_string.len() != 5 {
            println!("Search string is not five characters");
            exit(6);
        }
        results = wordle(&search_string, &word_list, &args.exclude, &args.include);
    }

    if args.anagram {
        results = anagram_search(&search_string, &word_list, &anagrams);
    }

    if args.lookup {
        results = lookup(&search_string, &word_list, "");
    }

    results.sort();
    display_results(
        &results,
        &search_string,
        args.panagram,
        args.spellingbee,
        args.wide,
    );
    exit(0);
}

fn display_results(
    results: &Vec<String>,
    search_string: &str,
    panagram: bool,
    spellingbee: bool,
    wide: bool,
) {
    for word in results {
        if (panagram && word.len() == 9) || (spellingbee && word_is_pangram(word, search_string)) {
            print!("{}", word.to_uppercase().bold());
        } else {
            print!("{}", word);
        }
        print_separator(wide);
    }
    println!();
}

fn word_is_pangram(word: &str, search_string: &str) -> bool {
    if word.len() < 7 {
        return false;
    }
    for c in search_string.chars() {
        if !word.contains(c) {
            return false;
        }
    }
    true
}

fn lookup(search_string: &str, word_list: &[String], exclude: &str) -> Vec<String> {
    let mut results: Vec<String> = Vec::new();
    for word in word_list {
        let mut matched = true;
        if word.len() != search_string.len() {
            continue;
        }
        for i in 0..word.as_bytes().len() {
            let c = word.as_bytes()[i] as char;
            let search_char = search_string.as_bytes()[i] as char;
            // Only exclude characters if they aren't explicitly at this position in the
            // search string, meaning "a___t -x a" would still match "avast", for example
            if c != search_char && exclude.contains(c) {
                matched = false;
                break;
            }
            if search_string.as_bytes()[i] == 95 {
                // wildcard (underscore) - we only pass this if the character we're comparing
                // is not a space (i.e. we wouldn't want "__ _____" to match "AA AA AA")
                if word.as_bytes()[i] == 32 {
                    matched = false;
                    break;
                }
                continue;
            }
            if search_string.as_bytes()[i] != word.as_bytes()[i] {
                matched = false;
                break;
            }
        }
        if matched {
            results.push(word.to_string());
        }
    }
    results
}

fn anagram_search(
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

fn panagram(
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

fn spellingbee(search_string: &str, word_list: &Vec<String>, debug: bool) -> Vec<String> {
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

fn wordle(search_string: &str, word_list: &[String], exclude: &str, include: &str) -> Vec<String> {
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

fn check_yellow_letters_exist(w: &str, search_string: &str, yellow_letters: &str) -> bool {
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
    for i in 0..yellow_letters.as_bytes().len() {
        let c = yellow_letters.as_bytes()[i] as char;
        if !word.contains(c) {
            return false;
        }
    }
    true
}

fn print_separator(wide: bool) {
    if wide {
        print!(" ");
    } else {
        println!();
    }
}

fn read_lines<P>(filename: &P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn sort_word(word: &str) -> String {
    word.chars().sorted().collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_word() {
        assert_eq!("aaeeilmort", sort_word("ameliorate"));
    }

    #[test]
    fn test_spellingbee() {
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
        ];
        let results = lookup("f_o_ni__", &words, "");
        assert_eq!(results.len(), 1); // should match "frobnish"
        let results2 = lookup("s__v", &words, "");
        assert_eq!(results2.len(), 0); // should not match anything
        let results3 = lookup("fra_____", &words, "z");
        assert_eq!(results3.len(), 0); // should not match anything
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
        let results2= lookup("a d________", &words, "");
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
        let words = vec![
            "adult".to_string(),
        ];
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
}
