use crate::structures::{literal::Literal, solve::Solve, variable::VariableId};

pub type ValuationVec = Vec<Option<bool>>;

pub trait Valuation {
    fn new_for_variables(variable_count: usize) -> Self;

    fn as_display_string(&self, solve: &Solve) -> String;

    fn as_internal_string(&self) -> String;

    fn of_v_id(&self, v_id: VariableId) -> Option<bool>;

    fn check_literal(&self, literal: Literal) -> ValuationStatus;

    fn update_value(&mut self, literal: Literal) -> Result<(), ValuationStatus>;

    fn to_vec(&self) -> ValuationVec;

    fn values(&self) -> impl Iterator<Item = Option<bool>>;
}

pub enum ValuationStatus {
    NotSet,
    Match,
    Conflict,
}

impl Valuation for ValuationVec {
    fn new_for_variables(variable_count: usize) -> Self {
        vec![None; variable_count]
    }

    fn as_display_string(&self, solve: &Solve) -> String {
        self.iter()
            .enumerate()
            .filter(|(_, p)| p.is_some())
            .map(|(i, p)| {
                let variable = solve.var_by_id(i as VariableId).unwrap();
                match p {
                    Some(true) => variable.name().to_string(),
                    Some(false) => format!("-{}", variable.name()),
                    _ => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn as_internal_string(&self) -> String {
        self.iter()
            .enumerate()
            .filter(|(_, p)| p.is_some())
            .map(|(i, p)| match p {
                Some(true) => format!("{}", i),
                Some(false) => format!("-{}", i),
                _ => String::new(),
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn of_v_id(&self, v_id: VariableId) -> Option<bool> {
        unsafe { *self.get_unchecked(v_id) }
    }

    fn check_literal(&self, literal: Literal) -> ValuationStatus {
        unsafe {
            match self.get_unchecked(literal.v_id) {
                Some(already_set) if *already_set == literal.polarity => ValuationStatus::Match,
                Some(_already_set) => ValuationStatus::Conflict,
                None => ValuationStatus::NotSet,
            }
        }
    }

    fn update_value(&mut self, literal: Literal) -> Result<(), ValuationStatus> {
        log::trace!("Set literal: {}", literal);
        unsafe {
            match self.get_unchecked(literal.v_id) {
                Some(value) if *value != literal.polarity => Err(ValuationStatus::Conflict),
                Some(_value) => Err(ValuationStatus::Match),
                None => {
                    *self.get_unchecked_mut(literal.v_id) = Some(literal.polarity);

                    Ok(())
                }
            }
        }
    }

    fn to_vec(&self) -> ValuationVec {
        self.clone()
    }

    fn values(&self) -> impl Iterator<Item = Option<bool>> {
        self.iter().cloned()
    }
}
