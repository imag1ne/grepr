use std::fs::File;
use std::io::{BufReader, Read};

use crate::error::GreprError;
use crate::matcher::Matcher;
use clap::ArgMatches;
use glob::glob;
use std::path::PathBuf;

pub struct GreprArgs {
    pub pattern: String,
    pub filenames: Option<Vec<PathBuf>>,
}

pub struct GreprApp {
    pub args: GreprArgs,
}

impl TryFrom<ArgMatches<'_>> for GreprArgs {
    type Error = GreprError;

    fn try_from(matches: ArgMatches<'_>) -> Result<Self, Self::Error> {
        let pattern = matches.value_of("PATTERN").unwrap().to_string();
        let fns = matches
            .values_of("INPUT")
            .map(|values| values.collect::<Vec<_>>());

        let filenames = if let Some(fns) = fns {
            let filenames = match_filenames(fns)?;

            if filenames.is_empty() {
                None
            } else {
                Some(filenames)
            }
        } else {
            None
        };

        Ok(Self { pattern, filenames })
    }
}

impl GreprApp {
    pub fn new() -> Self {
        let arg_matches = clap_app!(Grepr =>
            (version: "1.0")
            (author: "Dong")
            (about: "Simplified grep")
            (@arg PATTERN: +required "Sets the pattern to match(string or regex)")
            (@arg INPUT: ... "Sets the input files to use(wildcard supported)")
        )
        .get_matches();

        let args = match GreprArgs::try_from(arg_matches) {
            Ok(args) => args,
            Err(e) => {
                eprintln!("{:?}", e);
                panic!()
            }
        };

        Self { args }
    }

    pub fn grep(&self) -> Result<(), GreprError> {
        if let Some(filenames) = &self.args.filenames {
            for filename in filenames {
                let file = File::open(&filename)?;
                let mut reader = BufReader::new(file);

                let mut f_cont = String::new();
                reader.read_to_string(&mut f_cont)?;

                if let Some(ml) = f_cont.match_pattern(&self.args.pattern) {
                    println!(
                        "- {}",
                        filename
                            .file_name()
                            .expect("Path terminates in ..")
                            .to_str()
                            .expect("Path contains invalid Unicode.")
                    );
                    ml.iter().for_each(|r| println!("    {}", r));
                }
            }
        }

        Ok(())
    }
}

fn match_filenames(filenames: Vec<&str>) -> Result<Vec<PathBuf>, GreprError> {
    let mut fnames = Vec::new();

    for filename in filenames {
        fnames.extend(match_filename(filename)?);
    }

    Ok(fnames)
}

fn match_filename(filename: &str) -> Result<Vec<PathBuf>, GreprError> {
    let mut filenames = Vec::new();
    for entry in glob(filename)? {
        filenames.push(entry?);
    }

    Ok(filenames)
}
