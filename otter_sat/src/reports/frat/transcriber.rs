#![allow(clippy::useless_format)]

use std::{collections::HashSet, fmt::Write};

use crate::{
    db::ClauseKey,
    structures::{clause::Clause, literal::Literal},
};

use super::Transcriber;

impl Transcriber {
    /// A unique identifier for a clause
    ///
    /// Within a solve literals and clauses are distinguish both by their location and a unique identifier.
    /// The internal identifiers are all u32s, and so without some representation of the location information are ambiguous.
    /// FRAT identifiers are of the form [0-9]+, and so a simple 0*x* prefix is sufficient to disambiguate.
    fn write_id_to_string(key: &ClauseKey, string: &mut String) {
        match key {
            ClauseKey::OriginalUnit(literal) => match literal.polarity() {
                true => write!(string, "0110{}", literal.atom()),
                false => write!(string, "0100{}", literal.atom()),
            },

            ClauseKey::AdditionUnit(literal) => match literal.polarity() {
                true => write!(string, "0210{}", literal.atom()),
                false => write!(string, "0200{}", literal.atom()),
            },

            ClauseKey::OriginalBinary(index) => write!(string, "0300{index}"),

            ClauseKey::AdditionBinary(index) => write!(string, "0400{index}"),

            ClauseKey::Original(index) => write!(string, "0500{index}"),

            ClauseKey::Addition(index, _) => write!(string, "0600{index}"),
        };
    }

    /// Returns the external representation of a clause as a string of literals concatenated by a space (with no closing delimiter).
    fn write_clause_to_string(&self, clause: &impl Clause, string: &mut String) {
        for literal in clause.literals() {
            let atom = literal.atom();

            match literal.polarity() {
                true => write!(string, " {atom} "),
                false => write!(string, "-{atom} "),
            };
        }
    }

    /// Flushes any buffered steps to the proof file.
    pub fn flush(&mut self) {
        for step in &self.step_buffer {
            let _ = std::io::Write::write(&mut self.file, step.as_bytes());
        }
        self.step_buffer.clear();
    }

    /// Record a proof of unsatisfiability has concluded.
    ///
    /// From the FRAT spec, this is (required and) done by writing the empty clause.
    /// '1' is used as an identifier as all other clause ids begin with '0'.
    pub fn transcribe_unsatisfiable_clause(&mut self) {
        let mut step = String::new();
        writeln!(step, "a 1 0\n"); // add the empty clause
        writeln!(step, "f 1 0\n"); // finalise the empty clause

        self.step_buffer.push(step)
    }

    /// Transcribes `clause` and (optionally) the premises used to derive the clause.
    pub fn transcribe_clause(
        &mut self,
        step_id: char,
        key: &ClauseKey,
        clause: &impl Clause,
        premises: bool,
    ) {
        let mut step = format!("{step_id} ");
        Transcriber::write_id_to_string(key, &mut step);
        write!(step, " ");

        self.write_clause_to_string(clause, &mut step);
        writeln!(step, "0");
        if premises {
            let Some(steps) = self.resolution_queue.pop_front() else {
                panic!("Err(err::FRATError::CorruptResolutionQ)")
            };
            write!(step, " l ");

            for premise in steps.into_iter() {
                Transcriber::write_id_to_string(&premise, &mut step);
                write!(step, " ");
            }
            writeln!(step, "0");
        }

        self.step_buffer.push(step);
    }

    /// Notes the premises used in an instance of resolution.
    /// These will be used with the next call to `transcribe_clause` with premises set to true.
    pub fn note_resolution(&mut self, premises: &HashSet<ClauseKey>) {
        self.resolution_queue
            .push_back(premises.iter().copied().collect());
    }

    /// Transcribes that a clause is active.
    pub fn transcribe_active(&mut self, key: ClauseKey, clause: &impl Clause) {
        let mut step = format!("f ");
        Transcriber::write_id_to_string(&key, &mut step);
        write!(step, " ");
        self.write_clause_to_string(clause, &mut step);
        writeln!(step, "0");

        self.step_buffer.push(step);
    }
}
