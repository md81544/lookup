pub mod load {

    use std::{
        collections::{hash_map::Entry, HashMap}, fs::File, io::{self, BufRead}, path::Path
    };

    fn read_lines<P>(filename: &P) -> io::Result<io::Lines<io::BufReader<File>>>
    where
        P: AsRef<Path>,
    {
        let file = File::open(filename)?;
        Ok(io::BufReader::new(file).lines())
    }

    pub fn full_list(
        results: &mut Vec<String>,
        anagrams: &mut HashMap<String, Vec<usize>>,
        file_name: &str,
        vec_index: &mut usize
    ) {
        if let Ok(lines) = read_lines(&file_name) {
            for word in lines.map_while(Result::ok) {
                results.push(word.clone());
                let anagram = crate::sort_word(&word);
                // Does this entry already exist? Add to vec if so, else create new vec
                match anagrams.entry(anagram) {
                    Entry::Vacant(e) => {
                        e.insert(vec![*vec_index]);
                    }
                    Entry::Occupied(mut e) => {
                        e.get_mut().push(*vec_index);
                    }
                }
                *vec_index += 1;
            }
        }
    }

    pub fn thesaurus(results: &mut Vec<String>, word: &str) {
        let file_name = "./thesaurus.txt".to_string();
        if let Ok(lines) = read_lines(&file_name) {
            for line in lines.map_while(Result::ok) {
                let search_string = &(word.to_string() + ",").to_ascii_lowercase();
                if line.starts_with(search_string) {
                    let words = line.split(",");
                    let mut first: bool = true;
                    for word in words {
                        if first {
                            first = false;
                        } else {
                            results.push(word.to_string());
                        }
                    }
                }
            }
        }
    }

    pub fn definitions(results: &mut Vec<String>, word: &str) {
        let file_name = "./definitions.txt".to_string();
        if let Ok(lines) = read_lines(&file_name) {
            for line in lines.map_while(Result::ok) {
                let search_string = &(word.to_string() + "|");
                if line.starts_with(search_string) {
                    let parts = line.split("|");
                    let mut first: bool = true;
                    for part in parts {
                        if first {
                            first = false;
                        } else {
                            results.push(part.to_string());
                        }
                    }
                }
            }
        }
    }

    pub fn wordle(results: &mut Vec<String>, file_name: &str) {
        if let Ok(lines) = read_lines(&file_name) {
            for word in lines.map_while(Result::ok) {
                if word.len() == 5 {
                    results.push(word.clone());
                }
            }
        }
    }
}
