pub mod defaults;

pub type VariableActivity = f64;
pub type ClauseActivity = f64;
pub type DecayFrequency = u8;
pub type GlueStrength = u8;
pub type LubyConstant = crate::generic::luby::LubyType;
pub type PolarityLean = f64;
pub type RandomChoiceFrequency = f64;

#[derive(Debug, Clone)]
pub struct Config {
    pub activity_conflict: VariableActivity,
    pub activity_max: VariableActivity,
    pub variable_decay: VariableActivity,
    pub clause_decay: ClauseActivity,
    pub glue_strength: GlueStrength,
    pub luby_constant: LubyConstant,
    pub polarity_lean: PolarityLean,
    pub preprocessing: bool,
    pub random_choice_frequency: RandomChoiceFrequency,
    pub reduction_allowed: bool,
    pub restarts_allowed: bool,
    pub stopping_criteria: StoppingCriteria,
    pub subsumption: bool,
    pub time_limit: Option<std::time::Duration>,
    pub vsids_variant: VSIDS,
    pub reduction_interval: usize,
}

impl Default for Config {
    fn default() -> Self {
        use defaults::*;
        Config {
            activity_conflict: VARIABLE_BUMP,
            activity_max: (2.0 as VariableActivity).powi(512), // 1e150
            variable_decay: VARIABLE_DECAY_FACTOR,
            clause_decay: CLAUSE_DECAY_FACTOR,
            glue_strength: GLUE_STRENGTH,
            luby_constant: LUBY_U,
            polarity_lean: POLARITY_LEAN,
            preprocessing: false,
            random_choice_frequency: RANDOM_CHOICE_FREQUENCY,
            reduction_allowed: true,
            restarts_allowed: true,
            stopping_criteria: STOPPING_CRITERIA,
            subsumption: true,
            time_limit: None,
            vsids_variant: VSIDS_VARIANT,
            reduction_interval: REDUCTION_INTERVAL,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoppingCriteria {
    FirstUIP,
    None,
}

impl std::fmt::Display for StoppingCriteria {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FirstUIP => write!(f, "FirstUIP"),
            Self::None => write!(f, "None"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(clippy::upper_case_acronyms)]
pub enum VSIDS {
    Chaff,
    MiniSAT,
}

impl std::fmt::Display for VSIDS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Chaff => write!(f, "Chaff"),
            Self::MiniSAT => write!(f, "MiniSAT"),
        }
    }
}
