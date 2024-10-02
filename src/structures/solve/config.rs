#[derive(Debug, Clone)]
pub enum StoppingCriteria {
    FirstAssertingUIP,
    None,
}

#[derive(Debug, Clone)]
pub enum ExplorationPriority {
    Conflict,
    Implication,
    Default,
}

pub fn config_glue_strength() -> usize {
    unsafe { crate::CONFIG_GLUE_STRENGTH }
}

pub fn config_show_stats() -> bool {
    unsafe { crate::CONFIG_SHOW_STATS }
}

pub fn config_exploration_priority() -> ExplorationPriority {
    unsafe { crate::CONFIG_EXPLORATION_PRIORITY.clone() }
}

pub fn config_stopping_criteria() -> StoppingCriteria {
    unsafe { crate::CONFIG_STOPPING_CRITERIA.clone() }
}

pub fn config_show_core() -> bool {
    unsafe { crate::CONFIG_SHOW_CORE }
}

pub fn config_show_assignment() -> bool {
    unsafe { crate::CONFIG_SHOW_ASSIGNMENT }
}
