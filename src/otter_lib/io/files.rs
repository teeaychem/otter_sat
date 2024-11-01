#![allow(unused_imports)]
#![allow(dead_code)]

use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;

use crate::{
    config::Config,
    context::{self, Context, Report},
    structures::{
        literal::{Literal, LiteralSource},
        variable::list::VariableList,
    },
};

pub fn default_on_path(path: PathBuf, config: &Config) -> Report {
    let the_path = PathBuf::from(&path);
    let file = match File::open(&the_path) {
        Err(_) => panic!("o"), // return Err(BuildIssue::Parse(ParseIssue::NoFile)),
        Ok(f) => f,
    };
    let unique_config = config.clone();
    let parsed_context = match &the_path.extension() {
        None => Context::from_dimacs(&the_path, BufReader::new(&file), unique_config),
        Some(extension) if *extension == "gz" => Context::from_dimacs(
            &the_path,
            BufReader::new(GzDecoder::new(&file)),
            unique_config,
        ),
        Some(_) => Context::from_dimacs(&the_path, BufReader::new(&file), unique_config),
    };

    let mut the_context = match parsed_context {
        Ok(c) => c,
        Err(e) => {
            println!("Issue with {path:?}");
            panic!("{e:?}")
        }
    };
    match the_context.solve() {
        Ok(report) => report,
        Err(e) => panic!("solve error {e:?}"),
    }
}

pub fn default_on_dir(collection: PathBuf, config: &Config, require: Report) {
    let formulas = fs::read_dir(collection)
        .unwrap_or_else(|_| panic!("formulas missing"))
        .flatten();

    for test in formulas {
        if test
            .path()
            .extension()
            .is_some_and(|ext| ext == "cnf" || ext == "gz")
        {
            let result = default_on_path(test.path(), config);
            assert_eq!(require, result);
        }
    }
}

pub fn default_on_split_dir(collection: PathBuf, config: &Config) {
    default_on_dir(collection.join("sat"), config, Report::Satisfiable);
    default_on_dir(collection.join("unsat"), config, Report::Unsatisfiable);
}
