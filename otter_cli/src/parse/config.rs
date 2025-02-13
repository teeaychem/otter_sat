use clap::ArgMatches;

use otter_sat::config::{self, vsids::VSIDS, Config, StoppingCriteria};

pub fn config_from_args(args: &ArgMatches) -> Config {
    let mut cfg = Config::default();

    if let Ok(Some(strength)) = args.try_get_one::<config::LBD>("glue_strength") {
        cfg.clause_db.lbd_bound = *strength
    };

    if let Ok(Some(decay)) = args.try_get_one::<config::Activity>("variable_decay") {
        cfg.atom_db.decay = decay * 1e-3
    };

    if let Ok(Some(decay)) = args.try_get_one::<config::Activity>("clause_decay") {
        cfg.clause_db.decay = *decay * 1e-3
    };

    if let Ok(Some(interval)) = args.try_get_one::<Option<u32>>("reduction_interval") {
        cfg.scheduler.luby = *interval
    };

    if let Ok(Some(u)) = args.try_get_one::<otter_sat::generic::luby::LubyRepresentation>("luby") {
        cfg.luby_u = *u
    };

    if let Ok(Some(lean)) = args.try_get_one::<config::PolarityLean>("polarity_lean") {
        cfg.polarity_lean = *lean
    };

    if let Ok(Some(frequency)) =
        args.try_get_one::<config::RandomDecisionBias>("random_decision_bias")
    {
        cfg.random_decision_bias = *frequency
    };

    if let Ok(Some(value)) = args.try_get_one::<bool>("preprocessing") {
        cfg.preprocessing = *value
    };

    if let Ok(Some(value)) = args.try_get_one::<bool>("no_restarts") {
        cfg.restart = !*value
    };

    if let Ok(Some(true)) = args.try_get_one::<bool>("no_reduction") {
        cfg.scheduler.luby = None;
        cfg.scheduler.conflict = None;
    };

    if let Ok(Some(value)) = args.try_get_one::<bool>("no_subsumption") {
        cfg.subsumption = !*value
    };

    if let Ok(Some(secs)) = args.try_get_one::<u64>("time_limit") {
        cfg.time_limit = Some(std::time::Duration::from_secs(*secs))
    };

    if let Ok(Some(criteria)) = args.try_get_one::<StoppingCriteria>("stopping_criteria") {
        cfg.stopping_criteria = *criteria
    };

    if let Ok(Some(variant)) = args.try_get_one::<VSIDS>("VSIDS_variant") {
        cfg.vsids_variant = *variant
    };

    if let Ok(Some(true)) = args.try_get_one::<bool>("elephant") {
        cfg.restart = false;
        cfg.scheduler.luby = None;
        cfg.scheduler.conflict = None;
    };

    cfg
}
