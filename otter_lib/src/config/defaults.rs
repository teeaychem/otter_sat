use crate::config::{self};

pub const VARIABLE_BUMP: config::Activity = 1.0;
pub const VARIABLE_DECAY_FACTOR: config::Activity = 50.0;

pub const CLAUSE_BUMP: config::Activity = 1.0;
pub const CLAUSE_DECAY_FACTOR: config::Activity = 20.0;

pub const REDUCTION_INTERVAL: usize = 2;

pub const GLUE_STRENGTH: config::GlueStrength = 2;
pub const LUBY_U: config::LubyConstant = 128;
pub const POLARITY_LEAN: config::PolarityLean = 0.0;
pub const RANDOM_CHOICE_FREQUENCY: config::RandomChoiceFrequency = 0.0;
pub const STOPPING_CRITERIA: config::StoppingCriteria = config::StoppingCriteria::FirstUIP;
pub const VSIDS_VARIANT: config::VSIDS = config::VSIDS::MiniSAT;

pub const DEFAULT_VARIABLE_COUNT: usize = 1024;

pub const RNG_SEED: u64 = 0;

pub const INTER_REDUCTION_INTERVAL: usize = 50_000;
