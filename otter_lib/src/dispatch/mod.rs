use crate::context::Context;

pub mod frat;
pub mod library;

#[derive(Clone)]
pub enum Dispatch {
    Delta(library::delta::Delta),
    Report(library::report::Report),
    Comment(library::comment::Comment),
    Stats(library::stat::Stat),
}

impl Context {
    pub fn dispatch_active(&self) {
        if let Some(tx) = &self.tx {
            self.clause_db.dispatch_active();

            for literal in self.literal_db.proven_literals() {
                let report = library::report::LiteralDB::Active(*literal);
                tx.send(Dispatch::Report(library::report::Report::LiteralDB(report)));
            }
        }
    }
}
