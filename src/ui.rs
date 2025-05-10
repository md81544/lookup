pub mod display {

    use crate::Action;

    use colored::Colorize;

    fn print_separator(narrow: bool) {
        if narrow {
            println!();
        } else {
            print!(" ");
        }
    }

    fn word_is_anagram(word: &str, search_string: &str) -> bool {
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

    pub fn show_results(
        results: &Vec<String>,
        search_string: &str,
        action: crate::Action,
        narrow: bool,
    ) {
        for word in results {
            if word.contains(char::is_whitespace) && !narrow {
                print!("'");
            }
            if (action == Action::Panagram && word.len() == 9)
                || (action == Action::Spellingbee && word_is_anagram(word, search_string))
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
}
