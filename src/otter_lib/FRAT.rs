use std::{
    borrow::Borrow,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use crate::{
    context::{
        stores::{variable::VariableStore, ClauseKey},
        unique_id::UniqueIdentifier,
    },
    structures::{
        self,
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
    path: PathBuf,
    buffer: Vec<FRATStep>,
}

impl FRATProof {
    pub fn new() -> Self {
        FRATProof {
            // TODO: Argument path
            path: PathBuf::from("temp.txt"),
            buffer: Vec::default(),
        }
    }

    pub fn record(&mut self, step: FRATStep) {
        self.buffer.push(step)
    }

    pub fn flush(&mut self) {
        let file = std::fs::OpenOptions::new()
            .append(true)
            .open(&self.path)
            .unwrap();
        let mut writer = BufWriter::new(file);
        for step in &self.buffer {
            let _ = writer.write_all(step.as_bytes());
        }
        self.buffer.clear()
    }
}

impl FRATStep {
    fn key_id(key: &ClauseKey) -> String {
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
    pub fn original(key: ClauseKey, clause: &[Literal], variables: &VariableStore) -> Self {
        let mut the_string = String::from("o ");
        the_string.push_str(&FRATStep::key_id(&key));
        the_string.push(' ');
        the_string.push_str(clause.as_dimacs(variables, false).as_str());

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
        resolution_keys: &Vec<UniqueIdentifier>,
        variables: &VariableStore,
    ) -> Self {
        let mut the_string = String::from("a ");
        the_string.push_str(&FRATStep::key_id(&add_key));
        the_string.push_str(clause.as_dimacs(variables, false).as_str());

        if !resolution_keys.is_empty() {
            the_string.push_str(" l ");
            for antecedent in resolution_keys {
                the_string.push_str(format!("{} ", *antecedent as u32).as_str());
            }
        }

        the_string.push_str("0\n");
        FRATStep { str: the_string }
    }

    pub fn learnt_literal<L: Borrow<impl LiteralTrait>>(
        literal: L,
        resolution_keys: &Vec<UniqueIdentifier>,
        variables: &VariableStore,
    ) -> Self {
        let mut the_string = String::from("a ");
        let literal_copy: Literal = literal.borrow().canonical();
        the_string.push_str(&FRATStep::literal_id(literal_copy));
        the_string.push(' ');
        the_string.push_str(variables.external_name(literal.borrow().index()));

        if !resolution_keys.is_empty() {
            the_string.push_str(" l ");
            for antecedent in resolution_keys {
                the_string.push_str(format!("{} ", *antecedent as u32).as_str());
            }
        }

        the_string.push_str("0\n");
        FRATStep { str: the_string }
    }

    // A relocation step
    pub fn relocation(from: ClauseKey, to: ClauseKey) -> Self {
        FRATStep {
            str: format!(
                "r {} {} 0\n",
                FRATStep::key_id(&from),
                FRATStep::key_id(&to)
            ),
        }
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
