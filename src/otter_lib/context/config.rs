pub mod defaults {
    use crate::context::config::StoppingCriteria;
    use crate::context::config::VSIDS;

    pub const LUBY_U: usize = 512;
    pub const GLUE_STRENGTH: usize = 2;
    pub const STOPPING_CRITERIA: StoppingCriteria = StoppingCriteria::FirstUIP;
    pub const VSIDS_VARIANT: VSIDS = VSIDS::MiniSAT;
    pub const RANDOM_CHOICE_FREQUENCY: f64 = 0.0;
    pub const POLARITY_LEAN: f64 = 0.0;
}

#[derive(Debug, Clone)]
pub struct Config {
    pub glue_strength: usize,
    pub preprocessing: bool,
    pub luby_constant: usize,
    pub polarity_lean: f64,
    pub reduction_allowed: bool,
    pub restarts_allowed: bool,
    pub show_core: bool,
    pub show_stats: bool,
    pub show_valuation: bool,
    pub stopping_criteria: StoppingCriteria,
    pub time_limit: Option<std::time::Duration>,
    pub vsids_variant: VSIDS,
    pub activity_conflict: f32,
    pub decay_factor: f32,
    pub decay_frequency: usize,
    pub subsumption: bool,
    pub random_choice_frequency: f64,
    pub tidy_watches: bool,
}

impl Default for Config {
    fn default() -> Self {
        use defaults::*;
        Config {
            glue_strength: GLUE_STRENGTH,
            preprocessing: false,
            luby_constant: LUBY_U,
            polarity_lean: POLARITY_LEAN,
            reduction_allowed: true,
            restarts_allowed: true,
            show_core: false,
            show_stats: true,
            show_valuation: false,
            stopping_criteria: STOPPING_CRITERIA,
            time_limit: None,
            vsids_variant: VSIDS_VARIANT,
            activity_conflict: 1.0,
            decay_factor: 0.95,
            decay_frequency: 1,
            subsumption: false,
            random_choice_frequency: RANDOM_CHOICE_FREQUENCY,
            tidy_watches: false,
        }
    }
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
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

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
#[allow(clippy::upper_case_acronyms)]
pub enum VSIDS {
    MiniSAT,
    Chaff,
}

impl std::fmt::Display for VSIDS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MiniSAT => write!(f, "MiniSAT"),
            Self::Chaff => write!(f, "Chaff"),
        }
    }
}
