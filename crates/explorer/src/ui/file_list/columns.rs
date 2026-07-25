use crate::fluent::SPACE_SM;

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
pub(crate) const REORDER_DRAG_THRESHOLD: f32 = 6.0;

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

    /// Visible header/body width for a column (Name includes icon).
    pub fn display_width(&self, column: Column) -> f32 {
        match column {
            Column::Name => COL_ICON + self.name,
            other => self.get(other),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ColumnOrder([Column; 4]);

impl Default for ColumnOrder {
    fn default() -> Self {
        Self([
            Column::Name,
            Column::Modified,
            Column::Type,
            Column::Size,
        ])
    }
}

impl ColumnOrder {
    pub fn as_slice(&self) -> &[Column; 4] {
        &self.0
    }

    pub fn index_of(&self, column: Column) -> Option<usize> {
        self.0.iter().position(|&c| c == column)
    }

    /// Move column at `from` so it lands at `to` in the list after removal (`to` in `0..=3`).
    pub fn move_to(&mut self, from: usize, to: usize) {
        if from >= self.0.len() {
            return;
        }
        let mut cols: Vec<Column> = self.0.to_vec();
        let column = cols.remove(from);
        let to = to.min(cols.len());
        cols.insert(to, column);
        self.0 = cols.try_into().expect("column order length is fixed");
    }

    /// Insertion index after removing `origin_index`, based on drag displacement `dx`.
    pub fn insert_at_for_drag(
        &self,
        widths: &ColumnWidths,
        origin_index: usize,
        dx: f32,
    ) -> usize {
        let mut starts = [0.0f32; 4];
        let mut x = 0.0;
        for (i, &column) in self.0.iter().enumerate() {
            starts[i] = x;
            x += widths.display_width(column) + SPACE_SM;
        }

        let origin_width = widths.display_width(self.0[origin_index]);
        let cursor = starts[origin_index] + origin_width * 0.5 + dx;

        let remaining: Vec<(f32, f32)> = self
            .0
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != origin_index)
            .map(|(i, &column)| (starts[i], widths.display_width(column)))
            .collect();

        for (slot, (start, width)) in remaining.iter().enumerate() {
            let mid = start + width * 0.5;
            if cursor < mid {
                return slot;
            }
        }
        remaining.len()
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ActiveColumnResize {
    pub column: Column,
    pub last_x: Option<f32>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ActiveColumnReorder {
    pub column: Column,
    pub origin_index: usize,
    /// Destination index after removing the dragged column (`0..=3`).
    pub insert_at: usize,
    pub start_x: Option<f32>,
    pub dragging: bool,
}
