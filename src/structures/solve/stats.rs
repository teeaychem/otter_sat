pub static mut PROPAGATION_TIME: std::time::Duration = std::time::Duration::new(0, 0);
pub static mut CONFLICT_TIME: std::time::Duration = std::time::Duration::new(0, 0);
pub static mut REDUCTION_TIME: std::time::Duration = std::time::Duration::new(0, 0);
pub static mut CHOICE_TIME: std::time::Duration = std::time::Duration::new(0, 0);
pub static mut LITERAL_UPDATE_TIME: std::time::Duration = std::time::Duration::new(0, 0);
pub static mut WATCH_CHOICES_TIME: std::time::Duration = std::time::Duration::new(0, 0);

pub struct SolveStats {
    pub total_time: std::time::Duration,
    pub iterations: usize,
    pub conflicts: usize,
}

impl SolveStats {
    pub fn new() -> Self {
        SolveStats {
            total_time: std::time::Duration::new(0, 0),
            iterations: 0,
            conflicts: 0,
        }
    }
}

#[rustfmt::skip]
impl std::fmt::Display for SolveStats {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "c STATS")?;
        writeln!(f, "c   ITERATIONS:    {}", self.iterations)?;
        writeln!(f, "c   CONFLICTS:     {}", self.conflicts)?;
        #[cfg(feature = "extra_stats")]
        writeln!(f, "c   CONFLICT RATIO {:.8?}", self.conflicts as f32 / self.iterations as f32)?;
        writeln!(f, "c   TIME:          {:.2?}", self.total_time)?;
        #[cfg(feature = "time")]
        {
        writeln!(f, "c     PROPAGATION: {:.2?}", unsafe {PROPAGATION_TIME})?;
        writeln!(f, "c         WATCH CHOICE:    {:.2?}", unsafe {WATCH_CHOICES_TIME})?;
        writeln!(f, "c     CONFLICT:    {:.2?}", unsafe {CONFLICT_TIME})?;
        writeln!(f, "c     CHOICE:      {:.2?}", unsafe {CHOICE_TIME})?;
        writeln!(f, "c       REDUCTION: {:.2?}", unsafe {REDUCTION_TIME})?;
        }
        Ok(())
    }
}
