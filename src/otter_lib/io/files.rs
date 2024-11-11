use std::fs::{self, File};
use std::io::BufReader;
use std::path::PathBuf;

use crossbeam::channel::Sender;
use xz2::read::XzDecoder;

use crate::context::builder::{BuildErr, ParseErr};
use crate::dispatch::{Dispatch, SolveReport};
use crate::{config::Config, context::Context};

pub fn context_from_path(
    path: PathBuf,
    config: Config,
    sender: Sender<Dispatch>,
) -> Result<Context, BuildErr> {
    let the_path = PathBuf::from(&path);
    let file = match File::open(&the_path) {
        Err(_) => return Err(BuildErr::Parse(ParseErr::NoFile)),
        Ok(f) => f,
    };
    let unique_config = config.clone();
    match &the_path.extension() {
        None => Context::from_dimacs_file(&the_path, BufReader::new(&file), unique_config, sender),
        Some(extension) if *extension == "xz" => Context::from_dimacs_file(
            &the_path,
            BufReader::new(XzDecoder::new(&file)),
            unique_config,
            sender,
        ),
        Some(_) => {
            Context::from_dimacs_file(&the_path, BufReader::new(&file), unique_config, sender)
        }
    }
}

pub fn silent_formula_report(path: PathBuf, config: &Config) -> SolveReport {
    let (tx, rx) = crossbeam::channel::unbounded();

    let mut context_from_path =
        context_from_path(path, config.clone(), tx).expect("Context build failure");

    assert!(context_from_path.solve().is_ok());
    context_from_path.report()
}

pub fn silent_on_directory(collection: PathBuf, config: &Config, require: SolveReport) -> usize {
    let dir_info = fs::read_dir(collection);

    assert!(dir_info.is_ok(), "Formulas missing");

    let mut count = 0;

    for test in dir_info.unwrap().flatten() {
        if test
            .path()
            .extension()
            .is_some_and(|extension| extension == "xz")
        {
            let report = silent_formula_report(test.path(), config);
            assert_eq!(require, report);
            count += 1;
        }
    }
    count
}

pub fn silent_on_split_directory(collection: PathBuf, config: &Config) {
    silent_on_directory(collection.join("sat"), config, SolveReport::Satisfiable);
    silent_on_directory(collection.join("unsat"), config, SolveReport::Unsatisfiable);
}
