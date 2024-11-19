// use crate::{context::Context, dispatch::Dispatch, types::gen::Solve};

// use super::{
//     library::comment::{self, Comment},
//     library::report::{self, Report},
// };

// impl Context {
//     pub fn print_status(&self) {
//         match self.status {
//             Solve::FullValuation => {
//                 let the_report = report::Solve::Satisfiable;
//                 self.tx.send(Dispatch::Report(Report::Solve(the_report)));
//             }
//             Solve::NoSolution => {
//                 let the_report = report::Solve::Unsatisfiable;
//                 let _ = self.tx.send(Dispatch::Report(Report::Solve(the_report)));
//             }
//             Solve::NoClauses => {
//                 let the_comment = comment::Solve::NoClauses;
//                 self.tx.send(Dispatch::Comment(Comment::Solve(the_comment)));
//             }
//             _ => {
//                 let the_report = report::Solve::Unknown;
//                 self.tx.send(Dispatch::Report(Report::Solve(the_report)));
//             }
//         }
//     }
// }
