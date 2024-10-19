use crate::context::Context;
use clap::Parser;
use serde::Serialize;

/// Determines whether a formula is satisfiable or unsatisfialbe
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[allow(non_snake_case)]
pub struct Args {
    /// The DIMACS form CNF file to parse
    #[arg(short, long)]
    formula_file: std::path::PathBuf,

    /// Display stats on completion
    #[arg(short, long, default_value_t = false)]
    stats: bool,

    /// Display a satisfying valuation, if possible
    #[arg(short, long, default_value_t = false)]
    valuation: bool,

    /// Display an unsatisfiable core on UNSAT
    #[arg(short, long, default_value_t = false)]
    core: bool,

    /// Required glue strength
    #[arg(short, long, default_value_t = 2)]
    glue_strength: usize,

    /// Resolution stopping criteria
    #[arg(long, default_value_t, value_enum)]
    stopping_criteria: StoppingCriteria,

    /// Which VSIDS variant to use
    #[arg(long = "VSIDS", default_value_t, value_enum)]
    vsids: VSIDS,

    /// Suggest priority exploring conflcits, implications, or take no interest (does nothing at the moment)
    #[arg(long, default_value_t, value_enum)]
    exploration_priority: ExplorationPriority,

    /// Reduce and restart, where:
    #[arg(short, long = "reduce-and-restart", default_value_t = false)]
    rr: bool,

    /// Allow for the clauses to be forgotten, on occassion
    #[arg(long, default_value_t = false)]
    reduce: bool,

    /// Allow for the decisions to be forgotten, on occassion
    #[arg(long, default_value_t = false)]
    restart: bool,

    /// Initially settle all atoms which occur with a unique polarity
    #[arg(long, default_value_t = false)]
    hobson: bool,

    #[arg(long, default_value_t = 0.0)]
    /// The chance of making a random choice (as opposed to using most VSIDS activity)
    random_choice_frequency: f64,

    #[arg(short, long, default_value_t = 0.0)]
    /// The chance of choosing assigning positive polarity to a variant when making a choice
    polarity_lean: f64,

    #[arg(short = 'l', long = "luby", default_value_t = 512)]
    /// The u value to use for the luby calculation when restarts are permitted
    luby_u: usize,

    /// Time limit for the solve
    #[arg(short, long, value_parser = |seconds: &str| seconds.parse().map(std::time::Duration::from_secs))]
    time: Option<std::time::Duration>,

    #[arg(short = 'u', long, default_value_t = false, verbatim_doc_comment)]
    /// Allow (some simple) self-subsumption
    /// I.e. when performing resolutinon some stronger form of a clause may be found
    /// For example, p ∨ q ∨ r may be strengthened to p ∨ r
    /// With subsumption the weaker clause is replaced (subsumed by) the stronger clause, this flag disables the process
    subsumption: bool
}

#[derive(Clone)]
pub struct Config {
    pub formula_file: std::path::PathBuf,
    pub exploration_priority: ExplorationPriority,
    pub glue_strength: usize,
    pub hobson_choices: bool,
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
}

impl Config {
    pub fn from_args(args: Args) -> Self {
        let mut the_config = Config {
            formula_file: args.formula_file,
            exploration_priority: args.exploration_priority,
            glue_strength: args.glue_strength,
            hobson_choices: args.hobson,
            luby_constant: args.luby_u,
            polarity_lean: args.polarity_lean,
            reduction_allowed: if args.reduce && !args.restart {
                println!("c REDUCTION REQUIRES RESTARTS TO BE ENABLED");
                false
            } else {
                args.reduce
            },
            restarts_allowed: args.restart,
            show_core: args.core,
            show_stats: args.stats,
            show_valuation: args.valuation,
            stopping_criteria: args.stopping_criteria,
            time_limit: args.time,
            vsids_variant: args.vsids,
            activity_conflict: 1.0,
            decay_factor: 0.95,
            decay_frequency: 1,
            subsumption: args.subsumption,
            random_choice_frequency: args.random_choice_frequency
        };

        if args.rr {
            the_config.restarts_allowed = true;
            the_config.reduction_allowed = true;
        }

        the_config
    }
}

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

impl std::fmt::Display for VSIDS {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::MiniSAT => write!(f, "MiniSAT"),
            Self::Chaff => write!(f, "Chaff"),
        }
    }
}
