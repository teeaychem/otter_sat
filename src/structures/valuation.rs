use crate::structures::{literal::Literal, solve::Solve, variable::VariableId};

use std::ops::DerefMut;

pub type ValuationVec = Vec<Option<bool>>;
pub type ValuationBox = Box<[Option<bool>]>;

pub trait Valuation {
    fn as_display_string(&self, solve: &Solve) -> String;

    fn as_internal_string(&self) -> String;

    fn of_v_id(&self, v_id: VariableId) -> Option<bool>;

    fn of_index(&self, index: usize) -> Option<bool>;

    fn check_literal(&self, literal: Literal) -> Status;

    fn update_value(&mut self, literal: Literal) -> Result<(), Status>;

    fn set_value(&mut self, literal: Literal);

    fn values(&self) -> impl Iterator<Item = Option<bool>>;

    fn slice(&self) -> &[Option<bool>];
}

pub enum Status {
    NotSet,
    Match,
    Conflict,
}

impl<T: ?Sized + DerefMut<Target = [Option<bool>]>> Valuation for T {
    fn as_display_string(&self, solve: &Solve) -> String {
        self.iter()
            .enumerate()
            .filter(|(_, p)| p.is_some())
            .map(|(i, p)| {
                let variable = solve.variables().get(i).unwrap();
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
            .map(|(index, p)| match p {
                Some(true) => format!("{index}"),
                Some(false) => format!("-{index}"),
                _ => String::new(),
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn of_v_id(&self, v_id: VariableId) -> Option<bool> {
        unsafe { *self.get_unchecked(v_id as usize) }
    }

    fn of_index(&self, index: usize) -> Option<bool> {
        unsafe { *self.get_unchecked(index) }
    }

    fn check_literal(&self, literal: Literal) -> Status {
        let maybe_value = unsafe { self.get_unchecked(literal.index()) };
        match maybe_value {
            Some(already_set) if *already_set == literal.polarity() => Status::Match,
            Some(_already_set) => Status::Conflict,
            None => Status::NotSet,
        }
    }

    fn update_value(&mut self, literal: Literal) -> Result<(), Status> {
        log::trace!("Set literal: {}", literal);
        let maybe_value = unsafe { self.get_unchecked(literal.index()) };
        match maybe_value {
            Some(value) if *value != literal.polarity() => Err(Status::Conflict),
            Some(_value) => Err(Status::Match),
            None => unsafe {
                *self.get_unchecked_mut(literal.index()) = Some(literal.polarity());
                Ok(())
            },
        }
    }

    fn set_value(&mut self, literal: Literal) {
        log::trace!("Set literal: {}", literal);
        unsafe { *self.get_unchecked_mut(literal.index()) = Some(literal.polarity()) }
    }

    fn values(&self) -> impl Iterator<Item = Option<bool>> {
        self.iter().copied()
    }

    fn slice(&self) -> &[Option<bool>] {
        &self
}
}

// impl Valuation for [Option<bool>] {
//     fn as_display_string(&self, solve: &Solve) -> String {
//         self.iter()
//             .enumerate()
//             .filter(|(_, p)| p.is_some())
//             .map(|(i, p)| {
//                 let variable = solve.variables().get(i).unwrap();
//                 match p {
//                     Some(true) => variable.name().to_string(),
//                     Some(false) => format!("-{}", variable.name()),
//                     _ => String::new(),
//                 }
//             })
//             .collect::<Vec<_>>()
//             .join(" ")
//     }

//     fn as_internal_string(&self) -> String {
//         self.iter()
//             .enumerate()
//             .filter(|(_, p)| p.is_some())
//             .map(|(index, p)| match p {
//                 Some(true) => format!("{index}"),
//                 Some(false) => format!("-{index}"),
//                 _ => String::new(),
//             })
//             .collect::<Vec<_>>()
//             .join(" ")
//     }

//     fn of_v_id(&self, v_id: VariableId) -> Option<bool> {
//         unsafe { *self.get_unchecked(v_id as usize) }
//     }

//     fn of_index(&self, index: usize) -> Option<bool> {
//         unsafe { *self.get_unchecked(index) }
//     }

//     fn check_literal(&self, literal: Literal) -> Status {
//         let maybe_value = unsafe { self.get_unchecked(literal.index()) };
//         match maybe_value {
//             Some(already_set) if *already_set == literal.polarity() => Status::Match,
//             Some(_already_set) => Status::Conflict,
//             None => Status::NotSet,
//         }
//     }

//     fn update_value(&mut self, literal: Literal) -> Result<(), Status> {
//         log::trace!("Set literal: {}", literal);
//         let maybe_value = unsafe { self.get_unchecked(literal.index()) };
//         match maybe_value {
//             Some(value) if *value != literal.polarity() => Err(Status::Conflict),
//             Some(_value) => Err(Status::Match),
//             None => unsafe {
//                 *self.get_unchecked_mut(literal.index()) = Some(literal.polarity());
//                 Ok(())
//             },
//         }
//     }

//     fn set_value(&mut self, literal: Literal) {
//         log::trace!("Set literal: {}", literal);
//         unsafe { *self.get_unchecked_mut(literal.index()) = Some(literal.polarity()) }
//     }

//     fn values(&self) -> impl Iterator<Item = Option<bool>> {
//         self.iter().copied()
//     }
// }

// impl Valuation for Box<[Option<bool>]> {
//     fn as_display_string(&self, solve: &Solve) -> String {
//         self.iter()
//             .enumerate()
//             .filter(|(_, p)| p.is_some())
//             .map(|(i, p)| {
//                 let variable = solve.variables().get(i).unwrap();
//                 match p {
//                     Some(true) => variable.name().to_string(),
//                     Some(false) => format!("-{}", variable.name()),
//                     _ => String::new(),
//                 }
//             })
//             .collect::<Vec<_>>()
//             .join(" ")
//     }

//     fn as_internal_string(&self) -> String {
//         self.iter()
//             .enumerate()
//             .filter(|(_, p)| p.is_some())
//             .map(|(index, p)| match p {
//                 Some(true) => format!("{index}"),
//                 Some(false) => format!("-{index}"),
//                 _ => String::new(),
//             })
//             .collect::<Vec<_>>()
//             .join(" ")
//     }

//     fn of_v_id(&self, v_id: VariableId) -> Option<bool> {
//         unsafe { *self.get_unchecked(v_id as usize) }
//     }

//     fn of_index(&self, index: usize) -> Option<bool> {
//         unsafe { *self.get_unchecked(index) }
//     }

//     fn check_literal(&self, literal: Literal) -> Status {
//         let maybe_value = unsafe { self.get_unchecked(literal.index()) };
//         match maybe_value {
//             Some(already_set) if *already_set == literal.polarity() => Status::Match,
//             Some(_already_set) => Status::Conflict,
//             None => Status::NotSet,
//         }
//     }

//     fn update_value(&mut self, literal: Literal) -> Result<(), Status> {
//         log::trace!("Set literal: {}", literal);
//         let maybe_value = unsafe { self.get_unchecked(literal.index()) };
//         match maybe_value {
//             Some(value) if *value != literal.polarity() => Err(Status::Conflict),
//             Some(_value) => Err(Status::Match),
//             None => unsafe {
//                 *self.get_unchecked_mut(literal.index()) = Some(literal.polarity());
//                 Ok(())
//             },
//         }
//     }

//     fn set_value(&mut self, literal: Literal) {
//         log::trace!("Set literal: {}", literal);
//         unsafe { *self.get_unchecked_mut(literal.index()) = Some(literal.polarity()) }
//     }

//     fn values(&self) -> impl Iterator<Item = Option<bool>> {
//         self.iter().copied()
//     }
// }
