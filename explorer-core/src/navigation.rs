use crate::filesystem::EPath;

#[derive(Debug, Clone)]
pub struct NavigationHistory {
    history: Vec<EPath>,
    history_index: usize,
}

impl NavigationHistory {
    pub fn new(initial: EPath) -> Self {
        Self {
            history: vec![initial],
            history_index: 0,
        }
    }

    pub fn can_go_back(&self) -> bool {
        self.history_index > 0
    }

    pub fn can_go_forward(&self) -> bool {
        self.history_index + 1 < self.history.len()
    }

    pub fn go_back(&mut self) -> Option<EPath> {
        if !self.can_go_back() {
            return None;
        }

        self.history_index -= 1;
        Some(self.history[self.history_index].clone())
    }

    pub fn go_forward(&mut self) -> Option<EPath> {
        if !self.can_go_forward() {
            return None;
        }

        self.history_index += 1;
        Some(self.history[self.history_index].clone())
    }

    pub fn push(&mut self, path: EPath) {
        if self.history.get(self.history_index) == Some(&path) {
            return;
        }

        self.history.truncate(self.history_index + 1);
        self.history.push(path);
        self.history_index = self.history.len() - 1;
    }
}
