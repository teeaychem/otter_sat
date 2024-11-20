//! Configuration for a context.

use switches::AbalableThings;

use super::{
    dbs::{ClauseDBConfig, VariableDBConfig},
    LubyRepresentation, PolarityLean, RandomChoiceFrequency, ReductionScheduler, StoppingCriteria,
    VSIDS,
};

#[derive(Clone, Debug)]
pub struct Config {
    /// The `u` value to multiply the luby sequence by when determining whether to perform a restart.
    pub luby_u: LubyRepresentation,

    /// The probability of assigning positive polarity to a variable when freely choosing a variable.
    pub polarity_lean: PolarityLean,

    /// Preprocessing configuration
    pub random_choice_frequency: RandomChoiceFrequency,

    /// Which stopping criteria to use during resolution based analysis
    pub stopping_criteria: StoppingCriteria,

    /// The time limit for a solve.
    pub time_limit: Option<std::time::Duration>,

    /// Which VSIDS variant to use during resolution based analysis
    pub vsids_variant: VSIDS,

    /// Reduction schedules
    pub reduction_scheduler: ReductionScheduler,

    pub enabled: AbalableThings,
    pub clause_db: ClauseDBConfig,
    pub variable_db: VariableDBConfig,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            luby_u: 128,

            polarity_lean: 0.0,

            random_choice_frequency: 0.0,

            stopping_criteria: StoppingCriteria::FirstUIP,

            reduction_scheduler: ReductionScheduler {
                luby: Some(2),
                conflict: Some(50_000),
            },

            time_limit: None,
            vsids_variant: VSIDS::MiniSAT,

            enabled: AbalableThings::default(),
            clause_db: ClauseDBConfig::default(),
            variable_db: VariableDBConfig::default(),
        }
    }
}
pub mod switches {
    //! Boolean valued context configurations
    //! When set to true things related to the identifier are enabled.

    #[derive(Clone, Debug)]
    pub struct AbalableThings {
        pub preprocessing: bool,
        pub restart: bool,
        pub subsumption: bool,
    }

    impl Default for AbalableThings {
        fn default() -> Self {
            AbalableThings {
                preprocessing: false,
                restart: true,
                subsumption: true,
            }
        }
    }
}
