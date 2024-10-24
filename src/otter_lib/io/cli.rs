use clap::{value_parser, Arg, ArgMatches, Command};

use crate::config::{self, Config, StoppingCriteria, VSIDS};

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

        .arg(Arg::new("elephant")
            .long("elephant")
            .short('🐘')
            .value_parser(value_parser!(bool))
            .required(false)
            .num_args(0)
            .help("Remember everything.")
.long_help("Remember everything.
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
            .short('🧹')
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
Default: {}", config::defaults::GLUE_STRENGTH)))

        .arg(Arg::new("stopping_criteria")
            .long("stopping-criteria")
            .short('🚏')
            .value_name("CRITERIA")
            .value_parser(value_parser!(StoppingCriteria))
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
            .long("VSIDS-variant")
            .short('🦇')
            .value_parser(value_parser!(VSIDS))
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

        .arg(Arg::new("paths")
            .required(false)
            .trailing_var_arg(true)
            .num_args(0..)
            .help("The DIMACS form CNF files to parse."))
}

impl Config {
    pub fn from_args(args: &ArgMatches) -> Self {
        let mut the_config = Config::default();

        if let Ok(Some(strength)) = args.try_get_one::<config::GlueStrength>("glue_strength") {
            the_config.glue_strength = *strength
        };

        if let Ok(Some(u)) = args.try_get_one::<config::LubyConstant>("luby") {
            the_config.luby_constant = *u
        };
        if let Ok(Some(lean)) = args.try_get_one::<config::PolarityLean>("polarity_lean") {
            the_config.polarity_lean = *lean
        };
        if let Ok(Some(frequency)) =
            args.try_get_one::<config::RandomChoiceFrequency>("random_choice_frequency")
        {
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
        if let Ok(Some(value)) = args.try_get_one::<bool>("core") {
            the_config.show_core = *value
        };
        if let Ok(Some(value)) = args.try_get_one::<bool>("stats") {
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
            the_config.time_limit = Some(std::time::Duration::from_secs(*secs))
        };

        if let Ok(Some(criteria)) = args.try_get_one::<StoppingCriteria>("stopping_criteria") {
            the_config.stopping_criteria = *criteria
        };

        if let Ok(Some(variant)) = args.try_get_one::<VSIDS>("VSIDS_variant") {
            the_config.vsids_variant = *variant
        };

        if let Ok(Some(true)) = args.try_get_one::<bool>("elephant") {
            the_config.restarts_allowed = false;
            the_config.reduction_allowed = false;
        };

        the_config
    }
}
