use clap::{ArgGroup, Parser};
use colored::Colorize;
use itertools::Itertools;
use rand::seq::SliceRandom;
use rand::thread_rng;
use regex::Regex;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::collections::HashSet;
use std::f32::consts::PI;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::process::exit;

// Note, word lists are generated from public domain word lists,
// see http://wordlist.aspell.net/12dicts-readme/

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
// The following incantation defines an argument group of mutually-exclusive command flags
#[clap(group(
    ArgGroup::new("lookups")
        .required(false)
        .args(&["wordle", "spellingbee", "panagram", "lookup", "anagram", "jumble"]),
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

    /// Thesaurus lookup. Can be combined with lookup to filter results: use BOTH -l and -t flags.
    #[arg(short, long, default_value = "")]
    thesaurus: String,

    /// Plain anagram solver
    #[arg(short, long, default_value_t = false)]
    anagram: bool,

    /// Regex lookup - best single quoted, normally you will need ^/$ at beginning/end
    #[arg(short, long, default_value_t = false)]
    regex: bool,

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

    /// Word size (number of characters)
    #[arg(short = 'z', long, default_value_t = 0)]
    size: u8,

    /// Word obscurity level 1 = everyday, 2 = bigger list, 3 = a lot of weird words
    #[arg(short, long, default_value_t = 1)]
    obscurity: u8,

    /// Print regular patterns from phrase
    #[arg(short = 'g', long, default_value_t = false)]
    regular: bool,

    /// Narrow output (one word per line)
    #[arg(short, long, default_value_t = false)]
    narrow: bool,

    /// Debug output
    #[arg(short, long, default_value_t = false)]
    debug: bool,

    // Search string
    #[arg()]
    //#[arg(index(1))]
    search_string: Vec<String>,

    /// Jumble letters (for manual anagram solving)
    #[arg(short, long, default_value_t = false)]
    jumble: bool,

    /// Found letters confirmed in anagram for jumble, e.g. C_M_P_T_R.
    /// Use '/' for spaces, e.g. 'N_/M_NS/L_ND'
    #[arg(short, long, default_value = "", requires = "jumble")]
    found: String,
}

#[derive(Eq, PartialEq)]
enum Action {
    Undefined,
    Wordle,
    Spellingbee,
    Panagram,
    Lookup,
    Anagram,
    Jumble,
    Regex,
    Thesaurus,
    LookupWithThesaurus,
    RegularPatterns,
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
    } else if !args.search_string.is_empty() {
        search_string = args.search_string[0].clone().to_lowercase();
    }

    if search_string.is_empty() && args.thesaurus.is_empty() {
        let _ = cmd.print_help();
        exit(1);
    }

    // Finally allow "/" as word separators in search string (for consistency with the
    // "found" argument which has problems with spaces in it (see comments elsewhere)
    search_string = search_string.replace("/", " ");

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
    let mut thesaurus: Vec<String> = Vec::new();
    let mut lookup_mode = false;
    if phrase_lookup && !args.lookup {
        lookup_mode = true;
    }

    // Word list file must exist in the current path
    let mut vec_index = 0usize;
    if let Ok(lines) = read_lines(&file_name) {
        for word in lines.map_while(Result::ok) {
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

    // Also add phrases to the anagram list
    file_name = "./phrases.txt".to_string();
    if let Ok(lines) = read_lines(&file_name) {
        for word in lines.map_while(Result::ok) {
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
    } else {
        println!("Could not read {}", file_name);
        exit(2);
    }

    // Also read in thesaurus if required
    if !args.thesaurus.is_empty() {
        file_name = "./thesaurus.txt".to_string();
        if let Ok(lines) = read_lines(&file_name) {
            for line in lines.map_while(Result::ok) {
                //if line.starts_with(&(search_string.to_string() + ",")) {
                if line.starts_with(&(args.thesaurus.to_string() + ",")) {
                    let words = line.split(",");
                    let mut first: bool = true;
                    for word in words {
                        if first {
                            first = false;
                        } else {
                            thesaurus.push(word.to_string());
                        }
                    }
                }
            }
        }
    }

    let mut results: Vec<String> = Vec::new();

    let mut action: Action = Action::Undefined;

    // Check which action flags have been set and act accordingly
    if args.panagram {
        action = Action::Panagram;
    }
    if args.spellingbee {
        action = Action::Spellingbee;
    }
    if args.wordle {
        action = Action::Wordle;
    }
    if args.anagram {
        action = Action::Anagram;
    }
    if args.lookup {
        action = Action::Lookup;
    }
    if args.jumble {
        action = Action::Jumble;
    }
    if args.regex {
        action = Action::Regex;
    }
    if args.regular {
        action = Action::RegularPatterns;
    }
    // If none of the "types" are set then we try to infer which type
    // is required from the input
    if action == Action::Undefined {
        let mut msg = String::from("No game type specified, assuming ");
        if lookup_mode || search_string.contains('_') {
            action = Action::Lookup;
            msg += "lookup";
        } else if search_string.len() == 5 {
            action = Action::Wordle;
            msg += "Wordle";
        } else if search_string.len() == 9 {
            action = Action::Panagram;
            msg += "Panagram";
        } else if search_string.len() == 7 {
            action = Action::Spellingbee;
            msg += "Spelling Bee";
        } else {
            action = Action::Anagram;
            msg += "anagram";
        }
        println!("{}", msg.yellow())
    }
    if !args.thesaurus.is_empty() {
        if action == Action::Lookup {
            action = Action::LookupWithThesaurus;
        } else {
            action = Action::Thesaurus;
        }
    }

    if action == Action::Panagram {
        results = panagram(&search_string, &word_list, &anagrams);
    } else if action == Action::Spellingbee {
        results = spellingbee(&search_string, &word_list, args.debug);
    } else if action == Action::Wordle {
        if search_string.len() != 5 {
            println!("Search string is not five characters");
            exit(6);
        }
        results = wordle(&search_string, &word_list, &args.exclude, &args.include);
    } else if action == Action::Anagram {
        results = anagram_search(&search_string, &word_list, &anagrams);
    } else if action == Action::Lookup || action == Action::LookupWithThesaurus {
        results = lookup(&search_string, &word_list, "");
        if action == Action::LookupWithThesaurus {
            // we need to remove any words which don't exist in the 'thesaurus' vector
            results.retain(|item| thesaurus.contains(item));
        }
    } else if action == Action::Regex {
        results = regex_lookup(&search_string, &word_list, "");
    } else if action == Action::Jumble {
        let letters = args.found.clone();
        // Note! Clap can't seem to cope with spaces in arguments, even if quoted. So
        // we use '/' in the "found" string to indicate word boundaries, e.g. "N_/M_NS/L_ND"
        let letters_no_spaces: String = letters.replace("/", "");
        if letters_no_spaces.len() > 0 && letters_no_spaces.len() != search_string.len() {
            println!("Error: 'found' letters must be same length as search string");
            exit(7);
        }
        jumble(&search_string.to_uppercase(), &letters.to_uppercase());
    } else if action == Action::Thesaurus {
        results = thesaurus;
    } else if action == Action::RegularPatterns {
        results = regular_patterns(&search_string.to_uppercase());
    }

    if args.size != 0 {
        results = remove_wrong_sized_words(&results, args.size);
    }

    results.sort();
    display_results(&results, &search_string, action, args.narrow);
    exit(0);
}

fn remove_wrong_sized_words(results: &[String], length: u8) -> Vec<String> {
    let mut new_results: Vec<String> = Vec::new();
    for word in results {
        if word.len() == length.into() {
            new_results.push(word.to_string());
        }
    }
    new_results
}

fn display_results(results: &Vec<String>, search_string: &str, action: Action, narrow: bool) {
    for word in results {
        if word.contains(char::is_whitespace) && !narrow {
            print!("'");
        }
        if (action == Action::Panagram && word.len() == 9)
            || (action == Action::Spellingbee && word_is_pangram(word, search_string))
        {
            print!("{}", word.to_uppercase().bold());
        } else {
            print!("{}", word);
        }
        if word.contains(char::is_whitespace) && !narrow {
            print!("'");
        }
        print_separator(narrow);
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
    let mut results: HashSet<String> = HashSet::new();
    for word in word_list {
        let mut matched = true;
        if word.len() != search_string.len() {
            continue;
        }
        for i in 0..word.len() {
            let c = word.as_bytes()[i] as char;
            let search_char = search_string.as_bytes()[i] as char;
            // Only exclude characters if they aren't explicitly at this position in the
            // search string, meaning "a___t -x a" would still match "avast", for example
            if c != search_char && exclude.contains(c) {
                matched = false;
                break;
            }
            if search_string.as_bytes()[i] == 95 {
                // i.e. '_'
                // wildcard (underscore) - we only pass this if the character we're comparing
                // is not a space (i.e. we wouldn't want "__ _____" to match "AA AA AA")
                if word.as_bytes()[i] == 32 {
                    // i.e. ' '
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
            results.insert(word.to_string());
        }
    }
    results.into_iter().collect()
}

fn regex_lookup(search_string: &str, word_list: &[String], _exclude: &str) -> Vec<String> {
    let mut results: Vec<String> = Vec::new();
    let re = Regex::new(search_string).unwrap();

    for word in word_list {
        if re.is_match(word) {
            results.push(word.to_string());
        }
    }
    results
}

fn regular_patterns(search_string: &str) -> Vec<String> {
    // Just give a selection of regular letters from the search string, e.g.
    // "BRO SNEERS" could yield "BONES" and "RSER"
    let mut results: Vec<String> = Vec::new();
    let mut count = 0;
    let mut word_evens: String = String::new();
    let mut word_odds: String = String::new();
    for c in search_string.chars() {
        if count % 2 == 0 {
            word_evens.push(c);
        } else {
            word_odds.push(c);
        }
        count += 1;
    }
    results.push(word_evens);
    results.push(word_odds);
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

fn jumble(full_input: &str, found_letters: &str) {
    // Remove underscores from found_letters
    let mut input: String = full_input.to_string();
    for c in found_letters.chars() {
        if c != '_' && c != '/' {
            if let Some(pos) = input.find(c) {
                input.remove(pos);
            } else {
                println!(
                    "Error: You supplied a letter ({}) in the found (-f) option",
                    c
                );
                println!("which does not appear in the source set of letters");
                return;
            }
        }
    }
    println!();
    let mut chars: Vec<char> = input.chars().collect();
    if chars.len() % 2 == 1 {
        chars.push(' ');
    }
    let len = chars.len();

    let mut rng = thread_rng();
    chars.shuffle(&mut rng);

    let radius = ((len as f32 / PI).sqrt().ceil()) as usize;
    let mut grid = vec![vec![' '; radius * 4 + 1]; radius * 2 + 1];

    for i in 0..len / 2 {
        let angle = (i as f32 / (len / 2) as f32) * PI;

        let x1 = (radius as f32 * angle.cos()).round() as isize;
        let y1 = (radius as f32 * angle.sin()).round() as isize;

        let x2 = -(radius as f32 * angle.cos()).round() as isize;
        let y2 = -(radius as f32 * angle.sin()).round() as isize;

        grid[(y1 + radius as isize) as usize][(x1 * 2 + radius as isize * 2) as usize] =
            chars[i * 2].to_ascii_uppercase();
        grid[(y2 + radius as isize) as usize][(x2 * 2 + radius as isize * 2) as usize] =
            chars[i * 2 + 1].to_ascii_uppercase();
    }

    for row in grid {
        println!("  {}", row.iter().collect::<String>());
    }
    println!();
    print!("  ");
    for c in found_letters.chars() {
        if c == '/' {
            print!("  ");
        } else {
            print!("{} ", c.to_ascii_uppercase());
        }
    }
    println!();
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
    for i in 0..yellow_letters.len() {
        let c = yellow_letters.as_bytes()[i] as char;
        if !word.contains(c) {
            return false;
        }
    }
    true
}

fn print_separator(narrow: bool) {
    if narrow {
        println!();
    } else {
        print!(" ");
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
    // Strip all whitespace
    let no_space: String = word.chars().filter(|c| !c.is_whitespace()).collect();
    no_space.chars().sorted().collect::<String>()
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
}
