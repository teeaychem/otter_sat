use std::{
    borrow::Borrow,
    collections::VecDeque,
    io::{BufWriter, Write},
    path::PathBuf,
};

use crate::{
    config::Config,
    context::{
        delta::{self, ClauseStoreDelta, Dispatch},
        stores::{variable::VariableStore, ClauseKey},
        unique_id::UniqueIdentifier,
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

// A step in a FRAT proof
pub struct FRATStep {
    str: String,
}

// An FRAT proof, split between whats been written to the path and what's in the buffer
pub struct FRATProof {
    buffer: Vec<FRATStep>,
}

impl FRATProof {
    pub fn new() -> Self {
        FRATProof {
            // TODO: Argument path
            buffer: Vec::default(),
        }
    }

    pub fn record(&mut self, step: FRATStep) {
        self.buffer.push(step)
    }

    pub fn flush(&mut self, config: &Config) {
        if let Some(path) = &config.io.frat_path {
            let file = std::fs::OpenOptions::new().append(true).open(path).unwrap();
            let mut writer = BufWriter::new(file);
            for step in &self.buffer {
                let _ = writer.write_all(step.as_bytes());
            }
            self.buffer.clear()
        }
    }
}

impl FRATStep {
    pub fn key_id(key: &ClauseKey) -> String {
        match key {
            ClauseKey::Formula(index) => format!("f_{index}"),
            ClauseKey::Binary(index) => format!("b_{index}"),
            ClauseKey::Learned(index, _) => format!("l_{index}"),
        }
    }

    fn literal_id<L: Borrow<impl LiteralTrait>>(literal: L) -> String {
        format!("l_{}", literal.borrow().index())
    }
}

impl FRATStep {
    pub fn original_clause(key: ClauseKey, clause: &[Literal], variables: &VariableStore) -> Self {
        let mut the_string = String::from("o ");
        the_string.push_str(&FRATStep::key_id(&key));
        the_string.push(' ');
        the_string.push_str(clause.as_dimacs(variables, false).as_str());

        the_string.push_str("0\n");
        FRATStep { str: the_string }
    }

    pub fn original_literal<L: Borrow<impl LiteralTrait>>(
        literal: L,
        variables: &VariableStore,
    ) -> Self {
        let mut the_string = String::from("o ");
        let literal_copy: Literal = literal.borrow().canonical();
        the_string.push_str(&FRATStep::literal_id(literal_copy));
        the_string.push(' ');
        match literal_copy.polarity() {
            true => the_string.push_str(variables.external_name(literal.borrow().index())),
            false => the_string.push_str(
                format!("-{}", variables.external_name(literal.borrow().index())).as_str(),
            ),
        };

        the_string.push_str("0\n");
        FRATStep { str: the_string }
    }

    pub fn deletion(index: usize) -> Self {
        FRATStep {
            str: format!("d {} 0\n", index),
        }
    }

    // An addition step of a learnt clause
    pub fn learnt_clause(
        add_key: ClauseKey,
        clause: &Vec<Literal>,
        resolution_keys: &Vec<ClauseKey>,
        variables: &VariableStore,
    ) -> Self {
        let mut the_string = String::from("a ");
        the_string.push_str(&FRATStep::key_id(&add_key));
        the_string.push_str(clause.as_dimacs(variables, false).as_str());

        if !resolution_keys.is_empty() {
            the_string.push_str(" l ");
            for antecedent in resolution_keys {
                the_string.push_str(format!("{} ", antecedent.index()).as_str());
            }
        }

        the_string.push_str("0\n");
        FRATStep { str: the_string }
    }

    pub fn learnt_literal<L: Borrow<impl LiteralTrait>>(
        literal: L,
        resolution_keys: &Vec<ClauseKey>,
        variables: &VariableStore,
    ) -> Self {
        let mut the_string = String::from("a ");
        let literal_copy: Literal = literal.borrow().canonical();
        the_string.push_str(&FRATStep::literal_id(literal_copy));
        the_string.push(' ');
        match literal_copy.polarity() {
            true => the_string.push_str(variables.external_name(literal.borrow().index())),
            false => the_string.push_str(
                format!("-{}", variables.external_name(literal.borrow().index())).as_str(),
            ),
        };

        if !resolution_keys.is_empty() {
            the_string.push_str(" l ");
            for antecedent in resolution_keys {
                the_string.push_str(format!("{} ", antecedent.index()).as_str());
            }
        }

        the_string.push_str("0\n");
        FRATStep { str: the_string }
    }

    pub fn finalise(key: ClauseKey, clause: &[Literal], variables: &VariableStore) -> Self {
        let mut the_string = String::from("f ");
        the_string.push_str(&FRATStep::key_id(&key));
        the_string.push(' ');
        the_string.push_str(clause.as_dimacs(variables, false).as_str());

        the_string.push_str("0\n");
        FRATStep { str: the_string }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.str.as_bytes()
    }
}

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

    pub fn transcripe(&mut self, dispatch: Dispatch) {
        let mut transcription = match dispatch {
            Dispatch::ClauseStore(store_delta) => {
                // x
                match store_delta {
                    ClauseStoreDelta::Deletion(key) => Some(format!("d {}", Self::key_id(key))),
                    ClauseStoreDelta::TransferFormula(from, to) => {
                        /*
                        Derive new, delete formula
                         */
                        let mut the_string = String::from(format!("a {} ", Self::key_id(to)));
                        the_string.push_str("TODO TODO");
                        the_string.push_str(" l ");
                        the_string.push_str(
                            format!("{:?}", self.resolution_buffer.pop_front().expect("nri_tf"))
                                .as_str(),
                        );
                        the_string.push_str(format!("d {} 0\n", Self::key_id(from)).as_str());
                        Some(the_string)
                    }
                    ClauseStoreDelta::TransferLearned(from, to) => {
                        let mut the_string = String::from(format!("a {} ", Self::key_id(to)));
                        the_string.push_str("TODO TODO");
                        the_string.push_str(" l ");
                        the_string.push_str(
                            format!("{:?}", self.resolution_buffer.pop_front().expect("nri_tl"))
                                .as_str(),
                        );
                        the_string.push_str(format!("d {} 0\n", Self::key_id(from)).as_str());
                        Some(the_string)
                    }

                    ClauseStoreDelta::Learned(a, b) => {
                        let mut the_string = String::from("a ");
                        the_string.push_str(&b.as_string());
                        the_string.push_str(" l ");
                        the_string.push_str(
                            format!("{:?}", self.resolution_buffer.pop_front().expect("nri_l"))
                                .as_str(),
                        );
                        Some(the_string)
                    }
                    ClauseStoreDelta::BinaryFormula(a, b) => {
                        let mut the_string = String::from("o ");
                        the_string.push_str(&b.as_string());
                        Some(the_string)
                    }
                    ClauseStoreDelta::BinaryResolution(a, b) => {
                        let mut the_string = String::from("a ");
                        the_string.push_str(&b.as_string());
                        the_string.push_str(" l ");
                        the_string.push_str(
                            format!("{:?}", self.resolution_buffer.pop_front().expect("nri_br"))
                                .as_str(),
                        );
                        Some(the_string)
                    }
                    _ => None,
                }
            }
            Dispatch::Level(level_delta) => {
                //
                match level_delta {
                    delta::LevelDelta::FormulaAssumption(literal) => {
                        Some(format!("o {} 0", Self::literal_id(literal)))
                    }
                    delta::LevelDelta::ResolutionProof(literal) => {
                        let mut the_string = String::from("a ");
                        the_string.push_str(format!("{}", Self::literal_id(literal)).as_str());
                        the_string.push_str(" l ");
                        the_string.push_str(
                            format!("{:?}", self.resolution_buffer.pop_front().expect("nri_rp"))
                                .as_str(),
                        );
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
