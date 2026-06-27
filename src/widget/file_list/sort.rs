use std::cmp::Ordering;

use explorer_ui::{ExplorerModel, FileEntry};

use super::columns::Column;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SortState {
    pub column: Column,
    pub direction: SortDirection,
}

impl Default for SortState {
    fn default() -> Self {
        Self {
            column: Column::Name,
            direction: SortDirection::Ascending,
        }
    }
}

pub(crate) fn apply_sort(model: &mut ExplorerModel, sort: SortState) {
    if model.entries.is_empty() {
        return;
    }

    let selected_path = model
        .selected_index
        .and_then(|index| model.entries.get(index).map(|entry| entry.path.clone()));

    model
        .entries
        .sort_by(|left, right| compare_entries(left, right, sort, &model.bundle));

    model.selected_index = selected_path.and_then(|path| {
        model
            .entries
            .iter()
            .position(|entry| entry.path == path)
    });
}

fn compare_entries(
    left: &FileEntry,
    right: &FileEntry,
    sort: SortState,
    bundle: &explorer_ui::LanguageBundle,
) -> Ordering {
    let folder_order = match (left.is_dir, right.is_dir) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        _ => Ordering::Equal,
    };

    let column_order = if folder_order != Ordering::Equal {
        folder_order
    } else {
        match sort.column {
            Column::Name => left
                .name
                .to_lowercase()
                .cmp(&right.name.to_lowercase()),
            Column::Modified => match (left.modified, right.modified) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Greater,
                (Some(_), None) => Ordering::Less,
                (Some(left_time), Some(right_time)) => left_time.cmp(&right_time),
            },
            Column::Type => left
                .type_label(bundle)
                .to_lowercase()
                .cmp(&right.type_label(bundle).to_lowercase()),
            Column::Size => left.size.cmp(&right.size),
        }
    };

    let tie_breaker = left
        .name
        .to_lowercase()
        .cmp(&right.name.to_lowercase());

    let ordering = column_order.then(tie_breaker);

    match sort.direction {
        SortDirection::Ascending => ordering,
        SortDirection::Descending => ordering.reverse(),
    }
}
