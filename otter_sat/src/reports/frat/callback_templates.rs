use std::collections::HashSet;

use crate::{
    db::{clause::db_clause::dbClause, ClauseKey},
    structures::clause::ClauseSource,
};

use super::Transcriber;

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

pub fn transcribe_deletion(tx: &mut Transcriber, clause: &dbClause) {
    tx.transcribe_clause('d', clause.key(), clause, false);

    tx.flush()
}

pub fn transcribe_premises(tx: &mut Transcriber, premises: &HashSet<ClauseKey>) {
    tx.note_resolution(premises);
}

pub fn transcribe_unsatisfiable(tx: &mut Transcriber, _clause: &dbClause) {
    tx.transcribe_unsatisfiable_clause();
}
