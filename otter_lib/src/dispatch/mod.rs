use crate::context::Context;

pub mod frat;
pub mod library;
pub mod receivers;
pub mod transmitters;

#[derive(Clone)]
pub enum Dispatch {
    Delta(library::delta::Delta),
    Report(library::report::Report),
    Comment(library::comment::Comment),
    Stats(library::stat::Stat),
}

impl Context {
    pub fn dispatch_active(&self) {
        self.clause_db.dispatch_active();

        for literal in self.literal_db.proven_literals() {
            let report = library::report::VariableDB::Active(*literal);
            self.tx
                .send(Dispatch::Report(library::report::Report::VariableDB(
                    report,
                )));
        }
    }
}
