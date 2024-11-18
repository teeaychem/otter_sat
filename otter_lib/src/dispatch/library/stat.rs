use std::time::Duration;

#[derive(Clone)]
pub enum Stat {
    Iterations(usize),
    Chosen(usize),
    Conflicts(usize),
    Time(Duration),
}
