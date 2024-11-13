use std::ops::{Shl, Shr};

use crate::{
    db::keys::ClauseKey,
    structures::literal::{Literal, LiteralTrait},
};

/*
Unique representations of indexed things.
Primarily for FRAT proofs.

The idea is to store encode an internal identifier to a u64 with some flags to allow decoding.
The layout is:

[Index: u32, Disambiguation: u8, Action: u8, Relative: u16]

- Bytes 0..4 are the internal id
  The use of u32s is standard for all internal indicies
- Byte 4 is used for disambiguation
- Byte 5 is used for action
- Bytes 6 .. are relative information
  For literals, these store the polarity, for clause keys the token, if it exists

Things are set this way so unique ids can easily be cast to the encoded index
 */

/*
The primary motivation is to allow external recording of what's happened.
I think, also, to help with this it might be worthwhile to `schedule` the removal of clauses
Though, this isn't too pressing

For now, keeping a record of this information is sufficient to build an unsat core, external to the core of the solver

This doesn't help much with proofs, though…
 */

pub(crate) type UniqueIdentifier = u64;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypeFlag {
    Variable,
    Literal,
    Binary,
    Formula,
    Learned,
}

impl TypeFlag {
    fn as_u8(&self) -> u8 {
        match self {
            Self::Variable => 1,
            Self::Literal => 2,
            Self::Binary => 3,
            Self::Formula => 4,
            Self::Learned => 5,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionFlag {
    None,
    Addition,
    Deletion,
}

pub trait UniqueId {
    fn unique_id(self) -> UniqueIdentifier;
}

fn write_index(unique_id: &mut UniqueIdentifier, index: u32) {
    *unique_id += index as u64
}

fn write_token(unique_id: &mut UniqueIdentifier, token: u16) {
    *unique_id += (token as u64).shl(8 * 6)
}

fn write_type_flag(unique_id: &mut UniqueIdentifier, flag: TypeFlag) {
    *unique_id += (flag.as_u8() as u64).shl(8 * 4)
}

fn write_polarity(unique_id: &mut UniqueIdentifier, polarity: bool) {
    match polarity {
        true => *unique_id += 1_u64.shl(8 * 6),
        false => *unique_id += 2_u64.shl(8 * 6),
    }
}

impl UniqueId for ClauseKey {
    fn unique_id(self) -> UniqueIdentifier {
        let mut unique_id = UniqueIdentifier::default();
        match self {
            Self::Formula(index) => {
                write_index(&mut unique_id, index);
                write_type_flag(&mut unique_id, TypeFlag::Formula);
            }
            Self::Binary(index) => {
                write_index(&mut unique_id, index);
                write_type_flag(&mut unique_id, TypeFlag::Binary);
            }
            Self::Learned(index, token) => {
                write_index(&mut unique_id, index);
                write_token(&mut unique_id, token);
                write_type_flag(&mut unique_id, TypeFlag::Learned);
            }
        };
        unique_id
    }
}

impl UniqueId for Literal {
    fn unique_id(self) -> UniqueIdentifier {
        let mut unique_id = UniqueIdentifier::default();
        write_index(&mut unique_id, self.v_id());
        write_polarity(&mut unique_id, self.polarity());
        write_type_flag(&mut unique_id, TypeFlag::Literal);
        unique_id
    }
}

pub fn unique_index(unique_id: UniqueIdentifier) -> u32 {
    unique_id as u32
}

pub fn unique_relative(unique_id: UniqueIdentifier) -> u16 {
    unique_id.shr(8 * 6) as u16
}

pub fn unique_meta(unique_id: UniqueIdentifier) -> TypeFlag {
    let token = unique_id.shr(8 * 4) as u8;
    match token {
        1 => TypeFlag::Variable,
        2 => TypeFlag::Literal,
        3 => TypeFlag::Binary,
        4 => TypeFlag::Formula,
        5 => TypeFlag::Learned,
        _ => panic!("{token}"),
    }
}

pub fn unique_clause_key(unique_id: UniqueIdentifier) -> Option<ClauseKey> {
    let index = unique_index(unique_id);

    match unique_meta(unique_id) {
        TypeFlag::Binary => Some(ClauseKey::Binary(index)),
        TypeFlag::Formula => Some(ClauseKey::Formula(index)),
        TypeFlag::Learned => {
            let token = unique_relative(unique_id);
            Some(ClauseKey::Learned(index, token))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn unique_literal_test() {
        let the_literal = Literal::new(0, true);
        let unique = the_literal.unique_id();
        let expectation_bytes: [u8; 8] = [0, 0, 0, 0, 2, 0, 1, 0];
        let expectation = u64::from_le_bytes(expectation_bytes);
        assert_eq!(unique, expectation);
        assert_eq!(unique as u32, the_literal.v_id());

        let the_literal = Literal::new(u32::MAX, false);
        let unique = the_literal.unique_id();
        let expectation_bytes: [u8; 8] = [u8::MAX, u8::MAX, u8::MAX, u8::MAX, 2, 0, 2, 0];
        let expectation = u64::from_le_bytes(expectation_bytes);
        assert_eq!(unique, expectation);
        assert_eq!(unique as u32, the_literal.v_id());

        let the_literal = Literal::new(u32::MAX - 1, false);
        let unique = the_literal.unique_id();
        let expectation_bytes: [u8; 8] = [u8::MAX, u8::MAX, u8::MAX, u8::MAX, 2, 0, 2, 0];
        let expectation = u64::from_le_bytes(expectation_bytes);
        assert_ne!(unique, expectation);
        assert_eq!(unique as u32, the_literal.v_id());
    }

    #[test]
    fn unique_clause_key_test() {
        let index: u32 = 5353;
        let token: u16 = 643;
        let clause_key = ClauseKey::Learned(index, token);
        let unique_id = clause_key.unique_id();

        println!("{:?}", unique_id.to_be_bytes());

        let retreived_index = unique_index(unique_id);
        assert_eq!(index, retreived_index);
        let retreived_token = unique_relative(unique_id);
        assert_eq!(token, retreived_token);
        let meta = unique_meta(unique_id);
        assert_eq!(meta, TypeFlag::Learned);
        assert_eq!(clause_key, unique_clause_key(unique_id).unwrap());
    }
}
