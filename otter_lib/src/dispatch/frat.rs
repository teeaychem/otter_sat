#![allow(clippy::useless_format)]

use std::{borrow::Borrow, collections::VecDeque, fs::File, io::Write, path::PathBuf};

use crate::{
    db::keys::ClauseKey,
    dispatch::{
        self,
        delta::{self},
        Dispatch,
    },
    structures::literal::{Literal, LiteralT},
};

/*
A transcriber for recording FRAT proofs

Use by creating a listener for dispatches from a context and passing each dispatch to the transcriber.

For the moment the transcriber automatically synronises resolution information with new clauses by…
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

#[allow(dead_code)]
pub struct Transcriber {
    path: PathBuf,
    file: File,
    step_buffer: Vec<String>,
    resolution_buffer: Vec<ClauseKey>,
    resolution_queue: VecDeque<Vec<ClauseKey>>,
    variable_map: Vec<Option<String>>,
}

impl Transcriber {
    pub fn new(path: PathBuf) -> Self {
        std::fs::File::create(&path);
        let file = std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap();
        Transcriber {
            path,
            file,
            resolution_buffer: Vec::default(),
            resolution_queue: VecDeque::default(),
            step_buffer: Vec::default(),
            variable_map: Vec::default(),
        }
    }

    pub fn transcribe(&mut self, dispatch: &Dispatch) {
        let transcription = match dispatch {
            //
            Dispatch::VariableDB(v_delta) => match v_delta {
                delta::Variable::Internalised(name, id) => {
                    let required = *id as usize - self.variable_map.len();
                    for _ in 0..required {
                        self.variable_map.push(None);
                    }
                    // let name_clone = name.clone();
                    self.variable_map.push(Some(name.clone()));
                    // assert_eq!(self.variable_map[*id as usize], Some(name_clone));
                    None
                }
                delta::Variable::Unsatisfiable(_) => {
                    let mut the_string = String::from("a 1 0\n");
                    the_string.push_str("f 1");
                    Some(the_string)
                }
            },

            Dispatch::ClauseDB(store_delta) => {
                //
                match store_delta {
                    delta::ClauseDB::Deletion(key, clause) => {
                        let mut the_string = format!("d {} ", Self::key_id(key));
                        the_string.push_str(&self.externalised_clause(clause));
                        Some(the_string)
                    }

                    delta::ClauseDB::Original(key, clause) => {
                        let mut the_string = format!("o {} ", Self::key_id(key));
                        the_string.push_str(&self.externalised_clause(clause));
                        Some(the_string)
                    }

                    delta::ClauseDB::TransferBinary(from, to, clause) => {
                        // Derive new, delete formula
                        let mut the_string = format!("a {} ", Self::key_id(to));
                        the_string.push_str(&self.externalised_clause(clause));
                        the_string.push_str(" 0 l ");
                        let resolution_steps = self.resolution_queue.pop_front().expect("nri");
                        the_string.push_str(&Self::resolution_buffer_ids(resolution_steps));
                        the_string.push_str(format!("d {} 0\n", Self::key_id(from)).as_str());
                        Some(the_string)
                    }

                    delta::ClauseDB::Learned(key, clause) => {
                        let mut the_string = format!("a {} ", Self::key_id(key));
                        the_string.push_str(&self.externalised_clause(clause));
                        the_string.push_str(" 0 l ");
                        the_string.push_str(&Self::resolution_buffer_ids(
                            self.resolution_queue.pop_front().expect("nri_l"),
                        ));
                        Some(the_string)
                    }
                    delta::ClauseDB::BinaryOriginal(key, clause) => {
                        let mut the_string = format!("o {} ", Self::key_id(key));
                        the_string.push_str(&self.externalised_clause(clause));
                        Some(the_string)
                    }
                    delta::ClauseDB::BinaryResolution(key, clause) => {
                        let mut the_string = format!("a {} ", Self::key_id(key));
                        the_string.push_str(&self.externalised_clause(clause));
                        the_string.push_str(" 0 l ");
                        the_string.push_str(&Self::resolution_buffer_ids(
                            self.resolution_queue.pop_front().expect("nri_br"),
                        ));
                        Some(the_string)
                    }
                }
            }
            Dispatch::Level(level_delta) => {
                //
                match level_delta {
                    delta::Level::Assumption(literal) => Some(format!(
                        "o {} {}",
                        Self::literal_id(literal),
                        self.externalised_literal(literal)
                    )),
                    delta::Level::ResolutionProof(literal) => {
                        let mut the_string = format!(
                            "a {} {}",
                            Self::literal_id(literal),
                            self.externalised_literal(literal)
                        );
                        the_string.push_str(" 0 l ");
                        the_string.push_str(&Self::resolution_buffer_ids(
                            self.resolution_queue.pop_front().expect("nri_rp"),
                        ));
                        Some(the_string)
                    }
                    delta::Level::Pure(literal) => Some(format!(
                        "o {} {}",
                        Self::literal_id(literal),
                        self.externalised_literal(literal)
                    )),
                    delta::Level::Proof(literal) => Some(format!(
                        "a {} {}",
                        Self::literal_id(literal),
                        self.externalised_literal(literal)
                    )),
                    delta::Level::Forced(_, _) => None,
                }
            }

            Dispatch::ClauseDBReport(report) => match report {
                dispatch::report::ClauseDB::Active(key, clause) => {
                    let mut the_string = format!("f {} ", Self::key_id(key));
                    the_string.push_str(&self.externalised_clause(clause));
                    Some(the_string)
                }
            },

            Dispatch::VariableDBReport(report) => match report {
                dispatch::report::VariableDB::Active(literal) => {
                    let the_string = format!(
                        "f {} {}",
                        Self::literal_id(literal),
                        self.externalised_literal(literal)
                    );
                    Some(the_string)
                }
            },

            Dispatch::Resolution(delta) => {
                match delta {
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
                None
            }

            Dispatch::Parser(_) => None,

            Dispatch::SolveComment(_) => None,
            Dispatch::SolveReport(_) => None,
            Dispatch::Stats(_) => None,
            Dispatch::Finish => None,
            Dispatch::BCP(_) => None,
        };
        if let Some(mut step) = transcription {
            step.push_str(" 0\n");
            self.step_buffer.push(step.to_string())
        }
    }

    pub fn flush(&mut self) {
        for step in &self.step_buffer {
            let _ = self.file.write(step.as_bytes());
        }
        self.step_buffer.clear();
    }
}

impl Transcriber {
    fn literal_id(literal: impl Borrow<Literal>) -> String {
        format!("10{}", literal.borrow().var())
    }

    fn key_id(key: &ClauseKey) -> String {
        match key {
            ClauseKey::Formula(index) => format!("20{index}"),
            ClauseKey::Binary(index) => format!("30{index}"),
            ClauseKey::Learned(index, _) => format!("40{index}"),
        }
    }

    fn resolution_buffer_ids(buffer: Vec<ClauseKey>) -> String {
        let mut the_string = String::default();
        for key in buffer {
            the_string.push_str(Self::key_id(&key).as_str());
            the_string.push(' ');
        }
        the_string.pop();
        the_string
    }

    fn externalised_clause(&self, clause: &[Literal]) -> String {
        let mut the_string = String::default();
        for literal in clause {
            the_string.push_str(format!("{} ", self.externalised_literal(literal)).as_str());
        }
        the_string.pop();
        the_string
    }

    fn externalised_literal(&self, literal: impl Borrow<Literal>) -> String {
        match &self.variable_map[literal.borrow().var() as usize] {
            Some(ext) => match literal.borrow().polarity() {
                true => format!("{ext}"),
                false => format!("-{ext}"),
            },
            None => panic!("Missing external string for {}", literal.borrow()),
        }
    }
}
