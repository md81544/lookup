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

    pub fn anagram_helper(found_letters: &str, chars: Vec<char>, len: usize) {
        use std::f32::consts::PI;
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
            } else if c == '.' {
                print!("_ ");
            } else {
                print!("{} ", c.to_ascii_uppercase());
            }
        }
        println!();
    }
}
