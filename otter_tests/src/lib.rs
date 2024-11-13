use std::path::{Path, PathBuf};

use otter_lib::{config::Config, dispatch::report, io::files::context_from_path};

pub fn cnf_lib_subdir(dirs: Vec<&str>) -> PathBuf {
    let mut the_path = Path::new("..").join("cnf_lib");
    for dir in dirs {
        the_path = the_path.join(dir);
    }
    the_path
}

pub fn silent_formula_report(path: PathBuf, config: &Config) -> report::Solve {
    let (tx, _) = crossbeam::channel::unbounded();

    let mut context_from_path =
        context_from_path(path, config.clone(), tx).expect("Context build failure");

    assert!(context_from_path.solve().is_ok());
    context_from_path.report()
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
            let report = silent_formula_report(test.path(), config);
            assert_eq!(require, report);
            count += 1;
        }
    }
    count
}

pub fn silent_on_split_directory(collection: PathBuf, config: &Config) {
    silent_on_directory(collection.join("sat"), config, report::Solve::Satisfiable);
    silent_on_directory(
        collection.join("unsat"),
        config,
        report::Solve::Unsatisfiable,
    );
}
