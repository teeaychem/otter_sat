use std::{
    borrow::Borrow,
    collections::VecDeque,
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
Parts for recording FRAT proofs

The main struct splits a proof between some file and a buffer which is transfered to the file when flushed.

A few decisions make this a little more delicate than it otherwise could be

- On-the-fly self-subsumption
  + For formulas, specifically,  means it's important to record an origial formula before subsumption is applied
    Rather than do anything complex this is addressed by writing the original formula at the start of a proof.

- Variable renaming
  + … when mixed with 0 as a delimiter in the format requires (i think) translating a clause back to it's DIMACS representation

- Multiple clause databases
  + Requires disambiguating indicies.
    As there are no explicit limits on indicies in the FRAT document, simple ASCII prefixes are used

- Proofs of literals
  - And all the above also apply to when a literal is proven and so adding a clause is skipped completely
 */

pub struct Transcriber {
    path: PathBuf,
    step_buffer: Vec<String>,
    pub resolution_buffer: VecDeque<Vec<ClauseKey>>,
}

impl Transcriber {
    pub fn new(path: PathBuf) -> Self {
        std::fs::File::create(&path);
        Transcriber {
            path,
            resolution_buffer: VecDeque::default(),
            step_buffer: Vec::default(),
        }
    }

    fn key_id(key: ClauseKey) -> String {
        match key {
            ClauseKey::Formula(index) => format!("f_{index}"),
            ClauseKey::Binary(index) => format!("b_{index}"),
            ClauseKey::Learned(index, _) => format!("l_{index}"),
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

    pub fn transcripe(&mut self, dispatch: Dispatch) {
        let mut transcription = match dispatch {
            Dispatch::ClauseStore(store_delta) => {
                // x
                match store_delta {
                    delta::ClauseStore::Deletion(key) => Some(format!("d {}", Self::key_id(key))),
                    delta::ClauseStore::TransferFormula(from, to) => {
                        /*
                        Derive new, delete formula
                         */
                        let mut the_string = format!("a {} ", Self::key_id(to));
                        the_string.push_str("TODO TODO");
                        the_string.push_str(" l ");
                        the_string.push_str(&Self::resolution_buffer_ids(
                            self.resolution_buffer.pop_front().expect("nri_tf"),
                        ));
                        the_string.push_str(format!("d {} 0\n", Self::key_id(from)).as_str());
                        Some(the_string)
                    }
                    delta::ClauseStore::TransferLearned(from, to) => {
                        let mut the_string = format!("a {} ", Self::key_id(to));
                        the_string.push_str("TODO TODO");
                        the_string.push_str(" l ");
                        the_string.push_str(&Self::resolution_buffer_ids(
                            self.resolution_buffer.pop_front().expect("nri_tl"),
                        ));
                        the_string.push_str(format!("d {} 0\n", Self::key_id(from)).as_str());
                        Some(the_string)
                    }

                    delta::ClauseStore::Learned(a, b) => {
                        let mut the_string = String::from("a ");
                        the_string.push_str(&b.as_string());
                        the_string.push_str(" l ");
                        the_string.push_str(&Self::resolution_buffer_ids(
                            self.resolution_buffer.pop_front().expect("nri_l"),
                        ));
                        Some(the_string)
                    }
                    delta::ClauseStore::BinaryFormula(a, b) => {
                        let mut the_string = String::from("o ");
                        the_string.push_str(&b.as_string());
                        Some(the_string)
                    }
                    delta::ClauseStore::BinaryResolution(a, b) => {
                        let mut the_string = String::from("a ");
                        the_string.push_str(&b.as_string());
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
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&self.path)
            .unwrap();
        for step in &self.step_buffer {
            let _ = file.write(step.as_bytes());
        }
        self.step_buffer.clear();
    }
}
