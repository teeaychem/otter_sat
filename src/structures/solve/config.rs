use crate::structures::solve::Solve;
use serde::Serialize;

pub static ACTIVITY_CONFLICT: f32 = 1.0;
pub static DECAY_FACTOR: f32 = 0.95;
pub static DECAY_FREQUENCY: usize = 1;

// Configuration variables
pub static mut EXPLORATION_PRIORITY: ExplorationPriority = ExplorationPriority::Default;
pub static mut GLUE_STRENGTH: usize = 2;
pub static mut HOBSON_CHOICES: bool = false;
pub static mut LUBY_CONSTANT: usize = 512;
pub static mut POLARITY_LEAN: f64 = 0.5;
pub static mut REDUCTION_ALLOWED: bool = false;
pub static mut RESTARTS_ALLOWED: bool = true;
pub static mut SHOW_CORE: bool = false;
pub static mut SHOW_STATS: bool = false;
pub static mut SHOW_VALUATION: bool = false;
pub static mut STOPPING_CRITERIA: StoppingCriteria = StoppingCriteria::FirstUIP;
pub static mut TIME_LIMIT: Option<std::time::Duration> = None;
pub static mut VSIDS_VARIANT: VSIDS = VSIDS::MiniSAT;

#[derive(Debug, Clone, Copy, Default, Serialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum StoppingCriteria {
    #[default]
    /// Resolve until the first unique implication point
    FirstUIP,
    /// Resolve on each clause used to derive the conflict
    None,
}

#[derive(Debug, Clone, Copy, Default, Serialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[allow(clippy::upper_case_acronyms)]
pub enum VSIDS {
    #[default]
    /// Bump the activity of all variables in the a learnt clause
    MiniSAT,
    /// Bump the activity involved when using resolution to learn a clause
    Chaff,
}

#[derive(Debug, Clone, Copy, Default, Serialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum ExplorationPriority {
    Conflict,
    Implication,
    #[default]
    Default,
}

impl Solve {
    pub fn it_is_time_to_reduce(&self) -> bool {
        self.conflicts_since_last_forget
            >= unsafe { LUBY_CONSTANT }.wrapping_mul(luby(self.restarts + 1))
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
        k += 1;
    }
}

impl std::fmt::Display for VSIDS {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::MiniSAT => write!(f, "MiniSAT"),
            Self::Chaff => write!(f, "Chaff"),
        }
    }
}
