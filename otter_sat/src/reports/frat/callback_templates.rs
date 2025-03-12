/*!
Functions to hook FRAT proof transcription into callbacks.

Each function takes a transcriber and a parameters corresponding to those relevant from the callback.

For examples of use, see the bundled otter_cli.
*/

use std::collections::HashSet;

use crate::{
    db::{ClauseKey, clause::db_clause::dbClause},
    structures::clause::ClauseSource,
};

use super::Transcriber;

/// Transcribe the addition of an original or addition clause to the context.
pub fn transcribe_addition(tx: &mut Transcriber, clause: &dbClause, source: &ClauseSource) {
    match source {
        ClauseSource::BCP => {
            if let ClauseKey::AdditionUnit(literal) = clause.key() {
                tx.transcribe_clause('a', clause.key(), literal, false);
            } else {
                panic!("");
            }
        }

        ClauseSource::Original => tx.transcribe_clause('o', clause.key(), clause.clause(), false),

        ClauseSource::Resolution => tx.transcribe_clause('a', clause.key(), clause.clause(), true),

        ClauseSource::PureUnit => panic!("X_X"),
    }
    tx.flush()
}

/// Transcribe the deletion of a clause from the context.
pub fn transcribe_deletion(tx: &mut Transcriber, clause: &dbClause) {
    tx.transcribe_clause('d', clause.key(), clause, false);

    tx.flush()
}

/// Transcribe premises used in an instance of resolution.
pub fn transcribe_premises(tx: &mut Transcriber, premises: &HashSet<ClauseKey>) {
    tx.note_resolution(premises);
}

/// Transcribe the relevant information to highlight that an unsatisfiable clause has been derived.
pub fn transcribe_unsatisfiable(tx: &mut Transcriber, _clause: &dbClause) {
    tx.transcribe_unsatisfiable_clause();
}
