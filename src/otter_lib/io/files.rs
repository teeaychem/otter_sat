#![allow(unused_imports)]
#![allow(dead_code)]

use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;

use crate::context::builder::BuildIssue;
use crate::{
    config::Config,
    context::{self, Context, Report},
    structures::{
        literal::{Literal, LiteralSource},
        variable::list::VariableList,
    },
};

pub fn context_from_path(path: PathBuf, config: &Config) -> Result<Context, BuildIssue> {
    let the_path = PathBuf::from(&path);
    let file = match File::open(&the_path) {
        Err(_) => panic!("Could not open {path:?}"), // return Err(BuildIssue::Parse(ParseIssue::NoFile)),
        Ok(f) => f,
    };
    let unique_config = config.clone();
    match &the_path.extension() {
        None => Context::from_dimacs(&the_path, BufReader::new(&file), unique_config),
        Some(extension) if *extension == "gz" => Context::from_dimacs(
            &the_path,
            BufReader::new(GzDecoder::new(&file)),
            unique_config,
        ),
        Some(_) => Context::from_dimacs(&the_path, BufReader::new(&file), unique_config),
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
            let mut context_from_path = match context_from_path(test.path(), config) {
                Ok(c) => c,
                Err(e) => panic!("Builder error {e:?}"),
            };

            let result = match context_from_path.solve() {
                Ok(report) => report,
                Err(e) => panic!("solve error {e:?}"),
            };

            assert_eq!(require, result);
        }
    }
}

pub fn default_on_split_dir(collection: PathBuf, config: &Config) {
    default_on_dir(collection.join("sat"), config, Report::Satisfiable);
    default_on_dir(collection.join("unsat"), config, Report::Unsatisfiable);
}
