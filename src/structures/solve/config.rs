#[derive(Debug)]
pub enum StoppingCriteria {
    FirstAssertingUIP,
    None,
}

#[derive(Debug)]
pub enum ConflictPriority {
    High,
    Low,
    Default
}


#[derive(Debug)]
pub struct SolveConfig {
    pub glue_strength: usize,
    pub stats: bool,
    pub show_assignment: bool,
    pub core: bool,
    pub analysis: usize,
    pub stopping_criteria: StoppingCriteria,
    pub break_on_first: bool,
    pub multi_jump_max: bool,
    pub conflict_priority: ConflictPriority
}
