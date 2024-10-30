use crate::context::level::{Level, LevelIndex, LevelStore};

impl Default for LevelStore {
    fn default() -> Self {
        let mut the_store = LevelStore {
            levels: Vec::default(),
        };
        the_store.levels.push(Level::new(0));
        the_store
    }
}

impl LevelStore {
    pub fn with_capacity(capacity: usize) -> Self {
        let mut the_store = LevelStore {
            levels: Vec::with_capacity(capacity),
        };
        the_store.levels.push(Level::new(0));
        the_store
    }

    pub fn get(&self, index: LevelIndex) -> &Level {
        self.levels.get(index).expect("mising level")
    }

    pub fn get_mut(&mut self, index: LevelIndex) -> &mut Level {
        self.levels.get_mut(index).expect("mising level")
    }

    pub fn index(&self) -> usize {
        self.levels.len() - 1
    }

    pub fn get_fresh(&mut self) -> LevelIndex {
        let index = self.levels.len();
        self.levels.push(Level::new(index));
        index
    }

    pub fn top(&self) -> &Level {
        unsafe { self.levels.get_unchecked(self.index()) }
    }

    pub fn pop(&mut self) -> Option<Level> {
        self.levels.pop()
    }
}
