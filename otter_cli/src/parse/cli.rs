use std::path::PathBuf;

use clap::{value_parser, Arg, Command};

use otter_lib::config::{self, Activity, StoppingCriteria, VSIDS};

pub fn cli() -> Command {
    Command::new("otter_sat")
        .about("Determines whether a formula is satisfiable or unsatisfialbe")
        .version("pup (it's still growing)")

        .arg(Arg::new("paths")
            .required(false)
            .trailing_var_arg(true)
            .num_args(0..)
            .value_parser(value_parser!(PathBuf))
            .help("The DIMACS form CNF files to parse (as a single formula)."))

        .arg(Arg::new("core")
            .short('c')
            .long("show-core")
            .value_parser(value_parser!(bool))
            .required(false)
            .num_args(0)
            .help("Display an unsatisfiable core on finding a given formula is unsatisfiable."))

        .arg(Arg::new("variable_decay")
            .long("variable-decay")
            .value_parser(value_parser!(Activity))
            .required(false)
            .num_args(1)
            .help("The decay to use for variable activity.")
            .help(format!("The decay to use for variable activity.
Default: {}

After a conflict any future variables will be bumped with activity (proportional to) 1 / (1 - decay^-3).
Viewed otherwise, the activity of all variables is decayed by 1 - decay^-3 each conflict.
For example, at decay of 3 at each conflict the activity of a variable decays to 0.875 of it's previous activity.", config::defaults::VARIABLE_DECAY_FACTOR)))

        .arg(Arg::new("clause_decay")
            .long("clause-decay")
            .value_parser(value_parser!(Activity))
            .required(false)
            .num_args(1)
            .help("The decay to use for clause activity.")
            .help(format!("The decay to use for clause activity.
Default: {}

Works the same as variable activity, but applied to clauses.
If reductions are allowed then clauses are removed from low to high activity.", config::defaults::CLAUSE_DECAY_FACTOR)))

        .arg(Arg::new("reduction_interval")
            .long("reduction-interval")
            .value_parser(value_parser!(usize))
            .required(false)
            .num_args(1)
            .help("The interval to perform reductions, relative to conflicts.")
            .help(format!("The interval to perform reductions, relative to conflicts.
Default: {}

After interval number of conflicts the clause database is reduced.
Clauses of length two are never removed.
Clauses with length greater than two are removed, low activity to high (and high lbd to low on activity ties).", config::defaults::REDUCTION_INTERVAL)))

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
            .help("Prevent choices from being forgotten."))

        .arg(Arg::new("elephant")
            .long("elephant")
            .short('🐘')
            .value_parser(value_parser!(bool))
            .required(false)
            .num_args(0)
            .help("Remember everything.")
.long_help("Remember everything.
Equivalent to passing both '--no-reduction' and 'no_restarts'."))

        .arg(Arg::new("no_subsumption")
            .long("no-subsumption")
            .value_parser(value_parser!(bool))
            .required(false)
            .num_args(0)
            .help(
                "Prevent (some simple) self-subsumption.")
            .long_help("Prevent (some simple) self-subsumption.

That is, when performing resolutinon some stronger form of a clause may be found.
Subsumption allows the weaker clause is replaced (subsumed by) the stronger clause.
For example, p ∨ r subsumes p ∨ q ∨ r."))

        .arg(Arg::new("preprocessing")
            .long("preprocess")
            .short('p')
            .value_parser(value_parser!(bool))
            .required(false)
            .num_args(0)
            .help("Perform some pre-processing before a solve.
For the moment this is limited to settling all atoms which occur with a unique polarity."))

        .arg(Arg::new("glue_strength")
            .long("glue")
            .short('g')
            .value_name("STRENGTH")
            .value_parser(value_parser!(usize))
            .required(false)
            .num_args(1)
            .help(format!("Required minimum (inintial) lbd to retain a clause during a reduction.
Default: {}", config::defaults::GLUE_STRENGTH)))

        .arg(Arg::new("stopping_criteria")
            .long("stopping-criteria")
            .short('🚏')
            .value_name("CRITERIA")
            .value_parser(clap::builder::ValueParser::new(stopping_criteria_parser))
            .required(false)
            .num_args(1)
            .help(format!("Resolution stopping criteria.
Default: {}", config::defaults::STOPPING_CRITERIA))
            .long_help(format!("The stopping criteria to use during resolution.
Default: {}

  - FirstUIP: Resolve until the first unique implication point
  - None    : Resolve on each clause used to derive the conflict", config::defaults::STOPPING_CRITERIA)))

        .arg(Arg::new("VSIDS_variant")
            .value_name("VARIANT")
            .long("VSIDS")
            .short('🦇')
            .value_parser(clap::builder::ValueParser::new(vsids_parser))
            .required(false)
            .num_args(1)
            .help(format!("Which VSIDS variant to use.
Default: {}", config::defaults::VSIDS_VARIANT))
            .long_help(format!("Which VSIDS variant to use.
Default: {}

  - MiniSAT: Bump the activity of all variables in the a learnt clause.
  - Chaff  : Bump the activity involved when using resolution to learn a clause.", config::defaults::VSIDS_VARIANT)))

        .arg(Arg::new("luby")
            .long("luby")
            .short('l')
            .value_name("U")
            .value_parser(value_parser!(usize))
            .required(false)
            .num_args(1)
            .help(format!("The 'u' value to use for the luby calculation when restarts are permitted.
Default: {}", config::defaults::LUBY_U)))

        .arg(Arg::new("random_choice_frequency")
            .long("random-choice-frequency")
            .short('r')
            .value_name("FREQUENCY")
            .value_parser(value_parser!(f64))
            .required(false)
            .num_args(1)
            .help(format!("The chance of making a random choice (as opposed to using most VSIDS activity).
Default: {}", config::defaults::RANDOM_CHOICE_FREQUENCY)))

        .arg(Arg::new("polarity_lean")
            .long("polarity-lean")
            .short('∠')
            .value_name("LEAN")
            .value_parser(value_parser!(f64))
            .required(false)
            .num_args(1)
            .help(format!("The chance of choosing assigning positive polarity to a variant when making a choice.
Default: {}", config::defaults::POLARITY_LEAN)))

        .arg(Arg::new("time_limit")
            .long("time-limit")
            .short('t')
            .value_name("SECONDS")
            .value_parser(value_parser!(u64))
            .required(false)
            .num_args(1)
            .help("Time limit for the solve in seconds.
Default: No limit"))

        // CLI specific arguments

        .arg(Arg::new("detail")
            .long("detail")
            .short('d')
            .value_name("LEVEL")
            .value_parser(value_parser!(u8))
            .required(false)
            .num_args(1)
            .help(format!("The level to which details are communicated during a solve.
Default: {}", crate::config_io::DETAILS)))

    .arg(Arg::new("stats")
            .short('s')
            .long("stats")
            .value_parser(value_parser!(bool))
            .required(false)
            .num_args(0)
            .help("Display stats during a solve."))

        .arg(Arg::new("valuation")
            .short('v')
            .long("valuation")
            .value_parser(value_parser!(bool))
            .required(false)
            .num_args(0)
            .help("Display valuation on completion."))


        .arg(Arg::new("FRAT")
            .long("FRAT")
            .value_parser(value_parser!(bool))
            .required(false)
            .num_args(0)
            .help("Write an FRAT proof."))

        .arg(Arg::new("FRAT_path")
            .long("FRAT-path")
            .value_name("PATH")
            .value_parser(value_parser!(PathBuf))
            .required(false)
            .num_args(1)
            .help("The path to write an FRAT proof."))
}

fn vsids_parser(arg: &str) -> Result<VSIDS, std::io::Error> {
    match arg {
        "Chaff" => Ok(VSIDS::Chaff),
        "MiniSAT" => Ok(VSIDS::MiniSAT),
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Unknown VSIDS variant",
        )),
    }
}

fn stopping_criteria_parser(arg: &str) -> Result<StoppingCriteria, std::io::Error> {
    match arg {
        "FirstUIP" => Ok(StoppingCriteria::FirstUIP),
        "None" => Ok(StoppingCriteria::None),
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Unknown stopping criteria variant",
        )),
    }
}
