#![allow(unused_imports)]
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use crate::{
    config::Config,
    context::{self, Context, Report},
    structures::{
        literal::{Literal, LiteralSource},
        variable::list::VariableList,
    },
};

pub fn default_on_path(path: PathBuf, config: &Config) -> Report {
    let mut the_context = match Context::from_dimacs(&path, config.clone()) {
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
        if test.path().extension().is_some_and(|ext| ext == "cnf") {
            let result = default_on_path(test.path(), config);
            assert_eq!(require, result);
        }
    }
}

pub fn default_on_split_dir(collection: PathBuf, config: &Config) {
    default_on_dir(collection.join("sat"), config, Report::Satisfiable);
    default_on_dir(collection.join("unsat"), config, Report::Unsatisfiable);
}
