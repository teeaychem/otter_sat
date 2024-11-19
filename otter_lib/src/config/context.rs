use switches::AbalableThings;

use super::{
    dbs::{ClauseDBConfig, VariableDBConfig},
    LubyRepresentation, PolarityLean, RandomChoiceFrequency, StoppingCriteria, VSIDS,
};

#[derive(Clone)]
pub struct Config {
    /// The `u` value to multiply the luby sequence by when determining whether to perform a restart.
    pub luby_u: LubyRepresentation,
    pub luby_reduction_interval: usize,

    /// The chance of choosing assigning positive polarity to a variant when making a choice.
    pub polarity_lean: PolarityLean,

    /// Preprocessing configuration
    pub random_choice_frequency: RandomChoiceFrequency,

    /// Which stopping criteria to use during resolution based analysis
    pub stopping_criteria: StoppingCriteria,
    pub time_limit: Option<std::time::Duration>,

    /// Which VSIDS variant to use during resolution based analysis
    pub vsids_variant: VSIDS,

    pub reduction_interval: usize,

    pub enabled: AbalableThings,
    pub clause_db: ClauseDBConfig,
    pub variable_db: VariableDBConfig,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            luby_u: 128,
            luby_reduction_interval: 2,

            polarity_lean: 0.0,

            random_choice_frequency: 0.0,

            stopping_criteria: StoppingCriteria::FirstUIP,

            time_limit: None,
            vsids_variant: VSIDS::MiniSAT,

            reduction_interval: 50_000,

            enabled: AbalableThings::default(),
            clause_db: ClauseDBConfig::default(),
            variable_db: VariableDBConfig::default(),
        }
    }
}
pub mod switches {
    //! Boolean valued context configurations
    //! When set to true things related to the identifier are enabled.

    #[derive(Clone)]
    pub struct AbalableThings {
        pub preprocessing: bool,
        pub reduction: bool,
        pub restart: bool,
        pub subsumption: bool,
    }

    impl Default for AbalableThings {
        fn default() -> Self {
            AbalableThings {
                preprocessing: false,
                reduction: true,
                restart: true,
                subsumption: true,
            }
        }
    }
}
