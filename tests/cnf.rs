#![allow(unused_imports)]
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use otter_lib::{
    config::Config,
    context::{self, Context, Report},
    structures::{
        literal::{Literal, Source},
        variable::list::VariableList,
    },
};

fn cnf_path() -> PathBuf {
    Path::new(".").join("tests").join("cnf")
}

fn satlib_path(collection: &str) -> PathBuf {
    let satlib_path = cnf_path().join("SATLIB");
    satlib_path.join(Path::new(collection))
}

fn test_path(path: PathBuf, config: &Config) -> Report {
    let mut the_context =
        Context::from_dimacs(&path, config.clone()).expect("failed to build context");
    match the_context.solve() {
        Ok(report) => report,
        Err(e) => panic!("solve error {e:?}"),
    }
}

fn test_collection(collection: PathBuf, config: &Config, require: Report) {
    let formulas = fs::read_dir(collection)
        .unwrap_or_else(|_| panic!("formulas missing"))
        .flatten();

    for test in formulas {
        let result = test_path(test.path(), config);
        assert_eq!(require, result);
    }
}

fn test_split_collection(collection: PathBuf, config: &Config) {
    test_collection(collection.join("sat"), config, Report::Satisfiable);
    test_collection(collection.join("unsat"), config, Report::Unsatisfiable);
}

#[test]
fn uniform_random_3_50_128() {
    test_split_collection(satlib_path("UUF50.218.1000"), &Config::default());
}

#[test]
fn uniform_random_3_1065_100() {
    test_split_collection(satlib_path("UF250.1065.100"), &Config::default());
}

#[test]
fn planning() {
    let planning_collections = vec!["logistics", "blocksworld"];
    for collection in planning_collections {
        test_collection(
            satlib_path(collection),
            &Config::default(),
            Report::Satisfiable,
        );
    }
}

#[test]
fn ais() {
    test_collection(satlib_path("ais"), &Config::default(), Report::Satisfiable);
}

#[test]
fn bmc() {
    test_collection(satlib_path("bmc"), &Config::default(), Report::Satisfiable);
}

#[test]
fn beijing() {
    let config = Config::default();
    let collection_path = satlib_path("beijing");

    let satisfiable_formulas = vec![
        "2bitcomp_5.cnf",
        "2bitmax_6.cnf",
        "3bitadd_31.cnf",
        "3bitadd_32.cnf",
        "3blocks.cnf",
        "4blocks.cnf",
        "4blocksb.cnf",
        "e0ddr2-10-by-5-1.cnf",
        "e0ddr2-10-by-5-4.cnf",
        "enddr2-10-by-5-1.cnf",
        "enddr2-10-by-5-8.cnf",
        "ewddr2-10-by-5-1.cnf",
        "ewddr2-10-by-5-8.cnf",
    ];
    for formula in satisfiable_formulas {
        assert_eq!(
            test_path(collection_path.join(formula), &config),
            Report::Satisfiable
        );
    }
}
