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
                tx.transcribe_bcp(clause.key(), *literal);
            } else {
                panic!("");
            }
        }

        ClauseSource::Original => tx.transcribe_original_clause(clause.key(), clause.clause()),

        ClauseSource::Resolution => tx.transcribe_addition_clause(clause.key(), clause.clause()),

        ClauseSource::PureUnit => panic!("X_X"),
    }
    tx.flush()
}

pub fn transcribe_deletion(tx: &mut Transcriber, clause: &dbClause) {
    tx.transcribe_deletion(clause.key(), clause.clause());

    tx.flush()
}

pub fn transcribe_premises(tx: &mut Transcriber, premises: &HashSet<ClauseKey>) {
    tx.transcribe_resolution(premises);
}

pub fn transcribe_unsatisfiable(tx: &mut Transcriber, _clause: &dbClause) {
    tx.transcribe_unsatisfiable_clause();
}
