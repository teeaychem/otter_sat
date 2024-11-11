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
        literal::{Literal, LiteralTrait},
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

    fn key_id(key: ClauseKey) -> String {
        match key {
            ClauseKey::Formula(index) => format!("10{index}"),
            ClauseKey::Binary(index) => format!("20{index}"),
            ClauseKey::Learned(index, _) => format!("30{index}"),
        }
    }

    fn literal_id(literal: Literal) -> String {
        format!("l_{}", literal.index())
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
            match &self.variable_map[literal.index()] {
                Some(ext) => match literal.polarity() {
                    true => the_string.push_str(format!("{ext} ").as_str()),
                    false => the_string.push_str(format!("-{ext} ").as_str()),
                },
                None => panic!("Missing external string for {literal}"),
            }
        }
        the_string.pop();
        the_string
    }

    pub fn transcripe(&mut self, dispatch: Dispatch) {
        let mut transcription = match dispatch {
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
            },
            Dispatch::ClauseDB(store_delta) => {
                // x
                match store_delta {
                    delta::ClauseStore::Deletion(key) => Some(format!("d {}", Self::key_id(key))),

                    delta::ClauseStore::TransferFormulaBinary(from, to, clause) => {
                        /*
                        Derive new, delete formula
                         */
                        let mut the_string = format!("a {} ", Self::key_id(to));
                        the_string.push_str(&self.externalised_clause(clause));
                        the_string.push_str(" l ");
                        the_string.push_str(&Self::resolution_buffer_ids(
                            self.resolution_buffer.pop_front().expect("nri_tf"),
                        ));
                        the_string.push_str(format!("d {} 0\n", Self::key_id(from)).as_str());
                        Some(the_string)
                    }
                    delta::ClauseStore::TransferLearnedBinary(from, to, clause) => {
                        let mut the_string = format!("a {} ", Self::key_id(to));
                        the_string.push_str(&self.externalised_clause(clause));
                        the_string.push_str(" l ");
                        the_string.push_str(&Self::resolution_buffer_ids(
                            self.resolution_buffer.pop_front().expect("nri_tl"),
                        ));
                        the_string.push_str(format!("d {} 0\n", Self::key_id(from)).as_str());
                        Some(the_string)
                    }

                    delta::ClauseStore::Learned(key, clause) => {
                        let mut the_string = String::from("a ");
                        the_string.push_str(&self.externalised_clause(clause));
                        the_string.push_str(" l ");
                        the_string.push_str(&Self::resolution_buffer_ids(
                            self.resolution_buffer.pop_front().expect("nri_l"),
                        ));
                        Some(the_string)
                    }
                    delta::ClauseStore::BinaryFormula(key, clause) => {
                        let mut the_string = String::from("o ");
                        the_string.push_str(&self.externalised_clause(clause));
                        Some(the_string)
                    }
                    delta::ClauseStore::BinaryResolution(key, clause) => {
                        let mut the_string = String::from("a ");
                        the_string.push_str(&self.externalised_clause(clause));
                        the_string.push_str(" l ");
                        the_string.push_str(&Self::resolution_buffer_ids(
                            self.resolution_buffer.pop_front().expect("nri_br"),
                        ));
                        Some(the_string)
                    }
                    _ => None,
                }
            }
            Dispatch::Level(level_delta) => {
                //
                match level_delta {
                    delta::Level::FormulaAssumption(literal) => {
                        Some(format!("o {} 0", Self::literal_id(literal)))
                    }
                    delta::Level::ResolutionProof(literal) => {
                        // TODO: Fix literal
                        let mut the_string =
                            format!("a {} {} ", Self::literal_id(literal), literal);
                        the_string.push_str(" l ");
                        the_string.push_str(&Self::resolution_buffer_ids(
                            self.resolution_buffer.pop_front().expect("nri_rp"),
                        ));
                        Some(the_string)
                    }
                    _ => None,
                }
            }
            _ => None,
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
