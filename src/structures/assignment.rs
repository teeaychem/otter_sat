use crate::structures::Literal;

#[derive(Debug)]
pub struct Assignment {
    status: Vec<Option<bool>>,
}

impl std::fmt::Display for Assignment {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[")?;
        for (maybe_literal) in self.status.iter() {
            if let Some(literal) = maybe_literal {
                write!(f, "{}", literal)?
            } else {
                write!(f, " ❍ ")?
            }
        }
        write!(f, "]")
    }
}

impl Assignment {
    pub fn new(variable_count: usize) -> Self {
        Assignment {
            status: vec![None; variable_count + 1],
        }
    }

    pub fn get(&self, index: usize) -> Option<Option<bool>> {
        if let Some(&info) = self.status.get(index) {
            Some(info)
        } else {
            None
        }
    }

    pub fn set(&mut self, literal: Literal) {
        self.status[literal.variable()] = Some(literal.polarity())
}

    pub fn clear(&mut self, index: usize) {
        self.status[index] = None
    }
}
