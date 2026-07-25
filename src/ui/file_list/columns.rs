#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Column {
    Name,
    Modified,
    Type,
    Size,
}

pub(crate) const COL_ICON: f32 = 24.0;
pub(crate) const DEFAULT_COL_NAME: f32 = 248.0;
pub(crate) const DEFAULT_COL_MODIFIED: f32 = 160.0;
pub(crate) const DEFAULT_COL_TYPE: f32 = 120.0;
pub(crate) const DEFAULT_COL_SIZE: f32 = 90.0;
pub(crate) const MIN_COL_WIDTH: f32 = 48.0;

#[derive(Debug, Clone)]
pub(crate) struct ColumnWidths {
    pub name: f32,
    pub modified: f32,
    pub type_: f32,
    pub size: f32,
}

impl Default for ColumnWidths {
    fn default() -> Self {
        Self {
            name: DEFAULT_COL_NAME,
            modified: DEFAULT_COL_MODIFIED,
            type_: DEFAULT_COL_TYPE,
            size: DEFAULT_COL_SIZE,
        }
    }
}

impl ColumnWidths {
    pub fn get(&self, column: Column) -> f32 {
        match column {
            Column::Name => self.name,
            Column::Modified => self.modified,
            Column::Type => self.type_,
            Column::Size => self.size,
        }
    }

    pub fn set(&mut self, column: Column, width: f32) {
        let width = width.max(MIN_COL_WIDTH);
        match column {
            Column::Name => self.name = width,
            Column::Modified => self.modified = width,
            Column::Type => self.type_ = width,
            Column::Size => self.size = width,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ActiveColumnResize {
    pub column: Column,
    pub last_x: Option<f32>,
}
