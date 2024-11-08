use crate::context::stores::{
    level::{Level, LevelStore},
    LevelIndex,
};

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

    pub fn top_mut(&mut self) -> &mut Level {
        let index = self.index();
        unsafe { self.levels.get_unchecked_mut(index) }
    }

    pub fn zero(&self) -> &Level {
        unsafe { self.levels.get_unchecked(0) }
    }

    pub fn zero_mut(&mut self) -> &mut Level {
        unsafe { self.levels.get_unchecked_mut(0) }
    }

    pub fn pop(&mut self) -> Option<Level> {
        self.levels.pop()
    }
}
