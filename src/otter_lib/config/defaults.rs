use crate::config::{self};

pub const ACTIVITY_CONFLICT: config::ActivityType = 1.0;
pub const DECAY_FACTOR: config::ActivityType = 50.0;
pub const DECAY_FREQUENCY: config::DecayFrequency = 1;
pub const GLUE_STRENGTH: config::GlueStrength = 2;
pub const LUBY_U: config::LubyConstant = 512;
pub const POLARITY_LEAN: config::PolarityLean = 0.0;
pub const RANDOM_CHOICE_FREQUENCY: config::RandomChoiceFrequency = 0.0;
pub const STOPPING_CRITERIA: config::StoppingCriteria = config::StoppingCriteria::FirstUIP;
pub const VSIDS_VARIANT: config::VSIDS = config::VSIDS::MiniSAT;

pub const DEFAULT_ACTIVITY: config::ActivityType = 0.0;
pub const DEFAULT_VARIABLE_COUNT: usize = 1024;

pub const RNG_SEED: u64 = 0;
