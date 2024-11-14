use crate::{context::Context, dispatch::Dispatch, types::gen::SolveStatus};

use super::{comment, report};

impl Context {
    pub fn print_status(&self) {
        match self.status {
            SolveStatus::FullValuation => {
                let _ = self
                    .tx
                    .send(Dispatch::SolveReport(report::Solve::Satisfiable));
            }
            SolveStatus::NoSolution => {
                let report = report::Solve::Unsatisfiable;
                let _ = self.tx.send(Dispatch::SolveReport(report));
            }
            SolveStatus::NoClauses => {
                self.tx
                    .send(Dispatch::SolveComment(comment::Solve::NoClauses));
            }
            _ => {
                let _ = self.tx.send(Dispatch::SolveReport(report::Solve::Unknown));
            }
        }
    }
}
