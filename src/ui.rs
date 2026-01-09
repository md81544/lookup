pub mod display {

    use crate::define;
    use crate::file::load::thesaurus;
    use crate::jumble;
    use crate::reverse;
    use crate::Action;
    use crate::OutputType;
    use std::collections::HashSet;

    use colored::Colorize;

    fn print_separator(output_type: OutputType) {
        if output_type == OutputType::Narrow {
            println!();
        } else {
            print!(" ");
        }
    }

    pub fn word_contains_all_letters(word: &str, letter_set: &str) -> bool {
        if word.len() < letter_set.len() {
            return false;
        }
        // Multiple letters the same are allowed, so for instance
        // if search_string is "EATS", a word of "TASTE" should return true
        // (this is for "spelling bee" where a letter can be used more than
        // once to construct answers)
        let mut ls = letter_set.to_string();
        let mut found_chars: HashSet<char> = HashSet::new();
        for c in word.chars() {
            if let Some(pos) = ls.find(c) {
                ls.remove(pos);
                found_chars.insert(c);
            } else {
                if !found_chars.contains(&c) {
                    return false;
                }
            }
        }
        // Finally, did we use all the letters in the letter_set?
        // For instance "ASSET" shouldn't match letter_set "TASTE"
        // because both Ts were not used
        if ls.is_empty() {
            true
        } else {
            false
        }
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
                    || (action == Action::Spellingbee
                        && word_contains_all_letters(word, search_string))
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

    pub fn anagram_helper(
        found_letters: &str,
        chars: Vec<char>,
        len: usize,
        output_type: OutputType,
    ) {
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
        if output_type == OutputType::Json {
            let mut rows: Vec<String> = Vec::new();
            for row in grid {
                rows.push(format!("  {}", row.iter().collect::<String>()));
            }
            let json_output = serde_json::to_string(&rows).unwrap();
            println!("{}", json_output);
            return;
        }
        for row in grid {
            println!("  {}", row.iter().collect::<String>());
        }
        println!();
        print!("  ");
        let mut count = 0;
        let mut letter_count = String::new();
        for c in found_letters.chars() {
            if c == '/' {
                if !letter_count.is_empty() {
                    letter_count.push(',');
                }
                letter_count.push_str(&count.to_string());
                count = 0;
                print!("  ");
            } else if c == '.' {
                print!("_ ");
                count += 1;
            } else {
                print!("{} ", c.to_ascii_uppercase());
                count += 1;
            }
        }
        if !letter_count.is_empty() {
            letter_count.push(',');
        }
        letter_count.push_str(&count.to_string());
        println!(" ({})", letter_count);
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
                Print(format!(
                    "  ({}) - esc to quit, space to reset",
                    search_string.len()
                )),
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

    fn get_key() -> char {
        use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
        use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
        let rc: char;
        let _ = enable_raw_mode();
        loop {
            if let Event::Key(KeyEvent { code, kind, .. }) = event::read().unwrap() {
                if kind == KeyEventKind::Press {
                    if code == KeyCode::Esc {
                        rc = 'q';
                        break;
                    }
                    let c = code.as_char();
                    if c.is_some() {
                        let c = code.as_char().unwrap().to_ascii_uppercase();
                        rc = c;
                        break;
                    }
                }
            }
        }
        let _ = disable_raw_mode();
        rc
    }

    fn input_string(prompt: &str, default: Option<&str>) -> String {
        use rustyline::error::ReadlineError;
        use std::sync::{Mutex, OnceLock};
        // rustyline::DefaultEditor is implemented as a static here to enable
        // history:
        static RL: OnceLock<Mutex<rustyline::DefaultEditor>> = OnceLock::new();
        RL.get_or_init(|| Mutex::new(rustyline::DefaultEditor::new().unwrap()));
        let mut rl = RL
            .get_or_init(|| Mutex::new(rustyline::DefaultEditor::new().unwrap()))
            .lock()
            .unwrap();
        let mut rc = "".to_string();
        let readline;
        if default.is_none() {
            readline = rl.readline(prompt);
        } else {
            readline = rl.readline_with_initial(prompt, (default.unwrap(), ""));
        }
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str()).unwrap();
                rc = line.to_uppercase();
                return rc;
            }
            Err(ReadlineError::Interrupted) => {
                return rc;
            }
            Err(ReadlineError::Eof) => {
                return rc;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                return rc;
            }
        }
    }

    pub fn tui() -> Result<(), rustyline::error::ReadlineError> {
        use crate::expand_found_string;
        'outer: loop {
            let search_string = input_string("Enter search string: ", Some(""));
            if search_string.len() == 0 {
                break;
            }
            let mut found_string = "".to_string();
            let mut comment = "".to_string();
            'restart: loop {
                println!();
                println!("Search string: {} ({})", search_string, search_string.len());
                if found_string.len() == 0 {
                    found_string = expand_found_string(&search_string, ".");
                }
                if found_string.len() > 0 {
                    print!("Found letters: ");
                    for c in found_string.chars() {
                        if c == '.' {
                            print!("_ ");
                        } else {
                            print!("{} ", c);
                        }
                    }
                    println!();
                }
                if comment.len() > 0 {
                    println!("Comment: {}", comment);
                }
                let mut more_printed = false;
                println!(
                    "\nMenu: {}umble {}ound {}emove {}omment re{}tart {}ore {}uit",
                    "J".yellow(),
                    "F".yellow(),
                    "R".yellow(),
                    "C".yellow(),
                    "S".yellow(),
                    "M".yellow(),
                    "Q".yellow()
                );
                loop {
                    match get_key() {
                        'J' => {
                            let mut letters = ".".to_string();
                            if found_string.len() > 0 {
                                letters = found_string.clone();
                            } else {
                                letters = expand_found_string(&search_string, &letters);
                            }
                            jumble(
                                &search_string,
                                &letters,
                                search_string.len() as u8,
                                OutputType::Normal,
                            );
                            break;
                        }
                        'F' => {
                            found_string =
                                input_string("Enter found letters: ", Some(&found_string));
                            found_string = expand_found_string(&search_string, &found_string);
                            let letters_no_spaces: String = found_string.replace("/", "");
                            if !letters_no_spaces.is_empty()
                                && letters_no_spaces.len() > search_string.len()
                            {
                                print!("{}", "ERROR: ".bold());
                                println!("'found' letters must be same length as search string");
                                found_string.clear();
                            }
                            break;
                        }
                        'R' => {
                            interactive_remove(search_string.clone());
                            break;
                        }
                        'C' => {
                            comment = input_string("Enter comment: ", Some(""));
                            break;
                        }
                        'Q' => {
                            break 'outer;
                        }
                        'S' => {
                            println!("\n________\n");
                            break 'restart;
                        }
                        'M' => {
                            if !more_printed {
                                println!(
                                    "      {}hesaurus {}nagram {}ookup {}efine re{}erse re{}ular",
                                    "T".yellow(),
                                    "A".yellow(),
                                    "L".yellow(),
                                    "D".yellow(),
                                    "V".yellow(),
                                    "G".yellow(),
                                );
                                more_printed = true;
                            }
                        }
                        'T' => {
                            println!("Thesaurus: {}", search_string.white().bold());
                            let mut results: Vec<String> = Vec::new();
                            thesaurus(&mut results, &search_string);
                            let mut first = true;
                            for s in results {
                                if !first {
                                    print!(", ");
                                }
                                print!("{}", s);
                                first = false;
                            }
                            println!();
                            break;
                        }
                        'A' => {
                            println!("Anagram: {}", "TODO".white().bold());
                            break;
                        }
                        'L' => {
                            println!("Lookup: {}", "TODO".white().bold());
                            break;
                        }
                        'D' => {
                            println!("Define: {}", search_string.white().bold());
                            define(&search_string, OutputType::Normal);
                            break;
                        }
                        'V' => {
                            let results = reverse(&search_string);
                            for s in results {
                                println!("{} reversed = {}", search_string, s.yellow().bold());
                            }
                            break;
                        }
                        'G' => {
                            println!("Regular: {}", "TODO".white().bold());
                            break;
                        }
                        _ => {
                            // do nothing
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
