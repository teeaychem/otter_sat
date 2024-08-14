pub type Variable = usize;

#[derive(Clone, Copy, Debug)]
pub struct Literal {
    variable: Variable,
    polarity: bool,
}

#[derive(Debug)]
pub enum LiteralError {
    NoVariable,
    NoFirst,
    BadStart,
    BadVariable,
    UnobtainableVariable,
    ZeroVariable,
}

impl std::fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.polarity {
            true => write!(f, "{}", self.variable),
            false => write!(f, "-{}", self.variable),
        }
    }
}

impl Literal {
    pub fn from_string(string: &str) -> Result<Literal, LiteralError> {
        if string.is_empty() || string == "-" {
            return Err(LiteralError::NoVariable);
        };
        if let Some(first) = string.chars().nth(0) {
            if first != '-' && !first.is_numeric() {
                return Err(LiteralError::BadStart);
            };
            if first == '0' {
                return Err(LiteralError::ZeroVariable);
            }
        } else {
            return Err(LiteralError::NoFirst);
        }

        let polarity = string.chars().nth(0) != Some('-');
        let variable_slice = if polarity {
            string.get(0..)
        } else {
            string.get(1..)
        };
        if let Some(variable_string) = variable_slice {
            if let Ok(variable) = variable_string.parse::<Variable>() {
                Ok(Literal { variable, polarity })
            } else {
                Err(LiteralError::BadVariable)
            }
        } else {
            Err(LiteralError::UnobtainableVariable)
        }
    }

    pub fn variable(&self) -> Variable {
        self.variable
    }

    pub fn polarity(&self) -> bool {
        self.polarity
    }
}

impl PartialOrd for Literal {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Literal {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.variable == other.variable {
            if self.polarity == other.polarity {
                std::cmp::Ordering::Equal
            } else if self.polarity {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        } else {
            self.variable.cmp(&other.variable)
        }
    }
}

impl PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        self.variable == other.variable && self.polarity == other.polarity
    }
}

impl Eq for Literal {}
