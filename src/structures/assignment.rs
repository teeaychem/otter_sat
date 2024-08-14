#[derive(Debug)]
pub struct Assignment {
    status: Vec<Option<bool>>
}

impl Assignment {

    pub fn new(variable_count: usize) -> Self {
        Assignment {
            status: vec![None; variable_count]
        }
    }

    pub fn get(&self, index: usize) -> Option<Option<bool>> {
        if let Some(&info) = self.status.get(index) {
            Some(info)
        } else {
            None
        }
    }

    pub fn clear(&mut self, index: usize) {
        self.status[index] = None
    }
}
