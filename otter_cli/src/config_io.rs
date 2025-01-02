use std::path::PathBuf;

pub const DETAILS: u8 = 0;

#[derive(Clone)]
pub struct ConfigIO {
    pub files: Vec<PathBuf>,
    pub detail: u8,
    pub show_core: bool,
    pub show_stats: bool,
    pub show_valuation: bool,
    pub frat: bool,
    pub frat_path: Option<PathBuf>,
}

impl Default for ConfigIO {
    fn default() -> Self {
        ConfigIO {
            files: Vec::default(),
            detail: DETAILS,
            show_core: false,
            show_stats: false,
            show_valuation: false,
            frat: true,
            frat_path: None,
        }
    }
}
