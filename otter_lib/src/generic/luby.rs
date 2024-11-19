// previously with help from https://github.com/aimacode/aima-python/blob/master/improving_sat_algorithms.ipynb
// now with help from https://gitlab.com/chaotic-evil/luby and https://doc.rust-lang.org/rust-by-example/trait/iter.html
//! An iterator to generate the luby sequence.
//!
//! See <https://oeis.org/A182105> for details.

use crate::config::LubyRepresentation;

pub struct Luby {
    curr: LubyRepresentation,
    next: LubyRepresentation,
}

// perhaps idiosyncratic but set the default iterator to be on the first element of the sequence
// this, while allowing different initialisers
impl Default for Luby {
    fn default() -> Self {
        let mut luby = Luby { curr: 0, next: 0 };
        luby.next();
        luby
    }
}

impl Iterator for Luby {
    type Item = LubyRepresentation;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr & self.curr.wrapping_neg() == self.next {
            self.curr = self.curr.checked_add(1)?;
            self.next = 1;
        } else {
            self.next = self.next.checked_add(self.next)?;
        }

        Some(self.next)
    }
}

impl Luby {
    pub fn current(&self) -> LubyRepresentation {
        self.curr
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // https://oeis.org/A182105
    const LUBY_SLICE: &[LubyRepresentation] = &[
        1, 1, 2, 1, 1, 2, 4, 1, 1, 2, 1, 1, 2, 4, 8, 1, 1, 2, 1, 1, 2, 4, 1, 1, 2, 1, 1, 2, 4, 8,
        16, 1, 1, 2, 1, 1, 2, 4, 1, 1, 2, 1, 1, 2, 4, 8, 1, 1, 2, 1, 1, 2, 4, 1, 1, 2, 1, 1, 2, 4,
        8, 16, 32, 1, 1, 2, 1, 1, 2, 4, 1, 1, 2, 1, 1, 2, 4, 8, 1, 1, 2, 1, 1, 2, 4, 1, 1, 2, 1, 1,
        2, 4, 8, 16, 1, 1, 2, 1, 1, 2, 4, 1, 1, 2, 1, 1, 2, 4, 8,
    ];

    #[test]
    fn luby() {
        let mut l = Luby { curr: 0, next: 0 };
        for known_value in LUBY_SLICE {
            let next = l.next().unwrap();
            assert_eq!(next, *known_value)
        }
    }

    #[ignore] // unless set to u8, etc.
    #[test]
    fn exhaust() {
        let l = Luby { curr: 0, next: 0 };
        for _ in l {}
    }
}
