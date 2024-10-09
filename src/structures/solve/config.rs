pub static ACTIVITY_CONFLICT: f32 = 1.0;
pub static DECAY_FACTOR: f32 = 0.95;
pub static DECAY_FREQUENCY: usize = 1;

// Configuration variables
pub static mut GLUE_STRENGTH: usize = 2;
pub static mut SHOW_STATS: bool = false;
pub static mut SHOW_CORE: bool = false;
pub static mut SHOW_ASSIGNMENT: bool = false;
pub static mut EXPLORATION_PRIORITY: ExplorationPriority = ExplorationPriority::Default;
pub static mut STOPPING_CRITERIA: StoppingCriteria = StoppingCriteria::FirstAssertingUIP;
pub static mut RESTARTS_ALLOWED: bool = true;
pub static mut HOBSON_CHOICES: bool = false;
pub static mut TIME_LIMIT: Option<std::time::Duration> = None;

pub static mut REDUCTION_ALLOWED: bool = false;

use crate::structures::solve::Solve;

#[derive(Debug, Clone, Copy)]
pub enum StoppingCriteria {
    FirstAssertingUIP,
    None,
}

#[derive(Debug, Clone)]
pub enum ExplorationPriority {
    Conflict,
    Implication,
    Default,
}

impl Solve {
    pub fn it_is_time_to_reduce(&self) -> bool {
        self.conflicts_since_last_forget >= 256_usize.wrapping_mul(luby(self.restarts + 1))
    }
}

// with help from https://github.com/aimacode/aima-python/blob/master/improving_sat_algorithms.ipynb
fn luby(i: usize) -> usize {
    let mut k = 1;
    loop {
        if i == (1_usize.wrapping_shl(k)) - 1 {
            return 1_usize.wrapping_shl(k - 1);
        } else if (1_usize.wrapping_shl(k - 1)) <= i && i < (1_usize.wrapping_shl(k)) - 1 {
            return luby(i - (1 << (k - 1)) + 1);
        }
        k += 1
    }
}
