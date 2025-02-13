use std::{
    cell::RefCell,
    collections::HashSet,
    path::{Path, PathBuf},
    rc::Rc,
};

use otter_sat::{
    context::Context,
    db::{clause::db_clause::dbClause, ClauseKey},
    reports::frat::{
        callback_templates::{
            transcribe_addition, transcribe_deletion, transcribe_premises, transcribe_unsatisfiable,
        },
        Transcriber,
    },
    structures::clause::ClauseSource,
};

pub fn frat_setup(cnf_path: &Path, ctx: &mut Context) -> Rc<RefCell<Transcriber>> {
    let mut frat_path = cnf_path.as_os_str().to_os_string();
    frat_path.push(".frat");

    let frat_path = PathBuf::from(&frat_path);

    let transcriber = Transcriber::new(frat_path.clone()).unwrap();
    let tx = std::rc::Rc::new(std::cell::RefCell::new(transcriber));

    let addition_tx = tx.clone();
    let addition_cb = move |clause: &dbClause, source: &ClauseSource| {
        transcribe_addition(&mut addition_tx.borrow_mut(), clause, source)
    };
    ctx.set_callback_addition(Box::new(addition_cb));

    let deletion_tx = tx.clone();
    let deletion_cb =
        move |clause: &dbClause| transcribe_deletion(&mut deletion_tx.borrow_mut(), clause);
    ctx.set_callback_delete(Box::new(deletion_cb));

    let resolution_tx = tx.clone();
    let resolution_cb = move |premises: &HashSet<ClauseKey>| {
        transcribe_premises(&mut resolution_tx.borrow_mut(), premises)
    };
    ctx.resolution_buffer
        .set_callback_resolution_premises(Box::new(resolution_cb));

    let unsatisfiable_tx = tx.clone();
    let unsatisfiable_cb = move |clause: &dbClause| {
        transcribe_unsatisfiable(&mut unsatisfiable_tx.borrow_mut(), clause)
    };
    ctx.set_callback_unsatisfiable(Box::new(unsatisfiable_cb));

    tx
}

pub fn frat_finalise(tx: Rc<RefCell<Transcriber>>, ctx: &mut Context) {
    for (key, literal) in ctx.clause_db.all_unit_clauses() {
        tx.borrow_mut().transcribe_active(key, &literal);
    }

    for (key, clause) in ctx.clause_db.all_active_nonunit_clauses() {
        tx.borrow_mut().transcribe_active(key, clause);
    }

    tx.borrow_mut().flush();
}
