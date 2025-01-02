//! A simple pseudorandom number generator.
//!
//! Specifically, a translation of the *really* minimal C PCG32 implementation from <https://www.pcg-random.org/> implemented to satisfy the [Random](std::random) trait.[^note]
//!
//! PCG(32) was chosen as the default source of (pseudo)random numbers as it is simple, fast, and has some nice supporting documentation.
//!
//! Each [context](crate::context) stores a source of rng as a [counter](crate::context::Counters).
//! The counter structure is paramaterised to anything which satisfies the [Random](std::random) trait.
//! Though to keep the rest of the library straightforward the rng is fixed in the [context](crate::context) as [MinimalPCG32].
//! Still, revising or paramaterising the context is all that's needed for a different source of rng.
//!
//! [^note]: At the time of writing, the C implementation is at the top of the [download page](https://www.pcg-random.org/download.html).

use rand::SeedableRng;
use rand_core::{impls, Error, RngCore};

/// State and increment
#[derive(Default)]
pub struct MinimalPCG32 {
    state: u64,
    inc: u64,
}

impl RngCore for MinimalPCG32 {
    fn next_u32(&mut self) -> u32 {
        let old_state = self.state;

        self.state = old_state
            .wrapping_mul(6364136223846793005_u64)
            .wrapping_add(self.inc);

        let xorshifted = ((old_state >> 18) ^ old_state) >> 27;
        let rot = (old_state >> 59) as u32;
        xorshifted.rotate_right(rot) as u32
    }

    fn next_u64(&mut self) -> u64 {
        self.next_u32() as u64
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        impls::fill_bytes_via_next(self, dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

impl SeedableRng for MinimalPCG32 {
    type Seed = [u8; 8];

    fn from_seed(seed: Self::Seed) -> Self {
        /// This increment is entirely unmotivated.
        const INCREMENT: u64 = 3215534235932367344;
        Self {
            state: (u64::from_le_bytes(seed)).wrapping_add(INCREMENT),
            inc: INCREMENT,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // TODO: test…
    fn random() {
        let mut x = MinimalPCG32::from_seed(2u64.to_le_bytes());
        for _ in 0..10 {
            println!("{}", x.next_u64())
        }
    }
}
