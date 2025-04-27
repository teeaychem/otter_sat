use std::{collections::HashSet, path::PathBuf};

use otter_sat::{
    config::Config,
    context::Context,
    db::{clause::db_clause::DBClause, ClauseKey},
    reports::frat::{
        callback_templates::{
            transcribe_addition, transcribe_deletion, transcribe_premises, transcribe_unsatisfiable,
        },
        Transcriber,
    },
    structures::clause::ClauseSource,
};

use crate::general::load_dimacs;

const FRAT_RS_PATH: &str = "./frat-rs";

pub fn frat_verify(file_path: PathBuf, config: Config) -> bool {
    #[cfg(feature = "log")]
    env_logger::init();

    let mut frat_path_string = file_path.clone().to_str().unwrap().to_owned();
    frat_path_string.push_str(".frat");
    let frat_path = PathBuf::from(&frat_path_string);

    let mut ctx = Context::from_config(config);

    let transcriber = Transcriber::new(frat_path.clone()).unwrap();
    let tx = std::rc::Rc::new(std::cell::RefCell::new(transcriber));

    let addition_tx = tx.clone();
    let addition_cb = move |clause: &DBClause, source: &ClauseSource| {
        transcribe_addition(&mut addition_tx.borrow_mut(), clause, source)
    };
    ctx.set_callback_addition(Box::new(addition_cb));

    let deletion_tx = tx.clone();
    let deletion_cb =
        move |clause: &DBClause| transcribe_deletion(&mut deletion_tx.borrow_mut(), clause);
    ctx.set_callback_delete(Box::new(deletion_cb));

    let resolution_tx = tx.clone();
    let resolution_cb = move |premises: &HashSet<ClauseKey>| {
        transcribe_premises(&mut resolution_tx.borrow_mut(), premises)
    };
    ctx.atom_cells
        .set_callback_resolution_premises(Box::new(resolution_cb));

    let unsatisfiable_tx = tx.clone();
    let unsatisfiable_cb = move |clause: &DBClause| {
        transcribe_unsatisfiable(&mut unsatisfiable_tx.borrow_mut(), clause)
    };
    ctx.set_callback_unsatisfiable(Box::new(unsatisfiable_cb));

    match load_dimacs(&mut ctx, &file_path) {
        Ok(()) => {}
        Err(e) => panic!("c Error loading file: {e:?}"),
    };

    let _result = ctx.solve();

    for (key, literal) in ctx.clause_db.all_unit_clauses() {
        tx.borrow_mut().transcribe_active(key, &literal);
    }

    for (key, clause) in ctx.clause_db.all_active_nonunit_clauses() {
        tx.borrow_mut().transcribe_active(key, clause);
    }

    tx.borrow_mut().flush();

    let mut frat_process = std::process::Command::new(FRAT_RS_PATH);
    frat_process.arg("elab");
    frat_process.arg(frat_path_string.clone());
    frat_process.arg("-m"); // keep the intermediate file in memory

    let output = match frat_process.output() {
        Ok(out) => out,
        Err(e) => panic!("{e:?}"),
    };

    let _ = std::fs::remove_file(frat_path);
    match output.status.code() {
        Some(0) => true,

        _unexpected_output_code => {
            println!("{output:?}");
            false
        }
    }
}

pub fn frat_dir_test(dir: PathBuf) -> usize {
    let mut counter = 0;

    if let Some(dir) = dir.to_str() {
        for entry in glob::glob(format!("{dir}/*.xz").as_str()).expect("bad glob") {
            let formula = entry.unwrap();
            let mut config = Config::default();
            config.subsumption.value = false;

            match frat_verify(formula, config) {
                true => counter += 1,
                false => break,
            }
        }
    }

    counter
}
