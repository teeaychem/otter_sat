/// Boolean valued context configurations
///
/// When set to true things related to the identifier are enabled.
#[derive(Clone)]
pub struct Switches {
    /// Default to th last set value of a atom when choosing  a value for the atom, otherwise decision with specified probability.
    pub phase_saving: bool,

    /// Enable preprocessing of 𝐅.
    pub preprocessing: bool,

    /// Permit (scheduled) restarts.
    pub restart: bool,

    /// Permit subsumption of formulas.
    pub subsumption: bool,
}

impl Default for Switches {
    fn default() -> Self {
        Switches {
            phase_saving: true,
            preprocessing: false,
            restart: true,
            subsumption: false,
        }
    }
}
