pub mod display {

    use crate::Action;
    use crate::OutputType;

    use colored::Colorize;

    fn print_separator(output_type: OutputType) {
        if output_type == OutputType::Narrow {
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
        output_type: OutputType,
    ) {
        if output_type == OutputType::Json {
            let json_output = serde_json::to_string(&results).unwrap();
            println!("{}", json_output);
        } else {
            for word in results {
                if word.contains(char::is_whitespace) && output_type != OutputType::Narrow {
                    print!("'");
                }
                if (action == Action::Panagram && word.len() == 9)
                    || (action == Action::Spellingbee && word_is_anagram(word, search_string))
                {
                    print!("{}", word.to_uppercase().bold());
                } else {
                    print!("{}", word);
                }
                if word.contains(char::is_whitespace) && output_type != OutputType::Narrow {
                    print!("'");
                }
                print_separator(output_type);
            }
            println!();
        }
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

    pub fn interactive_remove(search_string: String) {
        use crossterm::{
            cursor::{MoveToColumn, RestorePosition, SavePosition},
            event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
            execute,
            style::Print,
            terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
        };
        use std::io::stdout;
        let mut s = search_string.to_uppercase().clone();
        let mut removed = "".to_string();
        enable_raw_mode().unwrap();
        let mut stdout = stdout();
        println!();
        loop {
            if s.is_empty() {
                break;
            }
            execute!(
                stdout,
                MoveToColumn(0),
                Clear(ClearType::CurrentLine),
                Print(format!("{} ", s)),
                SavePosition
            )
            .unwrap();
            if !removed.is_empty() {
                execute!(
                    stdout,
                    SavePosition,
                    Print(format!(" removed: {}  ", removed))
                )
                .unwrap();
            }
            execute!(
                stdout,
                Print("  (Press esc to quit, space to reset)".to_string()),
                RestorePosition
            )
            .unwrap();

            if let Event::Key(KeyEvent { code, kind, .. }) = event::read().unwrap() {
                if kind == KeyEventKind::Press {
                    if code == KeyCode::Esc {
                        execute!(stdout, MoveToColumn(0), Clear(ClearType::CurrentLine)).unwrap();
                        break;
                    }
                    let c = code.as_char();
                    if c.is_some() {
                        if code.as_char().unwrap() == ' ' {
                            // Spacebar resets word
                            s = search_string.to_uppercase().clone();
                            removed = "".to_string();
                            continue;
                        }
                        let c = code.as_char().unwrap().to_ascii_uppercase();
                        if let Some(pos) = s.find(c) {
                            s.remove(pos);
                            removed.push(c);
                        } else {
                            // Print the bell (beep)
                            print!("{}", 0x07 as char);
                        }
                    }
                }
            }
        }
        execute!(stdout, MoveToColumn(0), Clear(ClearType::CurrentLine)).unwrap();
        disable_raw_mode().unwrap();
        if !removed.is_empty() {
            println!("{}", removed);
        }
        println!();
    }
}
