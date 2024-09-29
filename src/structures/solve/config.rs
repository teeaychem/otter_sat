#[derive(Debug)]
pub enum StoppingCriteria {
    FirstAssertingUIP,
    None,
}

#[derive(Debug)]
pub struct SolveConfig {
    pub core: bool,
    pub analysis: usize,
    pub min_glue_strength: usize,
    pub stopping_criteria: StoppingCriteria,
    pub break_on_first: bool,
}
