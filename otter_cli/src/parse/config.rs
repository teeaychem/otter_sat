use clap::ArgMatches;

use otter_lib::config::{self, Config, StoppingCriteria, VSIDS};

pub fn config_from_args(args: &ArgMatches) -> Config {
    let mut the_config = Config::default();

    if let Ok(Some(strength)) = args.try_get_one::<config::LBD>("glue_strength") {
        the_config.clause_db.lbd_bound = *strength
    };

    if let Ok(Some(decay)) = args.try_get_one::<config::Activity>("variable_decay") {
        the_config.atom_db.decay = decay * 1e-3
    };

    if let Ok(Some(decay)) = args.try_get_one::<config::Activity>("clause_decay") {
        the_config.clause_db.decay = *decay * 1e-3
    };

    if let Ok(Some(interval)) = args.try_get_one::<Option<u32>>("reduction_interval") {
        the_config.scheduler.luby = *interval
    };

    if let Ok(Some(u)) = args.try_get_one::<otter_lib::generic::luby::LubyRepresentation>("luby") {
        the_config.luby_u = *u
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
        the_config.switch.preprocessing = *value
    };

    if let Ok(Some(value)) = args.try_get_one::<bool>("no_restarts") {
        the_config.switch.restart = !*value
    };

    if let Ok(Some(true)) = args.try_get_one::<bool>("no_reduction") {
        the_config.scheduler.luby = None;
        the_config.scheduler.conflict = None;
    };

    if let Ok(Some(value)) = args.try_get_one::<bool>("no_subsumption") {
        the_config.switch.subsumption = !*value
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
        the_config.switch.restart = false;
        the_config.scheduler.luby = None;
        the_config.scheduler.conflict = None;
    };

    the_config
}
