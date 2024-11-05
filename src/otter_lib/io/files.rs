#![allow(unused_imports)]
#![allow(dead_code)]

use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;

use crate::context::builder::{BuildIssue, ParseIssue};
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
        Err(_) => return Err(BuildIssue::Parse(ParseIssue::NoFile)),
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

pub fn formula_report(path: PathBuf, config: &Config) -> Report {
    let mut context_from_path = context_from_path(path, config).expect("Context build failure");

    assert!(context_from_path.solve().is_ok());
    context_from_path.report()
}

pub fn default_on_dir(collection: PathBuf, config: &Config, require: Report) {
    let dir_info = fs::read_dir(collection);

    assert!(dir_info.is_ok(), "Formulas missing");

    for test in dir_info.unwrap().flatten() {
        if test
            .path()
            .extension()
            .is_some_and(|ext| ext == "cnf" || ext == "gz")
        {
            let result = formula_report(test.path(), config);

            if require != result {
                println!("issue with formula:\n{:?}", test);
            }

            assert_eq!(require, result);
        }
    }
}

pub fn default_on_split_dir(collection: PathBuf, config: &Config) {
    default_on_dir(collection.join("sat"), config, Report::Satisfiable);
    default_on_dir(collection.join("unsat"), config, Report::Unsatisfiable);
}
