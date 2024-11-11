#![allow(clippy::useless_format)]

use std::{
    borrow::Borrow,
    collections::VecDeque,
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

use crate::{
    config::Config,
    context::stores::{variable::VariableStore, ClauseKey},
    dispatch::{
        self,
        delta::{self},
        Dispatch,
    },
    structures::{
        clause::Clause,
        literal::{self, Literal, LiteralTrait},
    },
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

pub struct Transcriber {
    path: PathBuf,
    file: File,
    step_buffer: Vec<String>,
    pub resolution_buffer: VecDeque<Vec<ClauseKey>>,
    variable_map: Vec<Option<String>>,
}

impl Transcriber {
    pub fn new(path: PathBuf) -> Self {
        std::fs::File::create(&path);
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap();
        Transcriber {
            path,
            file,
            resolution_buffer: VecDeque::default(),
            step_buffer: Vec::default(),
            variable_map: Vec::default(),
        }
    }

    fn literal_id(literal: Literal) -> String {
        format!("10{}", literal.index())
    }

    fn key_id(key: ClauseKey) -> String {
        match key {
            ClauseKey::Formula(index) => format!("20{index}"),
            ClauseKey::Binary(index) => format!("30{index}"),
            ClauseKey::Learned(index, _) => format!("40{index}"),
        }
    }

    fn resolution_buffer_ids(buffer: Vec<ClauseKey>) -> String {
        let mut the_string = String::default();
        for key in buffer {
            the_string.push_str(Self::key_id(key).as_str());
            the_string.push(' ');
        }
        the_string.pop();
        the_string
    }

    fn externalised_clause(&self, clause: Vec<Literal>) -> String {
        let mut the_string = String::default();
        for literal in clause {
            the_string.push_str(format!("{} ", self.externalised_literal(literal)).as_str());
        }
        the_string.pop();
        the_string
    }

    fn externalised_literal(&self, literal: Literal) -> String {
        match &self.variable_map[literal.index()] {
            Some(ext) => match literal.polarity() {
                true => format!("{ext}"),
                false => format!("-{ext}"),
            },
            None => panic!("Missing external string for {literal}"),
        }
    }

    pub fn transcripe(&mut self, dispatch: Dispatch) {
        let mut transcription = match dispatch {
            //
            Dispatch::VariableDB(v_delta) => match v_delta {
                delta::Variable::Internalised(name, id) => {
                    let mut required = id as usize - self.variable_map.len();
                    for _ in 0..required {
                        self.variable_map.push(None);
                    }
                    let name_clone = name.clone();
                    self.variable_map.push(Some(name));
                    assert_eq!(self.variable_map[id as usize], Some(name_clone));
                    None
                }
                delta::Variable::Falsum(literal) => {
                    let mut the_string = String::new();
                    the_string.push_str("a 1 0\n");
                    the_string.push_str("f 1");
                    Some(the_string)
                }
            },

            Dispatch::ClauseDB(store_delta) => {
                // x
                match store_delta {
                    delta::ClauseDB::Deletion(key) => Some(format!("d {}", Self::key_id(key))),

                    delta::ClauseDB::Formula(key, clause) => {
                        let mut the_string = format!("o {} ", Self::key_id(key));
                        the_string.push_str(&self.externalised_clause(clause));
                        Some(the_string)
                    }

                    delta::ClauseDB::TransferFormulaBinary(from, to, clause) => {
                        /*
                        Derive new, delete formula
                         */
                        let mut the_string = format!("a {} ", Self::key_id(to));
                        the_string.push_str(&self.externalised_clause(clause));
                        the_string.push_str(" 0 l ");
                        the_string.push_str(&Self::resolution_buffer_ids(
                            self.resolution_buffer.pop_front().expect("nri_tf"),
                        ));
                        the_string.push_str(format!("d {} 0\n", Self::key_id(from)).as_str());
                        Some(the_string)
                    }
                    delta::ClauseDB::TransferLearnedBinary(from, to, clause) => {
                        let mut the_string = format!("a {} ", Self::key_id(to));
                        the_string.push_str(&self.externalised_clause(clause));
                        the_string.push_str(" 0 l ");
                        the_string.push_str(&Self::resolution_buffer_ids(
                            self.resolution_buffer.pop_front().expect("nri_tl"),
                        ));
                        the_string.push_str(format!("d {} 0\n", Self::key_id(from)).as_str());
                        Some(the_string)
                    }

                    delta::ClauseDB::Learned(key, clause) => {
                        let mut the_string = format!("a {} ", Self::key_id(key));
                        the_string.push_str(&self.externalised_clause(clause));
                        the_string.push_str(" 0 l ");
                        the_string.push_str(&Self::resolution_buffer_ids(
                            self.resolution_buffer.pop_front().expect("nri_l"),
                        ));
                        Some(the_string)
                    }
                    delta::ClauseDB::BinaryFormula(key, clause) => {
                        let mut the_string = format!("o {} ", Self::key_id(key));
                        the_string.push_str(&self.externalised_clause(clause));
                        Some(the_string)
                    }
                    delta::ClauseDB::BinaryResolution(key, clause) => {
                        let mut the_string = format!("a {} ", Self::key_id(key));
                        the_string.push_str(&self.externalised_clause(clause));
                        the_string.push_str(" 0 l ");
                        the_string.push_str(&Self::resolution_buffer_ids(
                            self.resolution_buffer.pop_front().expect("nri_br"),
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
                            self.resolution_buffer.pop_front().expect("nri_rp"),
                        ));
                        Some(the_string)
                    }
                    delta::Level::Pure(literal) => Some(format!(
                        "o {} {}",
                        Self::literal_id(literal),
                        self.externalised_literal(literal)
                    )),
                    delta::Level::BCP(literal) => Some(format!(
                        "a {} {}",
                        Self::literal_id(literal),
                        self.externalised_literal(literal)
                    )),
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
                    let mut the_string = format!(
                        "f {} {}",
                        Self::literal_id(literal),
                        self.externalised_literal(literal)
                    );
                    Some(the_string)
                }
            },

            Dispatch::Parser(_) => None,
            Dispatch::Resolution(_) => None,
            Dispatch::SolveComment(_) => None,
            Dispatch::SolveReport(_) => None,
        };
        if let Some(mut step) = transcription {
            step.push_str(" 0\n");
            self.step_buffer.push(step.to_string())
        }
    }

    pub fn take_resolution(&mut self, buffer: Vec<ClauseKey>) {
        self.resolution_buffer.push_back(buffer)
    }

    pub fn flush(&mut self) {
        for step in &self.step_buffer {
            let _ = self.file.write(step.as_bytes());
        }
        self.step_buffer.clear();
    }
}
