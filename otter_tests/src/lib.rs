use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use otter_lib::{config::Config, context::Context, dispatch::report, types::err};
use xz2::read::XzDecoder;

pub fn load_dimacs(context: &mut Context, path: &PathBuf) -> Result<(), err::Build> {
    let file = match File::open(path) {
        Err(_) => panic!("Could not load {path:?}"),
        Ok(f) => f,
    };

    match &path.extension() {
        None => {
            context.load_dimacs_file(BufReader::new(&file))?;
        }
        Some(extension) if *extension == "xz" => {
            context.load_dimacs_file(BufReader::new(XzDecoder::new(&file)))?;
        }
        Some(_) => {
            context.load_dimacs_file(BufReader::new(&file))?;
        }
    };
    Ok(())
}

pub fn cnf_lib_subdir(dirs: Vec<&str>) -> PathBuf {
    let mut the_path = Path::new("..").join("cnf_lib");
    for dir in dirs {
        the_path = the_path.join(dir);
    }
    the_path
}

pub fn silent_formula_report(path: PathBuf, config: &Config) -> report::Solve {
    let (tx, _) = crossbeam::channel::bounded(0);

    let mut the_context = Context::from_config(config.clone(), tx.clone());
    match load_dimacs(&mut the_context, &path) {
        Ok(()) => {}
        Err(err::Build::ClauseStore(err::ClauseDB::EmptyClause)) => {
            return report::Solve::Unsatisfiable;
        }
        Err(e) => {
            panic!("c Error loading file: {e:?}")
        }
    };

    assert!(the_context.solve().is_ok());
    the_context.report()
}

pub fn silent_on_directory(subdir: PathBuf, config: &Config, require: report::Solve) -> usize {
    let dir_info = std::fs::read_dir(subdir);

    assert!(dir_info.is_ok(), "Formulas missing");

    let mut count = 0;

    for test in dir_info.unwrap().flatten() {
        if test
            .path()
            .extension()
            .is_some_and(|extension| extension == "xz")
        {
            assert_eq!(require, silent_formula_report(test.path(), config));
            count += 1;
        }
    }
    count
}