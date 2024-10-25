#![allow(unused_imports)]
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use otter_lib::{
    config::Config,
    context::{self, Context},
    structures::{
        literal::{Literal, Source},
        variable::list::VariableList,
    },
};

fn satlib_path(collection: &str) -> PathBuf {
    let collection_path = Path::new(collection);
    let satlib_path = Path::new(".").join("tests").join("cnf").join("SATLIB");

    satlib_path.join(collection_path)
}

fn test_split_collection(collection: &str) {
    let config = Config::default();

    let collection_path = satlib_path(collection);

    let sat_dir = fs::read_dir(collection_path.join("sat"))
        .unwrap_or_else(|_| panic!("{collection} SAT missing"));

    for test in sat_dir.flatten() {
        let unique_config = config.clone();
        let mut the_context =
            Context::from_dimacs(&test.path(), unique_config).expect("failed to build context");
        // let _ = the_context.clause_from_string("p -q");
        let _the_result = the_context.solve();
        let sat = match the_context.status {
            context::Status::AllAssigned => true,
            context::Status::NoSolution(_) => false,
            _ => panic!("failed to solve"),
        };
        assert!(sat);
    }

    let unsat_dir = fs::read_dir(collection_path.join("unsat"))
        .unwrap_or_else(|_| panic!("{collection} UNSAT missing"));

    for test in unsat_dir.flatten() {
        let unique_config = config.clone();
        let mut the_context =
            Context::from_dimacs(&test.path(), unique_config).expect("failed to build context");
        // let _ = the_context.clause_from_string("p -q");
        let _the_result = the_context.solve();
        let sat = match the_context.status {
            context::Status::AllAssigned => true,
            context::Status::NoSolution(_) => false,
            _ => panic!("failed to solve"),
        };
        assert!(!sat);
    }
}

fn test_collection(collection: &str, sat: bool) {
    let config = Config::default();

    let collection_path = satlib_path(collection);

    let sat_dir =
        fs::read_dir(collection_path).unwrap_or_else(|_| panic!("{collection} SAT missing"));

    for test in sat_dir.flatten() {
        let unique_config = config.clone();
        let mut the_context =
            Context::from_dimacs(&test.path(), unique_config).expect("failed to build context");
        // let _ = the_context.clause_from_string("p -q");
        let _the_result = the_context.solve();
        let result = match the_context.status {
            context::Status::AllAssigned => true,
            context::Status::NoSolution(_) => false,
            _ => panic!("failed to solve"),
        };
        assert_eq!(sat, result);
    }
}

#[test]
fn uniform_random_3_50_128() {
    test_split_collection("UUF50.218.1000");
}

#[test]
fn uniform_random_3_1065_100() {
    test_split_collection("UF250.1065.100");
}

#[test]
fn logistics() {
    test_collection("logistics", true);
}

#[test]
fn blocksworld() {
    test_collection("blocksworld", true);
}

#[test]
fn ais() {
    test_collection("ais", true);
}

#[test]
fn bmc() {
    test_collection("bmc", true);
}
