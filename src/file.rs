pub mod load {

    use std::{
        fs::File,
        io::{self, BufRead},
        path::Path,
    };

    fn read_lines<P>(filename: &P) -> io::Result<io::Lines<io::BufReader<File>>>
    where
        P: AsRef<Path>,
    {
        let file = File::open(filename)?;
        Ok(io::BufReader::new(file).lines())
    }

    pub fn thesaurus(results: &mut Vec<String>, word: &str) {
        let file_name = "./thesaurus.txt".to_string();
        if let Ok(lines) = read_lines(&file_name) {
            for line in lines.map_while(Result::ok) {
                let search_string = &(word.to_string() + ",");
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
}
