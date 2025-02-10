#[derive(Clone)]
pub struct LiteralDBConfig {
    pub stacked_assumptions: bool,
}

impl Default for LiteralDBConfig {
    fn default() -> Self {
        Self {
            stacked_assumptions: false,
        }
    }
}
