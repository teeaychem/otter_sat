#![allow(clippy::useless_format)]

use std::{borrow::Borrow, collections::VecDeque, io::Write, path::PathBuf};

use crate::{
    db::ClauseKey,
    dispatch::{
        library::{
            delta::{self, Delta},
            report::{self, Report},
        },
        Dispatch,
    },
    structures::{
        clause::cClause,
        literal::{cLiteral, Literal},
    },
    types::err::{self},
};

use super::Transcriber;
type ResolutionSteps = Vec<ClauseKey>;

impl Transcriber {
    /// A new transcriber which will write a proof to the given path, if some proof exists.
    pub fn new(path: PathBuf) -> Result<Self, std::io::Error> {
        std::fs::File::create(&path);
        let file = std::fs::OpenOptions::new().append(true).open(&path)?;
        let transcriber = Transcriber {
            file,
            clause_buffer: Vec::default(),
            resolution_buffer: Vec::default(),
            resolution_queue: VecDeque::default(),
            step_buffer: Vec::default(),
        };
        Ok(transcriber)
    }

    /// Transcribes a dispatch.
    pub fn transcribe(&mut self, dispatch: &Dispatch) -> Result<(), err::FRATError> {
        match dispatch {
            Dispatch::Delta(δ) => match δ {
                Delta::AtomDB(atom_db_δ) => self.transcribe_atom_db_delta(atom_db_δ)?,

                Delta::ClauseDB(clause_db_δ) => self.transcribe_clause_db_delta(clause_db_δ)?,

                Delta::LiteralDB(literal_db_δ) => {
                    self.transcribe_literal_db_delta(literal_db_δ)?
                }

                Delta::Resolution(resolution_δ) => {
                    self.transcribe_resolution_delta(resolution_δ)?
                }

                Delta::BCP(_) => {}
            },

            Dispatch::Report(the_report) => {
                match the_report {
                    Report::ClauseDB(report) => {
                        //
                        match report {
                            report::ClauseDBReport::Active(key, clause) => {
                                self.step_buffer.push(Transcriber::finalise_clause(
                                    key,
                                    self.clause_string(clause.clone()),
                                ))
                            }

                            report::ClauseDBReport::ActiveOriginalUnit(literal) => self
                                .step_buffer
                                .push(Transcriber::finalise_original_unit_clause(
                                    literal,
                                    self.literal_string(literal),
                                )),

                            report::ClauseDBReport::ActiveAdditionUnit(literal) => self
                                .step_buffer
                                .push(Transcriber::finalise_addition_unit_clause(
                                    literal,
                                    self.literal_string(literal),
                                )),
                        }
                    }
                    Report::LiteralDB(_)
                    | Report::Parser(_)
                    | Report::Finish
                    | Report::Solve(_) => {}
                }
            }

            Dispatch::Stat(_) => {}
        };
        Ok(())
    }

    /// Flushes any buffered steps to the proof file.
    pub fn flush(&mut self) {
        for step in &self.step_buffer {
            let _ = self.file.write(step.as_bytes());
        }
        self.step_buffer.clear();
    }
}

/// Functions to map internal identifiers to FRAT suitable identifiers.
///
/// Within a solve literals and clauses are distinguish both by their location and a unique identifier.
/// The internal identifiers are all u32s, and so without some representation of the location information are ambiguous.
/// FRAT identifiers are of the form [0-9]+, and so a simple 0*x* prefix is sufficient to disambiguate.
impl Transcriber {
    /// The identifier of the given literal.
    fn unit_clause_id_original(literal: impl Borrow<cLiteral>) -> String {
        let literal = literal.borrow();
        match literal.polarity() {
            true => format!("0110{}", literal.atom()),
            false => format!("0100{}", literal.atom()),
        }
    }

    fn unit_clause_id_addition(literal: impl Borrow<cLiteral>) -> String {
        let literal = literal.borrow();
        match literal.polarity() {
            true => format!("0210{}", literal.atom()),
            false => format!("0200{}", literal.atom()),
        }
    }

    /// The identifier of the given clause.
    fn key_id(key: &ClauseKey) -> String {
        match key {
            ClauseKey::OriginalUnit(literal) => Transcriber::unit_clause_id_original(literal),
            ClauseKey::AdditionUnit(literal) => Transcriber::unit_clause_id_addition(literal),

            ClauseKey::Original(index) => format!("030{index}"),
            ClauseKey::Binary(index) => format!("040{index}"),
            ClauseKey::Addition(index, _) => format!("050{index}"),
        }
    }

    /// Maps a vector of clause keys to a string of their ids.
    fn resolution_buffer_ids(buffer: Vec<ClauseKey>) -> String {
        buffer
            .iter()
            .map(Transcriber::key_id)
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Functions to write a generate the string representation of an proof step.
///
/// The name format is: \<*type of step*\>_\<*structure to which function applies*\>.
impl Transcriber {
    /// Returns the string representation of a literal.
    fn literal_string(&self, literal: impl Borrow<cLiteral>) -> String {
        let literal = literal.borrow();
        let atom = literal.atom();

        match literal.polarity() {
            true => format!(" {atom}"),
            false => format!("-{atom}"),
        }
    }

    /// Returns the external representation of a clause as a string of literals concatenated by a space (with no closing delimiter).
    fn clause_string(&self, clause: cClause) -> String {
        clause
            .iter()
            .map(|l| self.literal_string(l))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// The clause is part of the original formula.
    fn original_clause(key: &ClauseKey, external: String) -> String {
        let id_rep = Transcriber::key_id(key);
        format!("o {id_rep} {external} 0\n")
    }

    /// The clause has been added, with a note of resolution steps as optional.
    fn add_clause(key: &ClauseKey, external: String, steps: Option<ResolutionSteps>) -> String {
        let id_rep = Transcriber::key_id(key);
        let resolution_rep = match steps {
            Some(sequence) => {
                let resolution_rep = Transcriber::resolution_buffer_ids(sequence);
                format!("0 l {resolution_rep} ")
            }
            None => String::new(),
        };
        format!("a {id_rep} {external} {resolution_rep}0\n")
    }

    /// The clause has been (or will be) deleted.
    fn delete_clause(key: &ClauseKey, external: String) -> String {
        let id_rep = Transcriber::key_id(key);
        format!("d {id_rep} {external} 0\n")
    }

    /// FRAT proofs require the addition of an empty clause to observe a proof of unsatisfiability has concluded.
    ///
    /// '1' is used to identify the empty clause.
    /// The ids of all original and added clauses begin with '0', so there is no conflict.
    fn meta_unsatisfiable() -> String {
        let mut the_string = String::new();
        the_string.push_str("a 1 0\n"); // add the contradiction
        the_string.push_str("f 1 0\n"); // finalise the contradiction
        the_string
    }

    /// Finalises a unit clause.
    ///
    /// Distinguished from finalising a non-unit clause on with respect to paramaters.
    fn finalise_original_unit_clause(literal: impl Borrow<cLiteral>, external: String) -> String {
        let id_rep = Transcriber::unit_clause_id_original(literal);
        format!("f {id_rep} {external} 0\n")
    }

    /// Finalises a unit clause.
    ///
    /// Distinguished from finalising a non-unit clause on with respect to paramaters.
    fn finalise_addition_unit_clause(literal: impl Borrow<cLiteral>, external: String) -> String {
        let id_rep = Transcriber::unit_clause_id_addition(literal);
        format!("f {id_rep} {external} 0\n")
    }

    /// Finalises a non-unit clause.
    ///
    /// Distinguished from finalising a unit clause on with respect to paramaters.
    fn finalise_clause(key: &ClauseKey, external: String) -> String {
        let id_rep = Transcriber::key_id(key);
        format!("f {id_rep} {external} 0\n")
    }
}

/// Helper methods for transcription.
impl Transcriber {
    fn transcribe_atom_db_delta(&mut self, δ: &delta::AtomDB) -> Result<(), err::FRATError> {
        use delta::AtomDB::*;
        match δ {
            Unsatisfiable(_) => self.step_buffer.push(Transcriber::meta_unsatisfiable()),
        }
        Ok(())
    }

    fn transcribe_clause_db_delta(&mut self, δ: &delta::ClauseDB) -> Result<(), err::FRATError> {
        use delta::ClauseDB::*;
        match δ {
            ClauseStart => return Err(err::FRATError::CorruptClauseBuffer),

            ClauseLiteral(literal) => self.clause_buffer.push(*literal),

            Original(key) => {
                let step = match key {
                    ClauseKey::OriginalUnit(literal) => {
                        Transcriber::original_clause(key, self.literal_string(literal))
                    }

                    ClauseKey::AdditionUnit(_) => panic!("!"),

                    _ => {
                        let clause = std::mem::take(&mut self.clause_buffer);
                        Transcriber::original_clause(key, self.clause_string(clause))
                    }
                };
                self.step_buffer.push(step);
            }

            Added(key) => {
                let Some(steps) = self.resolution_queue.pop_front() else {
                    return Err(err::FRATError::CorruptResolutionQ);
                };
                let step = match key {
                    ClauseKey::OriginalUnit(_) => panic!("!"),

                    ClauseKey::AdditionUnit(literal) => {
                        Transcriber::add_clause(key, self.literal_string(literal), None)
                    }

                    _ => {
                        let the_clause = std::mem::take(&mut self.clause_buffer);
                        Transcriber::add_clause(key, self.clause_string(the_clause), Some(steps))
                    }
                };
                self.step_buffer.push(step);
            }

            BCP(key) => match key {
                ClauseKey::OriginalUnit(_) => panic!("! Original BCP"),

                ClauseKey::AdditionUnit(literal) => {
                    let step = Transcriber::add_clause(key, self.literal_string(literal), None);
                    self.step_buffer.push(step);
                }

                _ => panic!("only unit clause keys from BCP"),
            },

            Deletion(key) => {
                let the_clause = std::mem::take(&mut self.clause_buffer);
                let step = Transcriber::delete_clause(key, self.clause_string(the_clause));
                self.step_buffer.push(step);
            }

            Transfer(_from, _to) => return Err(err::FRATError::TransfersAreTodo),
        };

        Ok(())
    }

    fn transcribe_literal_db_delta(
        &mut self, _δ: &delta::LiteralDB
    ) -> Result<(), err::FRATError> {
        Ok(())
    }

    fn transcribe_resolution_delta(
        &mut self, δ: &delta::Resolution
    ) -> Result<(), err::FRATError> {
        use delta::Resolution::*;
        match δ {
            Begin => assert!(self.resolution_buffer.is_empty()),

            End => self
                .resolution_queue
                .push_back(std::mem::take(&mut self.resolution_buffer)),

            Used(k) => self.resolution_buffer.push(*k),

            Subsumed(_, _) => {} // TODO: Someday… maybe…
        }
        Ok(())
    }
}
