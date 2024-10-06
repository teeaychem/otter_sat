pub struct SolveStats {
    pub total_time: std::time::Duration,
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
            implication_time: std::time::Duration::new(0, 0),
            unsat_time: std::time::Duration::new(0, 0),
            reduction_time: std::time::Duration::new(0, 0),
            choice_time: std::time::Duration::new(0, 0),
            iterations: 0,
            conflicts: 0,
        }
    }
}

#[rustfmt::skip]
impl std::fmt::Display for SolveStats {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "c STATS")?;
        writeln!(f, "c | ITERATIONS:    {}", self.iterations)?;
        writeln!(f, "c | CONFLICTS:     {}", self.conflicts)?;
        writeln!(f, "c | CONFLICT RATIO {:.8?}", self.conflicts as f32 / self.iterations as f32)?;
        writeln!(f, "c | TIME:          {:.2?}", self.total_time)?;
        writeln!(f, "c | | IMPLICATION: {:.2?}", self.implication_time)?;
        writeln!(f, "c | | UNSAT:       {:.2?}", self.unsat_time)?;
        writeln!(f, "c | | CHOICE:      {:.2?}", self.choice_time)?;
        writeln!(f, "c | |   REDUCTION:   {:.2?}", self.reduction_time)?;
        Ok(())
    }
}
