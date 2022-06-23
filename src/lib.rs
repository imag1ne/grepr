use clap::{arg, command};
use colored::Colorize;
use regex::{Captures, Regex, RegexBuilder};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::{fs, io};
use walkdir::WalkDir;

type MyResult<T> = Result<T, Box<dyn Error>>;

type FmtStrategy = fn(&Regex, &str) -> String;

#[derive(Debug)]
pub struct Config {
    pattern: Regex,
    files: Vec<String>,
    recursive: bool,
    count: bool,
    invert_match: bool,
}

pub fn get_args() -> MyResult<Config> {
    let matches = command!()
        .args(&[
            arg!(pattern: <PATTERN> "Search pattern"),
            arg!(files: <FILE> "Input file(s)")
                .required(false)
                .multiple_values(true)
                .default_value("-"),
            arg!(recursive: -r --recursive "Recursive search"),
            arg!(count: -c --count "Count occurrences"),
            arg!(invert_match: -v --"invert-match" "Invert match"),
            arg!(insensitive: -i --insensitive "Case-insensitive"),
        ])
        .get_matches();

    let pattern = matches
        .value_of("pattern")
        .map(|pattern| {
            let mut regex_builder = RegexBuilder::new(pattern);
            regex_builder.case_insensitive(matches.is_present("insensitive"));
            regex_builder
                .build()
                .map_err(|_| format!("Invalid pattern \"{}\"", pattern))
        })
        .transpose()?
        .unwrap();

    Ok(Config {
        pattern,
        files: matches.values_of_t_or_exit("files"),
        recursive: matches.is_present("recursive"),
        count: matches.is_present("count"),
        invert_match: matches.is_present("invert_match"),
    })
}

fn find_files(paths: &[String], recursive: bool) -> Vec<MyResult<String>> {
    let mut res = Vec::new();

    for path in paths {
        match path.as_str() {
            "-" => res.push(Ok(path.to_string())),
            _ => match fs::metadata(path) {
                Ok(metadata) => {
                    if metadata.is_dir() {
                        if recursive {
                            res.extend(WalkDir::new(path).into_iter().filter_map(|dir_entry| {
                                match dir_entry {
                                    Ok(entry) if entry.file_type().is_dir() => None,
                                    Ok(entry) => Some(Ok(entry.path().display().to_string())),
                                    Err(err) => Some(Err(err.into())),
                                }
                            }))
                        } else {
                            res.push(Err(format!("{} is a directory", path).into()));
                        }
                    } else if metadata.is_file() {
                        res.push(Ok(path.to_string()))
                    }
                }
                Err(err) => res.push(Err(format!("{}: {}", path, err).into())),
            },
        }
    }

    res
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

#[derive(Debug)]
struct MatchedLine {
    line: usize,
    content: String,
}

impl MatchedLine {
    pub fn new(line: usize, content: String) -> Self {
        Self { line, content }
    }
}

impl Display for MatchedLine {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:>6}:{}", self.line.to_string().cyan(), self.content)
    }
}

fn find_lines_and_fmt_with<T: BufRead>(
    mut file: T,
    pattern: &Regex,
    invert_match: bool,
    fmt_strategy: FmtStrategy,
) -> MyResult<Vec<MatchedLine>> {
    let mut res = Vec::new();
    let mut line = String::new();
    let mut line_num: usize = 0;
    while file.read_line(&mut line)? > 0 {
        line_num += 1;

        if pattern.is_match(&line) ^ invert_match {
            res.push(MatchedLine::new(line_num, fmt_strategy(pattern, &line)))
        }

        line.clear();
    }

    Ok(res)
}

#[allow(dead_code)]
fn find_lines_with_default_strategy<T: BufRead>(
    file: T,
    pattern: &Regex,
    invert_match: bool,
) -> MyResult<Vec<MatchedLine>> {
    find_lines_and_fmt_with(file, pattern, invert_match, default_fmt_strategy)
}

fn find_lines_with_highlight_all_matches_red<T: BufRead>(
    file: T,
    pattern: &Regex,
    invert_match: bool,
) -> MyResult<Vec<MatchedLine>> {
    find_lines_and_fmt_with(file, pattern, invert_match, highlight_all_matches_red)
}

fn highlight_all_matches_red(pattern: &Regex, line: &str) -> String {
    pattern
        .replace_all(line, |caps: &Captures| format!("{}", &caps[0].red()))
        .to_string()
}

#[allow(dead_code)]
fn default_fmt_strategy(_pattern: &Regex, line: &str) -> String {
    line.to_string()
}

pub fn run(config: Config) -> MyResult<()> {
    let entries = find_files(&config.files, config.recursive);

    for entry in &entries {
        match entry {
            Err(e) => eprintln!("{}", e),
            Ok(filename) => match open(filename) {
                Err(err) => eprintln!("{}: {}", filename, err),
                Ok(file) => {
                    let matches = find_lines_with_highlight_all_matches_red(
                        file,
                        &config.pattern,
                        config.invert_match,
                    )?;

                    let filename = filename.green();
                    let matches_num = matches.len();

                    if entries.len() > 1 {
                        if config.count {
                            println!("{}:{}", filename, matches_num);
                        } else {
                            if matches_num > 0 {
                                println!("{}", filename);
                            }

                            for line in matches {
                                print!("{}", line);
                            }
                        }
                    } else {
                        if config.count {
                            println!("{}", matches_num);
                        } else {
                            for line in matches {
                                print!("{}", line);
                            }
                        }
                    }
                }
            },
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::find_files;
    use crate::find_lines_with_default_strategy;
    use rand::{distributions::Alphanumeric, Rng};
    use regex::{Regex, RegexBuilder};
    use std::io::Cursor;

    #[test]
    fn test_find_files() {
        // Verify that the function finds a file known to exist
        let files = find_files(&["./tests/inputs/fox.txt".to_string()], false);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].as_ref().unwrap(), "./tests/inputs/fox.txt");

        // The function should reject a directory without the recursive option
        let files = find_files(&["./tests/inputs".to_string()], false);
        assert_eq!(files.len(), 1);
        if let Err(e) = &files[0] {
            assert_eq!(e.to_string(), "./tests/inputs is a directory");
        }

        // Verify the function finds four files in the directory recursively
        let res = find_files(&["./tests/inputs".to_string()], true);
        let mut files: Vec<String> = res
            .iter()
            .map(|r| r.as_ref().unwrap().replace("\\", "/"))
            .collect();
        files.sort();
        assert_eq!(files.len(), 4);
        assert_eq!(
            files,
            vec![
                "./tests/inputs/bustle.txt",
                "./tests/inputs/empty.txt",
                "./tests/inputs/fox.txt",
                "./tests/inputs/nobody.txt",
            ]
        );

        // Generate a random string to represent a nonexistent file
        let bad: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();

        // Verify that the function returns the bad file as an error
        let files = find_files(&[bad], false);
        assert_eq!(files.len(), 1);
        assert!(files[0].is_err());
    }

    #[test]
    fn test_find_lines() {
        let text = b"Lorem\nIpsum\r\nDOLOR";
        // The pattern _or_ should match the one line, "Lorem"
        let re1 = Regex::new("or").unwrap();
        let matches = find_lines_with_default_strategy(Cursor::new(&text), &re1, false);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 1);
        // When inverted, the function should match the other two lines
        let matches = find_lines_with_default_strategy(Cursor::new(&text), &re1, true);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 2);
        // This regex will be case-insensitive
        let re2 = RegexBuilder::new("or")
            .case_insensitive(true)
            .build()
            .unwrap();
        // The two lines "Lorem" and "DOLOR" should match
        let matches = find_lines_with_default_strategy(Cursor::new(&text), &re2, false);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 2);
        // When inverted, the one remaining line should match
        let matches = find_lines_with_default_strategy(Cursor::new(&text), &re2, true);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 1);
    }
}
