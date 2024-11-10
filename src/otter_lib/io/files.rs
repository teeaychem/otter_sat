use std::fs::{self, File};
use std::io::BufReader;
use std::path::PathBuf;

use xz2::read::XzDecoder;

use crate::context::builder::{BuildErr, ParseErr};
use crate::{config::Config, context::Context, types::gen::Report};

pub fn context_from_path(path: PathBuf, config: Config) -> Result<Context, BuildErr> {
    let the_path = PathBuf::from(&path);
    let file = match File::open(&the_path) {
        Err(_) => return Err(BuildErr::Parse(ParseErr::NoFile)),
        Ok(f) => f,
    };
    let unique_config = config.clone();
    match &the_path.extension() {
        None => Context::from_dimacs_file(&the_path, BufReader::new(&file), unique_config),
        Some(extension) if *extension == "xz" => Context::from_dimacs_file(
            &the_path,
            BufReader::new(XzDecoder::new(&file)),
            unique_config,
        ),
        Some(_) => Context::from_dimacs_file(&the_path, BufReader::new(&file), unique_config),
    }
}

pub fn formula_report(path: PathBuf, config: Config) -> Report {
    let mut context_from_path = context_from_path(path, config).expect("Context build failure");

    assert!(context_from_path.solve().is_ok());
    context_from_path.report()
}

pub fn default_on_dir(collection: PathBuf, config: Config, require: Report) -> usize {
    let dir_info = fs::read_dir(collection);

    assert!(dir_info.is_ok(), "Formulas missing");

    let mut count = 0;

    for test in dir_info.unwrap().flatten() {
        if test
            .path()
            .extension()
            .is_some_and(|extension| extension == "xz")
        {
            let report = formula_report(test.path(), config.clone());
            assert_eq!(require, report);
            count += 1;
        }
    }
    count
}

pub fn default_on_split_dir(collection: PathBuf, config: Config) {
    default_on_dir(collection.join("sat"), config.clone(), Report::Satisfiable);
    default_on_dir(collection.join("unsat"), config, Report::Unsatisfiable);
}
