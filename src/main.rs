use clap::{ArgGroup, Parser};
use colored::Colorize;
use rand::seq::SliceRandom;
use rand::thread_rng;
use regex::Regex;
use std::collections::HashMap;
use std::process::exit;

use lookup::*;

pub mod file;
pub mod ui;

// Note, word lists are generated from public domain word lists,
// see http://wordlist.aspell.net/12dicts-readme/
// Definitions are from https://github.com/wordset/wordset-dictionary

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
// The following incantation defines an argument group of mutually-exclusive command flags
#[clap(group(
    ArgGroup::new("lookups")
        .required(false)
        .args(&["wordle", "spellingbee", "panagram", "lookup", "jumble"]),
))]
// Note, this magic incantation way of defining arguments for clap is called "derive"
// (see https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html)
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

    /// Display a word's definition only
    #[arg(short, long, default_value = "", num_args = 1..)]
    define: Vec<String>,

    /// Plain anagram solver
    #[arg(short, long, default_value_t = false)]
    anagram: bool,

    /// Regex lookup - best single quoted, normally you will need ^/$ at beginning/end
    #[arg(short = 'R', long, default_value_t = false)]
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
    #[arg(short, long, default_value_t = 3)]
    obscurity: u8,

    /// Reverse string (useful for reverse 'in' clues)
    #[arg(short = 'v', long, default_value_t = false)]
    reverse: bool,

    /// Print regular patterns from phrase
    #[arg(short = 'g', long, default_value_t = false)]
    regular: bool,

    /// Narrow output (one word per line)
    #[arg(short, long, default_value_t = false)]
    narrow: bool,

    /// Debug output
    #[arg(short = 'D', long, default_value_t = false)]
    debug: bool,

    // Search string
    #[arg()]
    //#[arg(index(1))]
    search_string: Vec<String>,

    /// Jumble letters (for manual anagram solving)
    #[arg(short, long, default_value_t = false)]
    jumble: bool,

    /// Exclude phrases in results (single words only)
    #[arg(short, long, default_value_t = false)]
    excludephrases: bool,

    /// Found letters confirmed in anagram for jumble, e.g. C_M_P_T_R.
    /// Use '/' for spaces, e.g. 'N_/M_NS/L_ND'
    #[arg(short, long, default_value = "")]
    found: String,

    /// Comment (for comments only, does nothing)
    #[arg(short, long, default_value = "", num_args = 1..)]
    comment: String,

    /// Remove letters interactively
    #[arg(short, long, default_value_t = false)]
    remove: bool,
}

#[derive(Eq, PartialEq)]
pub enum Action {
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
    RegexWithThesaurus,
    RegularPatterns,
    Reverse,
    Remove,
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

    if !args.define.is_empty() && !args.define[0].is_empty() {
        let combined = args.define.join(" ").to_lowercase();
        define(&combined);
        exit(0);
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

    // We allow numbers in the search string, these represent the number of "blanks".
    // so for example -f 3f7 would result in "...f......."
    search_string = expand_numbers(&search_string);

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

    if args.remove {
        ui::display::interactive_remove(search_string);
        exit(0);
    }

    // Word list file must exist in the current path
    let mut vec_index: usize = 0usize;
    if args.wordle {
        file::load::wordle(&mut word_list, &file_name);
    } else {
        file::load::full_list(&mut word_list, &mut anagrams, &file_name, &mut vec_index);
    }

    // Also read in thesaurus if required
    if !args.thesaurus.is_empty() {
        file::load::thesaurus(&mut thesaurus, &(args.thesaurus.to_string()));
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
    if args.lookup {
        action = Action::Lookup;
    }
    if args.jumble {
        action = Action::Jumble;
    }
    if args.regex {
        action = Action::Regex;
    }
    if args.regular && args.reverse {
        action = Action::RegularPatterns;
    } else {
        if args.regular {
            action = Action::RegularPatterns;
        }
        if args.reverse {
            action = Action::Reverse;
        }
    }
    if args.anagram {
        action = Action::Anagram;
    }
    if search_string.contains('%') {
        action = Action::Lookup;
    }
    // If none of the "types" are set then we try to infer which type
    // is required from the input
    if action == Action::Undefined {
        let mut msg = String::from("No game type specified, assuming ");
        if lookup_mode || search_string.contains('_') || search_string.contains('.') {
            action = Action::Lookup;
            msg += "lookup";
        } else {
            action = Action::Jumble;
            msg += "jumble";
        }
        if args.thesaurus.is_empty() {
            println!("{}", msg.yellow())
        }
    }
    if !args.thesaurus.is_empty() {
        if action == Action::Lookup {
            action = Action::LookupWithThesaurus;
        } else if action == Action::Regex {
            action = Action::RegexWithThesaurus;
        } else {
            action = Action::Thesaurus;
        }
    }

    // Also add phrases to the word list
    // unless excluded or game type is wordle, spellingbee, or panagram
    if !args.excludephrases
        && !args.debug
        && action != Action::Spellingbee
        && action != Action::Panagram
        && action != Action::Wordle
    {
        file_name = "./phrases.txt".to_string();
        file::load::full_list(&mut word_list, &mut anagrams, &file_name, &mut vec_index);
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
        if search_string.contains('%') && search_string.find('%') != Some(search_string.len() - 1) {
            println!("Error: '%' wildcard must only be used at end of search string");
            exit(8);
        }
        results = lookup(&search_string, &word_list, "");
        if action == Action::LookupWithThesaurus {
            // we need to remove any words which don't exist in the 'thesaurus' vector
            results.retain(|item| thesaurus.contains(item));
        }
    } else if action == Action::Regex {
        results = regex_lookup(&search_string, &word_list, "");
    } else if action == Action::RegexWithThesaurus {
        results = regex_lookup(&search_string, &thesaurus, "");
    } else if action == Action::Jumble {
        let mut letters = args.found.clone();
        letters = expand_numbers(&letters);
        // Note we use '/' in the "found" string to indicate word boundaries, e.g. "N_/M_NS/L_ND"
        let letters_no_spaces: String = letters.replace("/", "");
        if !letters_no_spaces.is_empty() && letters_no_spaces.len() > search_string.len() {
            println!("Error: 'found' letters must be same length as search string");
            exit(7);
        }
        if letters_no_spaces.len() < search_string.len() {
            for _ in 0..search_string.len() - letters_no_spaces.len() {
                letters.push('_');
            }
        }
        jumble(
            &search_string.to_uppercase(),
            &letters.to_uppercase(),
            args.size,
        );
    } else if action == Action::Thesaurus {
        results = thesaurus;
    } else if action == Action::RegularPatterns {
        results = regular_patterns(&search_string.to_uppercase(), args.reverse);
    } else if action == Action::Reverse {
        results = reverse(&search_string.to_uppercase());
    }

    if !args.found.is_empty() {
        results = remove_wrong_sized_words(&results, args.size);
    }

    if !args.found.is_empty() {
        results = remove_found_mismatches(&results, args.found, args.excludephrases);
    }

    results.sort();
    ui::display::show_results(&results, &search_string, action, args.narrow);
    exit(0);
}

fn remove_found_mismatches(
    results: &[String],
    found: String,
    exclude_phrases: bool,
) -> Vec<String> {
    let found_letters = expand_numbers(&found);
    let mut new_results: Vec<String> = Vec::new();
    let mut regex_string = "(?i)^".to_string();
    for i in 0..found_letters.len() {
        if found_letters.as_bytes()[i] == b'_' {
            regex_string.push('.');
        } else if found_letters.as_bytes()[i] == b'%' {
            regex_string.push_str(".*");
            break;
        } else if found_letters.as_bytes()[i] == b'/' {
            regex_string.push(' ');
        } else {
            regex_string.push(found_letters.as_bytes()[i] as char);
        }
    }
    if !regex_string.contains(".*") {
        regex_string.push_str(".*");
    }
    let re = Regex::new(&regex_string).unwrap();
    for word in results {
        if exclude_phrases && word.contains(' ') {
            continue;
        }
        if re.is_match(word) {
            new_results.push(word.to_string());
        }
    }
    new_results
}

fn remove_wrong_sized_words(results: &[String], length: u8) -> Vec<String> {
    let mut new_results: Vec<String> = Vec::new();
    for word in results {
        let mut w = word.clone();
        w.retain(|c| !c.is_whitespace());
        if w.len() == length.into() {
            new_results.push(word.to_string());
        }
    }
    new_results
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

fn regular_patterns(search_string: &str, reverse: bool) -> Vec<String> {
    // Just give a selection of regular letters from the search string, e.g.
    // "BRO SNEERS" could yield "BONES" and "RSER"
    // If the reverse flag is specified we do it in reverse
    let mut word: String;
    word = search_string.to_string();
    if reverse {
        word = word.chars().rev().collect();
    }
    let mut results: Vec<String> = Vec::new();
    let mut word_evens: String = String::new();
    let mut word_odds: String = String::new();
    for (count, c) in word.chars().enumerate() {
        if count % 2 == 0 {
            word_evens.push(c);
        } else {
            word_odds.push(c);
        }
    }
    results.push(word_evens);
    results.push(word_odds);
    results
}

fn reverse(search_string: &str) -> Vec<String> {
    // Just reverse the search string... useful for reverse 'in' clues, for example
    // "BEER IFFY" would print "YFFIREEB" from which the answer might be "FIRE"
    let mut results: Vec<String> = Vec::new();
    let word = search_string.chars().rev().collect::<String>();
    results.push(word);
    results
}

fn jumble(full_input: &str, found_letters: &str, size: u8) {
    if size > 0 && size as usize != full_input.len() {
        println!(
            "Error: the number of supplied letters ({}) did not match the 'size' argument",
            full_input.len()
        );
        return;
    }
    // Remove underscores from found_letters
    let mut input: String = full_input.to_string();
    for c in found_letters.chars() {
        if c != '_' && c != '/' && c != '.' {
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

    ui::display::anagram_helper(found_letters, chars, len);
}

fn define(word: &str) {
    let mut results = vec![];
    file::load::definitions(&mut results, word);
    if results.is_empty() {
        println!("No definition found.");
    } else {
        for result in results {
            println!(" * {}", result);
        }
    }
}
