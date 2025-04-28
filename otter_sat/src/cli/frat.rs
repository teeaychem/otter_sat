use std::{
    cell::RefCell,
    collections::HashSet,
    path::{Path, PathBuf},
    rc::Rc,
};

use otter_sat::{
    context::Context,
    db::{ClauseKey, clause::db_clause::DBClause},
    reports::frat::{
        Transcriber,
        callback_templates::{
            transcribe_addition, transcribe_deletion, transcribe_premises, transcribe_unsatisfiable,
        },
    },
    structures::clause::ClauseSource,
};

/// Create a file to write the FRAT proof to and set transiption callbacks where required.
///
/// Returns a smart pointer to the transcriber.
pub(super) fn frat_setup(cnf_path: &Path, ctx: &mut Context) -> Rc<RefCell<Transcriber>> {
    /*
    Setup amounts to linking a transcriber to the given context by registering a collection of callbacks.

    To keep things simple, the callbacks borrow a mutable reference to the transcriber.
     */

    let mut frat_path = cnf_path.as_os_str().to_owned();
    frat_path.push(".frat");

    let frat_path = PathBuf::from(&frat_path);

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

    tx
}

/// Finalise the FRAT proof written by the given transcriber.
pub(super) fn frat_finalise(transcriber: Rc<RefCell<Transcriber>>, context: &mut Context) {
    for (key, literal) in context.clause_db.all_unit_clauses() {
        transcriber.borrow_mut().transcribe_active(key, &literal);
    }

    for (key, clause) in context.clause_db.all_active_nonunit_clauses() {
        transcriber.borrow_mut().transcribe_active(key, clause);
    }

    transcriber.borrow_mut().flush();
}
