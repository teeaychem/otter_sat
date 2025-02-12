use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use otter_sat::{config::Config, context::Context, dispatch::SolveReport, types::err};
use xz2::read::XzDecoder;

pub fn load_dimacs(context: &mut Context, path: &PathBuf) -> Result<(), err::ErrorKind> {
    let file = match File::open(path) {
        Err(_) => panic!("Could not load {path:?}"),
        Ok(f) => f,
    };

    match &path.extension() {
        None => {
            context.read_dimacs(BufReader::new(&file))?;
        }
        Some(extension) if *extension == "xz" => {
            context.read_dimacs(BufReader::new(XzDecoder::new(&file)))?;
        }
        Some(_) => {
            context.read_dimacs(BufReader::new(&file))?;
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

pub fn silent_formula_report(path: PathBuf, config: &Config) -> SolveReport {
    let mut the_context = Context::from_config(config.clone());
    match load_dimacs(&mut the_context, &path) {
        Ok(()) => {}
        Err(err::ErrorKind::ClauseDB(err::ClauseDBError::EmptyClause)) => {
            return SolveReport::Unsatisfiable;
        }
        Err(_) => {
            panic!("c Error loading file.")
        }
    };

    match the_context.solve() {
        Ok(_) => {}
        Err(e) => panic!("{e:?}"),
    }

    the_context.report()
}

pub fn silent_on_directory(subdir: PathBuf, config: &Config, require: SolveReport) -> usize {
    let dir_info = std::fs::read_dir(subdir);

    let mut count = 0;

    match dir_info {
        Err(_) => panic!("Formulas missing"),
        Ok(the_gen) => {
            for test in the_gen.flatten() {
                if test
                    .path()
                    .extension()
                    .is_some_and(|extension| extension == "xz")
                {
                    assert_eq!(require, silent_formula_report(test.path(), config));
                    count += 1;
                }
            }
        }
    }

    count
}
