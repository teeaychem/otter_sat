pub struct SolveStats {
    pub total_time: std::time::Duration,
    pub examination_time: std::time::Duration,
    pub implication_time: std::time::Duration,
    pub unsat_time: std::time::Duration,
    pub reduction_time: std::time::Duration,
    pub choice_time: std::time::Duration,
    pub iterations: usize,
    pub conflicts: usize,
}

impl SolveStats {
    pub fn new() -> Self {
        SolveStats {
            total_time: std::time::Duration::new(0, 0),
            examination_time: std::time::Duration::new(0, 0),
            implication_time: std::time::Duration::new(0, 0),
            unsat_time: std::time::Duration::new(0, 0),
            reduction_time: std::time::Duration::new(0, 0),
            choice_time: std::time::Duration::new(0, 0),
            iterations: 0,
            conflicts: 0,
        }
    }
}

impl std::fmt::Display for SolveStats {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "c STATS")?;
        writeln!(f, "c ITERATIONS: {}", self.iterations)?;
        writeln!(f, "c CONFLICTS: {}", self.conflicts)?;
        writeln!(f, "c TIME: {:.2?}", self.total_time)?;
        writeln!(f, "c \tEXAMINATION: {:.2?}", self.examination_time)?;
        writeln!(f, "c \tIMPLICATION: {:.2?}", self.implication_time)?;
        writeln!(f, "c \tUNSAT: {:.2?}", self.unsat_time)?;
        writeln!(f, "c \tREDUCTION: {:.2?}", self.reduction_time)?;
        writeln!(f, "c \tCHOICE: {:.2?}", self.choice_time)?;
        Ok(())
    }
}
