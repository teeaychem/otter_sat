use crate::structures::{Literal, LiteralError, Valuation, ValuationVec};

pub type ClauseVec = Vec<Literal>;

pub trait Clause: IntoIterator {
    fn add_literal(&mut self, literal: Literal) -> Result<(), ClauseError>;

    fn literals(&self) -> impl Iterator<Item = Literal>;

    fn is_sat_on(&self, valuation: &ValuationVec) -> bool;

    fn is_unsat_on(&self, valuation: &ValuationVec) -> bool;

    fn find_unit_literal<T: Valuation>(&self, valuation: &T) -> Option<Literal>;

    fn collect_choices<T: Valuation>(&self, valuation: &T) -> Option<Vec<Literal>>;

    fn as_string(&self) -> String;
}

pub type ClauseId = usize;

#[derive(Debug)]
pub enum ClauseError {
    Literal(LiteralError),
    Empty,
}

#[derive(Clone, Debug)]
pub struct StoredClause {
    pub id: usize,
    pub position: usize,
    pub clause: Vec<Literal>,
}

impl std::fmt::Display for StoredClause {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "#[{}] ", self.id)?;
        write!(f, "(")?;
        for literal in self.clause.iter() {
            write!(f, " {literal} ")?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

impl Clause for ClauseVec {
    fn add_literal(&mut self, literal: Literal) -> Result<(), ClauseError> {
        self.push(literal);
        Ok(())
    }

    fn literals(&self) -> impl Iterator<Item = Literal> {
        self.iter().cloned()
    }

    fn is_sat_on(&self, valuation: &ValuationVec) -> bool {
        self.iter()
            .any(|l| valuation.of_v_id(l.v_id) == Ok(Some(l.polarity)))
    }

    fn is_unsat_on(&self, valuation: &ValuationVec) -> bool {
        self.iter().all(|l| {
            if let Ok(Some(var_valuie)) = valuation.of_v_id(l.v_id) {
                var_valuie != l.polarity
            } else {
                false
            }
        })
    }

    fn find_unit_literal<T: Valuation>(&self, valuation: &T) -> Option<Literal> {
        let mut unit = None;

        for literal in self {
            if let Ok(assigned_value) = valuation.of_v_id(literal.v_id) {
                if assigned_value.is_some_and(|v| v == literal.polarity) {
                    // the clause is satisfied and so does not provide any new information
                    break;
                } else if assigned_value.is_some() {
                    // either every literal so far has been valued the opposite, or there has been exactly on unvalued literal, so continue
                    continue;
                } else {
                    // if no other literal has been found then this literal may be unit, so mark it and continue
                    // though, if some other literal has already been marked, the clause does not force any literal
                    match unit {
                        Some(_) => {
                            unit = None;
                            break;
                        }
                        None => unit = Some(*literal),
                    }
                }
            }
        }
        unit
    }

    fn collect_choices<T: Valuation>(&self, valuation: &T) -> Option<Vec<Literal>> {
        let mut the_literals = vec![];

        for literal in self {
            match valuation.of_v_id(literal.v_id) {
                Ok(assigned_value) => match assigned_value {
                    Some(value) => {
                        if value == literal.polarity {
                            return None;
                        } else {
                            continue;
                        }
                    }
                    None => the_literals.push(*literal),
                },
                Err(_) => panic!("Failed to get valuation of variable"),
            }
        }
        Some(the_literals)
    }

    fn as_string(&self) -> String {
        let mut the_string = String::from("(");
        for literal in self {
            the_string.push_str(format!(" {} ", literal).as_str())
        }
        the_string += ")";
        the_string
    }
}


impl StoredClause {
    pub fn new(id: usize, position: usize) -> StoredClause {
        StoredClause {
            id,
            position,
            clause: Vec::new(),
        }
    }

    pub fn add_literal(&mut self, literal: Literal) -> Result<(), ClauseError> {
        self.clause.push(literal);
        Ok(())
    }

    pub fn is_sat_on(&self, valuation: &ValuationVec) -> bool {
        self.clause
            .iter()
            .any(|l| valuation.of_v_id(l.v_id) == Ok(Some(l.polarity)))
    }

    pub fn is_unsat_on(&self, valuation: &ValuationVec) -> bool {
        self.clause.iter().all(|l| {
            if let Ok(Some(var_valuie)) = valuation.of_v_id(l.v_id) {
                var_valuie != l.polarity
            } else {
                false
            }
        })
    }

    pub fn find_unit_literal<T: Valuation>(&self, valuation: &T) -> Option<Literal> {
        let mut unit = None;

        for literal in &self.clause {
            if let Ok(assigned_value) = valuation.of_v_id(literal.v_id) {
                if assigned_value.is_some_and(|v| v == literal.polarity) {
                    // the clause is satisfied and so does not provide any new information
                    break;
                } else if assigned_value.is_some() {
                    // either every literal so far has been valued the opposite, or there has been exactly on unvalued literal, so continue
                    continue;
                } else {
                    // if no other literal has been found then this literal may be unit, so mark it and continue
                    // though, if some other literal has already been marked, the clause does not force any literal
                    match unit {
                        Some(_) => {
                            unit = None;
                            break;
                        }
                        None => unit = Some(*literal),
                    }
                }
            }
        }
        unit
    }

    /*
    If
     */
    pub fn collect_choices<T: Valuation>(&self, valuation: &T) -> Option<Vec<Literal>> {
        let mut the_literals = vec![];

        for literal in &self.clause {
            match valuation.of_v_id(literal.v_id) {
                Ok(assigned_value) => match assigned_value {
                    Some(value) => {
                        if value == literal.polarity {
                            return None;
                        } else {
                            continue;
                        }
                    }
                    None => the_literals.push(*literal),
                },
                Err(_) => panic!("Failed to get valuation of variable"),
            }
        }
        Some(the_literals)
    }
}
