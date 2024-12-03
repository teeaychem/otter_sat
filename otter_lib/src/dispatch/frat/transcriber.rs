#![allow(clippy::useless_format)]

use std::{borrow::Borrow, collections::VecDeque, io::Write, path::PathBuf};

use crate::{
    db::keys::ClauseKey,
    dispatch::{
        library::{
            delta::{self, Delta},
            report::{self, Report},
        },
        Dispatch,
    },
    structures::{
        literal::{Literal, LiteralT},
        variable::Variable,
    },
    types::err::{self},
};

use super::Transcriber;

/*
Use by creating a listener for dispatches from a context and passing each dispatch to the transcriber.

For the moment the transcriber automatically syncronises resolution information with new clauses by…
- Storing a clause after resolution has completed and before any other instance of resolution begins
  Specifically, the channel is FIFO and resolution information is stored in a FIFO queue.
  So, the contents of some buffered resolution information can always be associated with the relevant stored clause

A few decisions make this a little more delicate than it otherwise could be

- On-the-fly self-subsumption
  + For formulas, specifically,  means it's important to record an origial formula before subsumption is applied
    Rather than do anything complex this is addressed by writing the original formula at the start of a proof.

- Variable renaming
  + … when mixed with 0 as a delimiter in the format requires (i think) translating a clause back to it's DIMACS representation
  - The context stores a translation, but to avoid interacting (and introducing mutexes) the transcriber listens for variables being added to the context and keeps an internal map of their external string

- Multiple clause databases
  + Requires disambiguating indicies.
    As there are no explicit limits on indicies in the FRAT document, simple ASCII prefixes are used

- Proofs of literals
  - And all the above also apply to when a literal is proven and so adding a clause is skipped completely
 */

impl Transcriber {
    //! Functions to map internal identifiers to FRAT suitable identifiers.
    //!
    //! within a solve literals and clauses are distinguish both by their location and a unique identifier.
    //! The internal identifiers are all u32s, and so without some representation of the location information are ambiguous.
    //! FRAT identifiers are of the form [0-9]+, and so a simple 0*X* prefix is sufficient to disambiguate.
    // use super::*;

    pub(super) fn literal_id(literal: impl Borrow<Literal>) -> String {
        format!("010{}", literal.borrow().var())
    }

    pub(super) fn key_id(key: &ClauseKey) -> String {
        match key {
            ClauseKey::Unit(l) => format!("010{l}"),
            ClauseKey::Formula(index) => format!("020{index}"),
            ClauseKey::Binary(index) => format!("030{index}"),
            ClauseKey::Learned(index, _) => format!("040{index}"),
        }
    }

    #[doc(hidden)]
    pub(super) fn resolution_buffer_ids(buffer: Vec<ClauseKey>) -> String {
        buffer
            .iter()
            .map(Transcriber::key_id)
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl Transcriber {
    //! Functions to write a generate the string representation of an proof step.
    //!
    //! The name format is: \<*type of step*\>_\<*structure to which function applies*\>.

    pub fn original_literal(literal: impl Borrow<Literal>, external: String) -> String {
        let id_rep = Transcriber::literal_id(literal);
        format!("o {id_rep} {external} 0\n")
    }

    pub fn original_clause(key: &ClauseKey, external: String) -> String {
        let id_rep = Transcriber::key_id(key);
        format!("o {id_rep} {external} 0\n")
    }

    pub fn add_literal(
        internal: impl Borrow<Literal>,
        external: String,
        steps: Option<Vec<ClauseKey>>,
    ) -> String {
        let id_rep = Transcriber::literal_id(internal);
        let resolution_rep = match steps {
            Some(sequence) => {
                let resolution_rep = Transcriber::resolution_buffer_ids(sequence);
                format!("0 l {resolution_rep} ")
            }
            None => String::new(),
        };
        format!("a {id_rep} {external} {resolution_rep}0\n")
    }

    pub fn add_clause(key: &ClauseKey, external: String, steps: Option<Vec<ClauseKey>>) -> String {
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

    pub fn delete_clause(key: &ClauseKey, external: String) -> String {
        let id_rep = Transcriber::key_id(key);
        format!("d {id_rep} {external} 0\n")
    }

    pub fn meta_unsatisfiable() -> String {
        let mut the_string = String::new();
        the_string.push_str("a 1 0\n"); // add the contradiction
        the_string.push_str("f 1 0\n"); // finalise the contradiction
        the_string
    }

    pub fn finalise_literal(literal: impl Borrow<Literal>, external: String) -> String {
        let id_rep = Transcriber::literal_id(literal);
        format!("f {id_rep} {external} 0\n")
    }

    pub fn finalise_clause(key: &ClauseKey, external: String) -> String {
        let id_rep = Transcriber::key_id(key);
        format!("f {id_rep} {external} 0\n")
    }
}

impl Transcriber {
    pub fn new(path: PathBuf) -> Self {
        std::fs::File::create(&path);
        let file = std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap();
        Transcriber {
            file,
            clause_buffer: Vec::default(),
            resolution_buffer: Vec::default(),
            variable_buffer: String::default(),
            resolution_queue: VecDeque::default(),
            step_buffer: Vec::default(),
            variable_map: Vec::default(),
        }
    }

    fn note_variable(&mut self, variable: Variable, name: &str) {
        let required = variable as usize - self.variable_map.len();
        for _ in 0..required {
            self.variable_map.push(None);
        }
        self.variable_map.push(Some(name.to_string()));
    }

    pub fn transcribe(&mut self, dispatch: &Dispatch) -> Result<(), err::FRAT> {
        match dispatch {
            Dispatch::Delta(δ) => match δ {
                Delta::VariableDB(variable_db_δ) => self.variable_db_delta(variable_db_δ)?,

                Delta::ClauseDB(clause_db_δ) => self.clause_db_delta(clause_db_δ)?,

                Delta::LiteralDB(literal_db_δ) => self.literal_db_delta(literal_db_δ)?,

                Delta::Resolution(resolution_δ) => self.resolution_delta(resolution_δ)?,

                Delta::BCP(_) => {}
            },

            Dispatch::Report(the_report) => {
                match the_report {
                    Report::ClauseDB(report) => {
                        //
                        match report {
                            report::ClauseDB::Active(key, clause) => {
                                self.step_buffer.push(Transcriber::finalise_clause(
                                    key,
                                    self.clause_string(clause.clone()),
                                ))
                            }
                        }
                    }

                    Report::LiteralDB(report) => match report {
                        report::LiteralDB::Active(literal) => self.step_buffer.push(
                            Transcriber::finalise_literal(literal, self.literal_string(literal)),
                        ),
                    },
                    Report::Parser(_) | Report::Finish | Report::Solve(_) => {}
                }
            }

            Dispatch::Stat(_) => {}
        };
        Ok(())
    }

    pub fn flush(&mut self) {
        for step in &self.step_buffer {
            let _ = self.file.write(step.as_bytes());
        }
        self.step_buffer.clear();
    }
}

impl Transcriber {
    //! Methods to transform from internal to external representation strings.

    pub(super) fn clause_string(&self, clause: Vec<Literal>) -> String {
        clause
            .iter()
            .map(|l| self.literal_string(l))
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub(super) fn literal_string(&self, literal: impl Borrow<Literal>) -> String {
        match &self.variable_map[literal.borrow().var() as usize] {
            Some(ext) => match literal.borrow().polarity() {
                true => format!("{ext}"),
                false => format!("-{ext}"),
            },
            None => panic!("Missing external string for {}", literal.borrow()),
        }
    }
}

impl Transcriber {
    pub(super) fn variable_db_delta(&mut self, δ: &delta::VariableDB) -> Result<(), err::FRAT> {
        use delta::VariableDB::*;
        match δ {
            ExternalRepresentation(rep) => self.variable_buffer = rep.clone(),

            Internalised(variable) => {
                let rep = std::mem::take(&mut self.variable_buffer);
                self.note_variable(*variable, rep.as_str());
            }
            Unsatisfiable(_) => self.step_buffer.push(Transcriber::meta_unsatisfiable()),
        }
        Ok(())
    }

    pub(super) fn clause_db_delta(&mut self, δ: &delta::ClauseDB) -> Result<(), err::FRAT> {
        use delta::ClauseDB::*;
        match δ {
            ClauseStart => return Err(err::FRAT::CorruptClauseBuffer),

            ClauseLiteral(literal) => self.clause_buffer.push(*literal),

            Deletion(key) => {
                let the_clause = std::mem::take(&mut self.clause_buffer);
                let step = Transcriber::delete_clause(key, self.clause_string(the_clause));
                self.step_buffer.push(step);
            }

            Original(key) | BinaryOriginal(key) => {
                let the_clause = std::mem::take(&mut self.clause_buffer);
                let step = Transcriber::original_clause(key, self.clause_string(the_clause));
                self.step_buffer.push(step);
            }

            Resolution(key) | BinaryResolution(key) => {
                let the_clause = std::mem::take(&mut self.clause_buffer);
                let Some(steps) = self.resolution_queue.pop_front() else {
                    return Err(err::FRAT::CorruptResolutionQ);
                };
                let step =
                    Transcriber::add_clause(key, self.clause_string(the_clause), Some(steps));
                self.step_buffer.push(step);
            }

            TransferBinary(_from, _to) => return Err(err::FRAT::TransfersAreTodo),
        };

        Ok(())
    }

    pub(super) fn literal_db_delta(&mut self, δ: &delta::LiteralDB) -> Result<(), err::FRAT> {
        use delta::LiteralDB::*;
        match δ {
            Assumption(literal) => {
                let step = Transcriber::original_literal(literal, self.literal_string(literal));
                self.step_buffer.push(step);
            }

            ResolutionProof(literal) => {
                let Some(steps) = self.resolution_queue.pop_front() else {
                    return Err(err::FRAT::CorruptResolutionQ);
                };
                let step =
                    Transcriber::add_literal(literal, self.literal_string(literal), Some(steps));
                self.step_buffer.push(step);
            }
            Proof(literal) => {
                let step = Transcriber::add_literal(literal, self.literal_string(literal), None);
                self.step_buffer.push(step);
            }
        }
        Ok(())
    }

    pub(super) fn resolution_delta(&mut self, δ: &delta::Resolution) -> Result<(), err::FRAT> {
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
