use std::time::Duration;

use crate::context::Context;
use clap::{value_parser, Arg, ArgMatches, Command};
// use serde::Serialize;

const LUBY_U: usize = 512;
const GLUE_STRENGTH: usize = 2;
const STOPPING_CRITERIA: StoppingCriteria = StoppingCriteria::FirstUIP;
const VSIDS_VARIANT: VSIDS = VSIDS::MiniSAT;
const RANDOM_CHOICE_FREQUENCY: f64 = 0.0;
const POLARITY_LEAN: f64 = 0.0;

pub fn cli() -> Command {
    Command::new("otter_sat")
        .about("Determines whether a formula is satisfiable or unsatisfialbe")
        .version("pup (it's still growing)")

        .arg(Arg::new("core")
            .short('c')
            .long("show-core")
            .value_parser(value_parser!(bool))
            .required(false)
            .num_args(0)
            .help("Display an unsatisfiable core on finding a given formula is unsatisfiable."))

        .arg(Arg::new("no_reduction")
            .long("no-reduction")
            .value_parser(value_parser!(bool))
            .required(false)
            .num_args(0)
            .help("Prevent clauses from being forgotten."))

        .arg(Arg::new("no_restarts")
            .long("no-restart")
            .value_parser(value_parser!(bool))
            .required(false)
            .num_args(0)
            .help("Prevent decisions being forgotten."))

        .arg(Arg::new("persevere")
            .long("persevere")
            .value_parser(value_parser!(bool))
            .required(false)
            .num_args(0)
            .help("Deny both to reduce and to restart.
Equivalent to passing both '--no-reduction' and 'no_restarts'."))

        .arg(Arg::new("preprocessing")
            .long("preprocess")
            .short('p')
            .value_parser(value_parser!(bool))
            .required(false)
            .num_args(0)
            .help("Perform some pre-processing before a solve.
For the moment this is limited to settling all atoms which occur with a unique polarity."))

        .arg(Arg::new("stats")
            .short('s')
            .long("stats")
            .value_parser(value_parser!(bool))
            .required(false)
            .num_args(0)
            .help("Display stats during a solve."))


        .arg(Arg::new("subsumption")
            .long("subsumption")
            .short('u')
            .value_parser(value_parser!(bool))
            .required(false)
            .num_args(0)
            .help(
                "Allow (some simple) self-subsumption.")
            .long_help("Allow (some simple) self-subsumption.

That is, when performing resolutinon some stronger form of a clause may be found.
Subsumption allows the weaker clause is replaced (subsumed by) the stronger clause.
For example, p ∨ r subsumes p ∨ q ∨ r."))


        .arg(Arg::new("tidy_watches")
            .long("tidy-watches")
            .value_parser(value_parser!(bool))
            .required(false)
            .num_args(0)
            .help("Continue updating watches for all queued literals after a conflict."))

        .arg(Arg::new("valuation")
            .short('v')
            .long("valuation")
            .value_parser(value_parser!(bool))
            .required(false)
            .num_args(0)
            .help("Display valuation on completion."))


        .arg(Arg::new("glue_strength")
            .long("glue")
            .short('g')
            .value_name("STRENGTH")
            .value_parser(value_parser!(usize))
            .required(false)
            .num_args(1)
            .help(format!("Required minimum (inintial) lbd to retain a clause during a reduction.
Default: {GLUE_STRENGTH}")))

        .arg(Arg::new("stopping_criteria")
            .long("stopping-criteria")
            .value_name("CRITERIA")
            .value_parser(value_parser!(StoppingCriteria))
            .required(false)
            .num_args(1)
            .help(format!("Resolution stopping criteria.
Default: {STOPPING_CRITERIA}"))
            .long_help(format!("The stopping criteria to use during resolution.
Default: {STOPPING_CRITERIA}

  - FirstUIP: Resolve until the first unique implication point
  - None    : Resolve on each clause used to derive the conflict")))

        .arg(Arg::new("VSIDS_variant")
            .value_name("VARIANT")
            .long("VSIDS-variant")
            .value_parser(value_parser!(VSIDS))
            .required(false)
            .num_args(1)
            .help(format!("Which VSIDS variant to use.
Default: {VSIDS_VARIANT}"))
            .long_help(format!("Which VSIDS variant to use.
Default: {VSIDS_VARIANT}

  - MiniSAT: Bump the activity of all variables in the a learnt clause.
  - Chaff  : Bump the activity involved when using resolution to learn a clause.")))

        .arg(Arg::new("luby")
            .long("luby")
            .short('l')
            .value_name("U")
            .value_parser(value_parser!(usize))
            .required(false)
            .num_args(1)
            .help(format!("The 'u' value to use for the luby calculation when restarts are permitted.
Default: {LUBY_U}")))

        .arg(Arg::new("random_choice_frequency")
            .long("random-choice-frequency")
            .short('r')
            .value_name("FREQUENCY")
            .value_parser(value_parser!(f64))
            .required(false)
            .num_args(1)
            .help(format!("The chance of making a random choice (as opposed to using most VSIDS activity).
Default: {RANDOM_CHOICE_FREQUENCY}")))

        .arg(Arg::new("polarity_lean")
            .long("polarity-lean")
            .value_name("LEAN")
            .value_parser(value_parser!(f64))
            .required(false)
            .num_args(1)
            .help(format!("The chance of choosing assigning positive polarity to a variant when making a choice.
Default: {POLARITY_LEAN}")))

        .arg(Arg::new("time_limit")
            .long("time-limit")
            .short('t')
            .value_name("LIMIT")
            .value_parser(value_parser!(u64))
            .required(false)
            .num_args(1)
            .help("Time limit for the solve in seconds.
Default: No limit"))

        .arg(Arg::new("paths")
            .required(false)
            .trailing_var_arg(true)
            .num_args(0..)
            .help("The DIMACS form CNF files to parse."))
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

impl Config {
    pub fn from_args(args: &ArgMatches) -> Self {
        let mut the_config = Config::default();

        if let Ok(Some(strength)) = args.try_get_one::<usize>("glue_strength") {
            the_config.glue_strength = *strength
        };

        if let Ok(Some(u)) = args.try_get_one::<usize>("luby") {
            the_config.luby_constant = *u
        };
        if let Ok(Some(lean)) = args.try_get_one::<f64>("polarity_lean") {
            the_config.polarity_lean = *lean
        };
        if let Ok(Some(frequency)) = args.try_get_one::<f64>("random_choice_frequency") {
            the_config.random_choice_frequency = *frequency
        };

        if let Ok(Some(value)) = args.try_get_one::<bool>("preprocessing") {
            the_config.preprocessing = *value
        };
        if let Ok(Some(value)) = args.try_get_one::<bool>("no_restarts") {
            the_config.restarts_allowed = !*value
        };
        if let Ok(Some(value)) = args.try_get_one::<bool>("no_reduction") {
            the_config.reduction_allowed = !*value
        };
        if let Ok(Some(value)) = args.try_get_one::<bool>("show_core") {
            the_config.show_core = *value
        };
        if let Ok(Some(value)) = args.try_get_one::<bool>("show_stats") {
            the_config.show_stats = *value
        };
        if let Ok(Some(value)) = args.try_get_one::<bool>("valuation") {
            the_config.show_valuation = *value
        };
        if let Ok(Some(value)) = args.try_get_one::<bool>("subsumption") {
            the_config.subsumption = *value
        };
        if let Ok(Some(value)) = args.try_get_one::<bool>("tidy_watches") {
            the_config.tidy_watches = *value
        };

        if let Ok(Some(secs)) = args.try_get_one::<u64>("time_limit") {
            the_config.time_limit = Some(Duration::from_secs(*secs))
        };

        if let Ok(Some(criteria)) = args.try_get_one::<StoppingCriteria>("stopping_criteria") {
            the_config.stopping_criteria = *criteria
        };

        if let Ok(Some(variant)) = args.try_get_one::<VSIDS>("VSIDS_variant") {
            the_config.vsids_variant = *variant
        };

        if let Ok(Some(true)) = args.try_get_one::<bool>("persevere") {
            the_config.restarts_allowed = false;
            the_config.reduction_allowed = false;
        };

        the_config
    }
}

impl Default for Config {
    fn default() -> Self {
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

impl Context {
    pub fn it_is_time_to_reduce(&self, u: usize) -> bool {
        self.conflicts_since_last_forget >= u.wrapping_mul(luby(self.restarts + 1))
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
        } else {
            k += 1;
        }
    }
}
