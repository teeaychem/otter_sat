use crate::config::{self};

pub const VARIABLE_BUMP: config::VariableActivity = 1.0;
pub const VARIABLE_DECAY_FACTOR: config::VariableActivity = 50.0;

pub const CLAUSE_BUMP: config::ClauseActivity = 1.0;
pub const CLAUSE_DECAY_FACTOR: config::ClauseActivity = 100.0;

pub const REUCE_ON_RESTARTS: usize = 5 * 10_usize.pow(3);

pub const DECAY_FREQUENCY: config::DecayFrequency = 1;
pub const GLUE_STRENGTH: config::GlueStrength = 3;
pub const LUBY_U: config::LubyConstant = 128;
pub const POLARITY_LEAN: config::PolarityLean = 0.0;
pub const RANDOM_CHOICE_FREQUENCY: config::RandomChoiceFrequency = 0.0;
pub const STOPPING_CRITERIA: config::StoppingCriteria = config::StoppingCriteria::FirstUIP;
pub const VSIDS_VARIANT: config::VSIDS = config::VSIDS::MiniSAT;

pub const DEFAULT_ACTIVITY: config::VariableActivity = 0.0;
pub const DEFAULT_VARIABLE_COUNT: usize = 1024;

pub const RNG_SEED: u64 = 0;
