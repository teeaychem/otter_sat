pub mod defaults;

pub type ActivityType = f64;
pub type DecayFrequency = usize;
pub type GlueStrength = u32;
pub type LubyConstant = usize;
pub type PolarityLean = f64;
pub type RandomChoiceFrequency = f64;

#[derive(Debug, Clone)]
pub struct Config {
    pub activity_conflict: ActivityType,
    pub activity_max: ActivityType,
    pub decay_factor: ActivityType,
    pub decay_frequency: DecayFrequency,
    pub glue_strength: GlueStrength,
    pub luby_constant: LubyConstant,
    pub polarity_lean: PolarityLean,
    pub preprocessing: bool,
    pub random_choice_frequency: RandomChoiceFrequency,
    pub reduction_allowed: bool,
    pub restarts_allowed: bool,
    pub show_core: bool,
    pub show_stats: bool,
    pub show_valuation: bool,
    pub stopping_criteria: StoppingCriteria,
    pub subsumption: bool,
    pub consequence_criteria: ConsequenceCriteria,
    pub time_limit: Option<std::time::Duration>,
    pub vsids_variant: VSIDS,
}

impl Default for Config {
    fn default() -> Self {
        use defaults::*;
        Config {
            activity_conflict: ACTIVITY_CONFLICT,
            activity_max: (2.0 as ActivityType).powi(512), // 1e150
            decay_factor: DECAY_FACTOR,
            decay_frequency: DECAY_FREQUENCY,
            glue_strength: GLUE_STRENGTH,
            luby_constant: LUBY_U,
            polarity_lean: POLARITY_LEAN,
            preprocessing: false,
            random_choice_frequency: RANDOM_CHOICE_FREQUENCY,
            reduction_allowed: true,
            restarts_allowed: true,
            show_core: false,
            show_stats: false,
            show_valuation: false,
            stopping_criteria: STOPPING_CRITERIA,
            subsumption: true,
            consequence_criteria: CONSEQUENCE_CRITERA,
            time_limit: None,
            vsids_variant: VSIDS_VARIANT,
        }
    }
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
pub enum ConsequenceCriteria {
    Tidy,
    Fresh,
    Messy,
}

impl std::fmt::Display for ConsequenceCriteria {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tidy => write!(f, "Tidy"),
            Self::Fresh => write!(f, "Fresh"),
            Self::Messy => write!(f, "Messy"),
        }
    }
}
