pub mod display {

    use crate::anagram_search;
    use crate::define;
    use crate::file;
    use crate::file::load::thesaurus;
    use crate::jumble;
    use crate::lookup;
    use crate::regular_patterns;
    use crate::remove_found_mismatches;
    use crate::reverse;
    use crate::Action;
    use crate::OutputType;
    use std::collections::HashMap;
    use std::collections::HashSet;

    use colored::Colorize;
    use rustyline::config::Configurer;

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
            } else if !found_chars.contains(&c) {
                return false;
            }
        }
        // Finally, did we use all the letters in the letter_set?
        // For instance "ASSET" shouldn't match letter_set "TASTE"
        // because both Ts were not used
        ls.is_empty()
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
        rl.set_edit_mode(rustyline::EditMode::Vi);
        let mut rc = "".to_string();
        let readline;
        if default.is_none() {
            readline = rl.readline(prompt);
        } else {
            readline = rl.readline_with_initial(prompt, ("", default.unwrap()));
        }
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str()).unwrap();
                rc = line.to_uppercase();
                rc
            }
            Err(ReadlineError::Interrupted) => rc,
            Err(ReadlineError::Eof) => rc,
            Err(err) => {
                println!("Error: {:?}", err);
                rc
            }
        }
    }

    pub fn tui() -> Result<(), rustyline::error::ReadlineError> {
        use crate::expand_found_string;
        use std::io::{self, Write};

        struct Datum {
            search_string: String,
            found_string: String,
            comment: String,
            clue: String,
        }
        let mut data: HashMap<String, Datum> = HashMap::new();
        println!();
        'outer: loop {
            let mut clue: String = "".to_string();
            while clue.is_empty() {
                clue = input_string("Enter clue number (e.g. 4A or 7D): ", None);
                if clue.is_empty() {
                    println!("Clue number is required");
                } else {
                    break;
                }
            }
            let mut search_string = input_string("Enter search string: ", None);
            if search_string.is_empty() {
                break;
            }
            let mut found_string = "".to_string();
            let mut comment = "".to_string();
            if search_string.contains('.')
                || search_string.contains('_')
                || search_string.contains('*')
            {
                found_string = search_string.clone();
                search_string.clear();
            }
            'restart: loop {
                println!();
                println!("Clue no:       {}", &clue);
                if !search_string.is_empty() {
                    println!("Search string: {} ({})", search_string, search_string.len());
                }
                if found_string.is_empty() {
                    found_string = expand_found_string(&search_string, ".");
                }
                if !found_string.is_empty() {
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
                if !comment.is_empty() {
                    println!("Comment:       {}", comment);
                }
                println!(
                    "\nMenu: {}umble {}ound {}emove {}omment re{}erse re{}ular {}hesaurus",
                    "J".yellow(),
                    "F".yellow(),
                    "R".yellow(),
                    "C".yellow(),
                    "V".yellow(),
                    "G".yellow(),
                    "T".yellow()
                );
                println!(
                    "      {}nagram {}ookup {}efine {}ote st{}re r{}trieve re{}tart {}uit",
                    "A".yellow(),
                    "L".yellow(),
                    "D".yellow(),
                    "N".yellow(),
                    "O".yellow(),
                    "E".yellow(),
                    "S".yellow(),
                    "Q".yellow(),
                );
                io::stdout().flush()?;
                loop {
                    print!(">");
                    io::stdout().flush()?;
                    let k = get_key();
                    print!("\x08 \x08"); // backspace
                    io::stdout().flush()?;
                    match k {
                        'J' => {
                            println!();
                            let mut letters = ".".to_string();
                            if !found_string.is_empty() {
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
                            println!();
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
                            println!();
                            interactive_remove(search_string.clone());
                            break;
                        }
                        'C' => {
                            println!();
                            comment = input_string("Enter comment: ", Some(""));
                            break;
                        }
                        'Q' => {
                            println!();
                            break 'outer;
                        }
                        'S' => {
                            println!("\n\n________\n");
                            break 'restart;
                        }
                        'T' => {
                            println!("\nThesaurus: {}", search_string.white().bold());
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
                            println!("\nAnagram: {}", search_string.white().bold());
                            let mut anagrams: HashMap<String, Vec<usize>> = HashMap::new();
                            let mut word_list: Vec<String> = Vec::new();
                            let mut vec_index: usize = 0usize;
                            file::load::full_list(
                                &mut word_list,
                                &mut anagrams,
                                "./words_3.txt",
                                &mut vec_index,
                            );
                            file::load::full_list(
                                &mut word_list,
                                &mut anagrams,
                                "./phrases.txt",
                                &mut vec_index,
                            );
                            let results = anagram_search(
                                &search_string.to_ascii_lowercase(),
                                &word_list,
                                &anagrams,
                            );
                            for s in results {
                                println!("* {}", s.yellow());
                            }
                            break;
                        }
                        'L' => {
                            println!("\nLookup: {}", search_string.white().bold());
                            let mut anagrams: HashMap<String, Vec<usize>> = HashMap::new();
                            let mut word_list: Vec<String> = Vec::new();
                            let file_name = "./words_3.txt";
                            let mut vec_index: usize = 0usize;
                            file::load::full_list(
                                &mut word_list,
                                &mut anagrams,
                                file_name,
                                &mut vec_index,
                            );
                            file::load::full_list(
                                &mut word_list,
                                &mut anagrams,
                                "./phrases.txt",
                                &mut vec_index,
                            );
                            let mut results;
                            if !search_string.is_empty()
                                && !search_string.contains('.')
                                && !search_string.contains('_')
                                && !search_string.contains('%')
                            {
                                // If we have a non-wildcarded search string, we can do the
                                // lookup by an anagram search followed by remove_found_mismatches()
                                results = anagram_search(
                                    &search_string.to_ascii_lowercase(),
                                    &word_list,
                                    &anagrams,
                                );
                                results =
                                    remove_found_mismatches(&results, found_string.clone(), false);
                            } else {
                                results =
                                    lookup(&found_string.to_ascii_lowercase(), &word_list, "");
                            }
                            for s in results {
                                println!("* {}", s.yellow());
                            }
                            break;
                        }
                        'D' => {
                            println!("\nDefine: {}", search_string.white().bold());
                            define(&search_string, OutputType::Normal);
                            break;
                        }
                        'V' => {
                            println!();
                            let results = reverse(&search_string);
                            for s in results {
                                println!("{} reversed = {}", search_string, s.yellow().bold());
                            }
                            break;
                        }
                        'G' => {
                            println!("\nRegular: {}", search_string.white().bold());
                            let results1 = regular_patterns(&search_string, false);
                            let results2 = regular_patterns(&search_string, true);
                            print!("Normal:   ");
                            for s in results1 {
                                print!("{}  ", s);
                            }
                            println!();
                            print!("Reversed: ");
                            for s in results2 {
                                print!("{}  ", s);
                            }
                            println!();
                            break;
                        }
                        'N' => {
                            println!("\nNote: {}", "TODO".white().bold());
                            break;
                        }
                        'O' => {
                            println!();
                            // stOre
                            let d = Datum {
                                comment: comment.clone(),
                                found_string: found_string.clone(),
                                search_string: search_string.clone(),
                                clue: clue.clone(),
                            };
                            data.insert(clue.clone(), d);
                            break;
                        }
                        'E' => {
                            println!();
                            let mut sorted_pairs: Vec<(&String, &Datum)> = data.iter().collect();
                            sorted_pairs.sort_by(|a, b| a.0.cmp(b.0));
                            let mut first: bool = true;
                            print!("Stored clues: ");
                            for (key, _) in sorted_pairs {
                                if !first {
                                    print!(", ");
                                }
                                print!("{}", key);
                                first = false;
                            }
                            println!();
                            let key =
                                input_string("Retrieve: Enter clue number: ", None);
                            if key.is_empty() {
                                break;
                            }
                            if let Some(d) = data.get(&key) {
                                comment = d.comment.clone();
                                found_string = d.found_string.clone();
                                search_string = d.search_string.clone();
                                clue = d.clue.clone();
                            } else {
                                println!("{}", "Clue not found".red());
                            }
                            break;
                        }
                        _ => {
                            // do nothing
                        }
                    }
                }
            }
        }
        println!();
        Ok(())
    }
}
