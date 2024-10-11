use crate::structures::{
    clause::{Clause, ClauseVec},
    literal::Literal,
    valuation::Valuation,
    variable::{Variable, VariableId},
};

pub type ClauseBox = Box<[Literal]>;

impl Clause for ClauseBox {
    fn literals(&self) -> impl Iterator<Item = Literal> {
        self.iter().cloned()
    }

    fn variables(&self) -> impl Iterator<Item = VariableId> {
        self.iter().map(|literal| literal.v_id())
    }

    fn as_string(&self) -> String {
        let mut the_string = String::from("(");
        for literal in self {
            the_string.push_str(format!(" {} ", literal).as_str())
        }
        the_string += ")";
        the_string
    }

    fn as_dimacs(&self, variables: &[Variable]) -> String {
        let mut the_string = String::from("");
        for literal in self {
            let the_represenetation = match literal.polarity() {
                true => format!("{} ", variables[literal.index()].name()),
                false => format!("-{} ", variables[literal.index()].name()),
            };
            the_string.push_str(the_represenetation.as_str())
        }
        the_string += "0";
        the_string
    }

    fn to_clause_vec(self) -> ClauseVec {
        self.to_vec()
    }

    fn length(&self) -> usize {
        self.len()
    }

    /// Returns the literal asserted by the clause on the given valuation
    fn asserts(&self, val: &impl Valuation) -> Option<Literal> {
        let mut the_literal = None;
        for lit in self.literals() {
            if let Some(existing_val) = val.of_v_id(lit.v_id()) {
                match existing_val == lit.polarity() {
                    true => return None,
                    false => continue,
                }
            } else if the_literal.is_none() {
                the_literal = Some(lit);
            } else {
                return None;
            }
        }
        the_literal
    }

    // TODO: consider a different approach to lbd
    // e.g. an approximate measure of =2, =3, >4 can be settled much more easily
    fn lbd(&self, vars: &[Variable]) -> usize {
        let mut decision_levels = self
            .iter()
            .map(|l| vars[l.index()].decision_level())
            .collect::<Vec<_>>();
        decision_levels.sort_unstable();
        decision_levels.dedup();
        decision_levels.len()
    }

    /// Returns Some(literal) whose variable id matches the given id
    /// Uses binary search on longer clauses, as literals are ordered by variable ids
    fn find_literal_by_id(&self, id: VariableId) -> Option<Literal> {
        if self.len() < 64 {
            self.iter().find(|l| l.v_id() == id).copied()
        } else {
            find_literal_by_id_binary(self, id)
        }
    }
}

fn find_literal_by_id_binary(clause: &[Literal], id: VariableId) -> Option<Literal> {
    let mut min = 0;
    let mut max = clause.len() - 1;
    let mut midpoint;
    let mut attempt;
    loop {
        midpoint = min + ((max - min) / 2);
        attempt = clause[midpoint];
        if max - min == 0 {
            match attempt.v_id() == id {
                true => return Some(attempt),
                false => return None,
            }
        }
        match attempt.v_id().cmp(&id) {
            std::cmp::Ordering::Less => min = midpoint + 1,
            std::cmp::Ordering::Equal => {
                return Some(attempt);
            }
            std::cmp::Ordering::Greater => max = midpoint - 1,
        }
    }
}
