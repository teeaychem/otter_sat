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

mod frat_id {
    //! Functions to map internal identifiers to FRAT suitable identifiers.
    //!
    //! within a solve literals and clauses are distinguish both by their location and a unique identifier.
    //! The internal identifiers are all u32s, and so without some representation of the location information are ambiguous.
    //! FRAT identifiers are of the form [0-9]+, and so a simple 0*X* prefix is sufficient to disambiguate.
    use super::*;

    pub(super) fn literal_id(literal: impl Borrow<Literal>) -> String {
        format!("010{}", literal.borrow().var())
    }

    pub(super) fn key_id(key: &ClauseKey) -> String {
        match key {
            ClauseKey::Formula(index) => format!("020{index}"),
            ClauseKey::Binary(index) => format!("030{index}"),
            ClauseKey::Learned(index, _) => format!("040{index}"),
        }
    }

    #[doc(hidden)]
    pub(super) fn resolution_buffer_ids(buffer: Vec<ClauseKey>) -> String {
        buffer.iter().map(key_id).collect::<Vec<_>>().join(" ")
    }
}

mod step {
    //! Functions to write a generate the string representation of an proof step.
    //!
    //! As there may be multiple functions for a type of step namespaces (roughly) corresponding to FRAT step type are used to further subdivide things.

    use std::borrow::Borrow;

    use super::*;

    pub(super) mod original {
        use super::*;
        pub fn literal(literal: impl Borrow<Literal>, literal_string: String) -> String {
            let id_rep = frat_id::literal_id(literal);
            format!("o {id_rep} {literal_string} 0\n")
        }

        pub fn clause(key: &ClauseKey, clause_string: String) -> String {
            let id_rep = frat_id::key_id(key);
            format!("o {id_rep} {clause_string} 0\n")
        }
    }

    pub(super) mod add {
        use super::*;

        pub fn literal(
            literal: impl Borrow<Literal>,
            literal_string: String,
            steps: Option<Vec<ClauseKey>>,
        ) -> String {
            let id_rep = frat_id::literal_id(literal);
            let resolution_rep = match steps {
                Some(sequence) => {
                    let resolution_rep = frat_id::resolution_buffer_ids(sequence);
                    format!("0 l {resolution_rep} ")
                }
                None => String::new(),
            };
            format!("a {id_rep} {literal_string} {resolution_rep}0\n")
        }

        pub fn clause(
            key: &ClauseKey,
            clause_string: String,
            steps: Option<Vec<ClauseKey>>,
        ) -> String {
            let id_rep = frat_id::key_id(key);
            let resolution_rep = match steps {
                Some(sequence) => {
                    let resolution_rep = frat_id::resolution_buffer_ids(sequence);
                    format!("0 l {resolution_rep} ")
                }
                None => String::new(),
            };
            format!("a {id_rep} {clause_string} {resolution_rep}0\n")
        }
    }

    pub(super) mod delete {
        use super::*;

        pub fn clause(key: &ClauseKey, clause_string: String) -> String {
            let id_rep = frat_id::key_id(key);
            format!("d {id_rep} {clause_string} 0\n")
        }
    }

    pub(super) mod meta {

        pub fn unsatisfiable() -> String {
            let mut the_string = String::from("a 1 0\n");
            the_string.push_str("f 1 0\n");
            the_string
        }
    }

    pub(super) mod finalise {
        use super::*;

        pub fn literal(literal: impl Borrow<Literal>, literal_string: String) -> String {
            let id_rep = frat_id::literal_id(literal);
            format!("f {id_rep} {literal_string} 0\n")
        }

        pub fn clause(key: &ClauseKey, clause_string: String) -> String {
            let id_rep = frat_id::key_id(key);
            format!("f {id_rep} {clause_string} 0\n")
        }
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
            resolution_buffer: Vec::default(),
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

    pub fn transcribe(&mut self, dispatch: &Dispatch) {
        match dispatch {
            Dispatch::Delta(δ) => match δ {
                Delta::VariableDB(variable_db_δ) => self.process_variable_db_delta(variable_db_δ),

                Delta::ClauseDB(clause_db_δ) => self.process_clause_db_delta(clause_db_δ),

                Delta::LiteralDB(literal_db_δ) => self.process_literal_db_delta(literal_db_δ),

                Delta::Resolution(resolution_δ) => self.process_resolution_delta(resolution_δ),

                Delta::BCP(_) => {}
            },

            Dispatch::Report(the_report) => {
                match the_report {
                    Report::ClauseDB(report) => {
                        //
                        match report {
                            report::ClauseDB::Active(key, clause) => self
                                .step_buffer
                                .push(step::finalise::clause(key, self.clause_string(clause))),
                        }
                    }

                    Report::LiteralDB(report) => match report {
                        report::LiteralDB::Active(literal) => self.step_buffer.push(
                            step::finalise::literal(literal, self.literal_string(literal)),
                        ),
                    },
                    Report::Parser(_) | Report::Finish | Report::Solve(_) => {}
                }
            }

            Dispatch::Comment(_) | Dispatch::Stats(_) => {}
        };
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

    pub(super) fn clause_string(&self, clause: &[Literal]) -> String {
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
    pub(super) fn process_variable_db_delta(&mut self, δ: &delta::VariableDB) {
        match δ {
            delta::VariableDB::Internalised(variable, name) => {
                self.note_variable(*variable, name);
            }
            delta::VariableDB::Unsatisfiable(_) => {
                self.step_buffer.push(step::meta::unsatisfiable())
            }
        }
    }

    pub(super) fn process_clause_db_delta(&mut self, δ: &delta::ClauseDB) {
        match δ {
            delta::ClauseDB::Deletion(key, clause) => {
                let step = step::delete::clause(key, self.clause_string(clause));
                self.step_buffer.push(step);
            }

            delta::ClauseDB::Original(key, clause)
            | delta::ClauseDB::BinaryOriginal(key, clause) => {
                let step = step::original::clause(key, self.clause_string(clause));
                self.step_buffer.push(step);
            }

            delta::ClauseDB::Resolution(key, clause)
            | delta::ClauseDB::BinaryResolution(key, clause) => {
                let step = step::add::clause(
                    key,
                    self.clause_string(clause),
                    Some(self.resolution_queue.pop_front().expect("nri_l")),
                );
                self.step_buffer.push(step);
            }

            delta::ClauseDB::TransferBinary(_from, _to, _clause) => {
                todo!("Clause transfers have not been implemented");
            }
        };
    }

    pub(super) fn process_literal_db_delta(&mut self, δ: &delta::LiteralDB) {
        match δ {
            delta::LiteralDB::Assumption(literal) | delta::LiteralDB::Pure(literal) => {
                let step = step::original::literal(literal, self.literal_string(literal));
                self.step_buffer.push(step);
            }

            delta::LiteralDB::ResolutionProof(literal) => {
                let resolution_steps = self.resolution_queue.pop_front().expect("nri_rp");
                let step = step::add::literal(
                    literal,
                    self.literal_string(literal),
                    Some(resolution_steps),
                );
                self.step_buffer.push(step);
            }
            delta::LiteralDB::Proof(literal) => {
                let step = step::add::literal(literal, self.literal_string(literal), None);
                self.step_buffer.push(step);
            }
            delta::LiteralDB::Forced(_, _) => {
                // forced at level zero?
            }
        }
    }

    pub(super) fn process_resolution_delta(&mut self, δ: &delta::Resolution) {
        match δ {
            delta::Resolution::Begin => {
                assert!(self.resolution_buffer.is_empty());
            }
            delta::Resolution::End => {
                self.resolution_queue
                    .push_back(std::mem::take(&mut self.resolution_buffer));
            }
            delta::Resolution::Used(k) => {
                self.resolution_buffer.push(*k);
            }
            delta::Resolution::Subsumed(_, _) => {
                // TODO: Someday… maybe…
            }
        }
    }
}
