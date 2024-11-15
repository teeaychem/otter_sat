use crate::{context::Context, dispatch::Dispatch, types::gen::Solve};

use super::{
    comment::{self},
    report::{self},
};

impl Context {
    pub fn print_status(&self) {
        match self.status {
            Solve::FullValuation => {
                let _ = self
                    .tx
                    .send(Dispatch::SolveReport(report::Solve::Satisfiable));
            }
            Solve::NoSolution => {
                let report = report::Solve::Unsatisfiable;
                let _ = self.tx.send(Dispatch::SolveReport(report));
            }
            Solve::NoClauses => {
                self.tx
                    .send(Dispatch::SolveComment(comment::Solve::NoClauses));
            }
            _ => {
                let _ = self.tx.send(Dispatch::SolveReport(report::Solve::Unknown));
            }
        }
    }
}
