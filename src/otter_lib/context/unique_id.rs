use std::ops::{Shl, Shr};

use crate::structures::literal::{Literal, LiteralTrait};

use super::stores::ClauseKey;

/*
Unique representations of indexed things.
Primarily for FRAT proofs.

The idea is to store encode an internal identifier to a u64 with some flags to allow decoding.
The layout is:

[Index: u32, Relative: u16, Flags: u16]

- Bytes 0..4 are the internal id
  The use of u32s is standard for all internal indicies
- Bytes 4 and 5 are used for flags
  For example, to determine whether the remaining bytes should be interpreted with respect to a literal, clause key, etc.
- Bytes 6 .. are relative information
  For literals, these store the polarity, for clause keys the token, if it exists

Things are set this way so unique ids can easily be cast to the encoded index
 */

pub type UniqueIdentifier = u64;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
enum TypeFlag {
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
}
