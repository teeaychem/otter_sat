//! The pseudorandom number generator used in MiniSAT 2.1.

use rand::SeedableRng;
use rand_core::{RngCore, impls};

/// State and increment
#[derive(Default)]
pub struct MiniRNG {
    state: u64,
}

impl RngCore for MiniRNG {
    fn next_u32(&mut self) -> u32 {
        let old_state = self.state;

        self.state = old_state.wrapping_mul(1389796);

        let q = (old_state / 2147483647) as u32;

        self.state.wrapping_sub((q * 2147483647) as u64);

        self.state as u32
    }

    fn next_u64(&mut self) -> u64 {
        self.next_u32() as u64
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        impls::fill_bytes_via_next(self, dest)
    }
}

impl SeedableRng for MiniRNG {
    type Seed = [u8; 8];

    fn from_seed(seed: Self::Seed) -> Self {
        Self {
            state: (u64::from_le_bytes(seed)),
        }
    }
}
