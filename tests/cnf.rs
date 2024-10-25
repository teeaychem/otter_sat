#![allow(unused_imports)]

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

#[test]
fn uniform_random_3_sat_50_128_sat() {
    let config = Config::default();

    let path = Path::new("./tests/cnf/SATLIB/UUF50.218.1000/sat");
    let sat_dir = fs::read_dir(path).expect("UUF50.218.1000 SAT missing");

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
}

#[test]
fn uniform_random_3_sat_50_128_unsat() {
    let config = Config::default();

    let path = Path::new("./tests/cnf/SATLIB/UUF50.218.1000/unsat");
    let unsat_dir = fs::read_dir(path).expect("UUF50.218.1000 UNSAT missing");

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
