mod cell;
mod columns;
mod header;
mod message;
mod resize;
mod row;
mod sort;
mod task;

pub use message::{Action, Message};
pub use task::load_directory_from_dir;

use explorer_app::{ids, ExplorerState};
use iced::event;
use iced::widget::{column, container, rule, scrollable, text};
use iced::{Element, Fill, Subscription, Task};

use crate::fluent::{PAGE_PADDING_H, SPACE_LG, SPACE_XS};

use columns::{
    ActiveColumnReorder, ActiveColumnResize, Column, ColumnOrder, ColumnWidths,
    REORDER_DRAG_THRESHOLD,
};
use header::ColumnLabels;
use sort::{sort_entries_task, SortDirection, SortState};

pub struct FileList {
    column_widths: ColumnWidths,
    column_order: ColumnOrder,
    column_resize: Option<ActiveColumnResize>,
    column_reorder: Option<ActiveColumnReorder>,
    hovered_column: Option<Column>,
    sort: SortState,
    sort_id: u64,
    sorting: bool,
}

impl FileList {
    pub fn new() -> Self {
        Self {
            column_widths: ColumnWidths::default(),
            column_order: ColumnOrder::default(),
            column_resize: None,
            column_reorder: None,
            hovered_column: None,
            sort: SortState::default(),
            sort_id: 0,
            sorting: false,
        }
    }
}

impl Default for FileList {
    fn default() -> Self {
        Self::new()
    }
}

pub fn subscription(ui: &FileList) -> Subscription<Message> {
    if ui.column_resize.is_some() {
        return event::listen_with(resize::column_resize_listener);
    }
    if ui.column_reorder.is_some() {
        return event::listen_with(resize::column_reorder_listener);
    }
    Subscription::none()
}

fn begin_sort(ui: &mut FileList, model: &ExplorerState) -> Task<Message> {
    if model.file_list.entries.is_empty() {
        ui.sorting = false;
        return Task::none();
    }
    ui.sort_id = ui.sort_id.wrapping_add(1);
    ui.sorting = true;
    sort_entries_task(model, ui.sort, ui.sort_id)
}

fn apply_sort_click(ui: &mut FileList, model: &ExplorerState, column: columns::Column) -> Task<Message> {
    if ui.sort.column == column {
        ui.sort.direction = match ui.sort.direction {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        };
    } else {
        ui.sort.column = column;
        ui.sort.direction = SortDirection::Ascending;
    }
    begin_sort(ui, model)
}

pub fn update(
    ui: &mut FileList,
    model: &mut ExplorerState,
    message: Message,
) -> (Task<Message>, Option<Action>) {
    match message {
        Message::EntryClicked(index) => {
            model.select_entry(index);
            (Task::none(), None)
        }
        Message::EntryDoubleClicked(index) => {
            let action = model.open_entry(index);
            let (task, file_action) = match action {
                Some(explorer_app::OpenEntryAction::Navigate(dir)) => {
                    let path = dir.path().to_path_buf();
                    (
                        load_directory_from_dir(dir),
                        Some(Action::Navigated(path)),
                    )
                }
                Some(explorer_app::OpenEntryAction::Preview(entry)) => {
                    (Task::none(), Some(Action::PreviewFile(entry)))
                }
                Some(explorer_app::OpenEntryAction::OpenArchive(path)) => {
                    (Task::none(), Some(Action::OpenArchive(path)))
                }
                Some(explorer_app::OpenEntryAction::OpenedSystem { .. }) => {
                    (Task::none(), None)
                }
                None => (Task::none(), None),
            };

            (task, file_action)
        }
        Message::DirectoryLoaded(result) => {
            let action = result
                .as_ref()
                .ok()
                .map(|(dir, _)| Action::DirectoryLoaded(dir.path().to_path_buf()));
            model.on_directory_loaded(result);
            (begin_sort(ui, model), action)
        }
        Message::EntriesSorted {
            id,
            entries,
            selected_index,
        } => {
            if id == ui.sort_id {
                model.file_list.entries = entries;
                model.file_list.selected_index = selected_index;
                ui.sorting = false;
            }
            (Task::none(), None)
        }
        Message::ColumnResizeStarted(column) => {
            ui.column_reorder = None;
            ui.column_resize = Some(ActiveColumnResize {
                column,
                last_x: None,
            });
            (Task::none(), None)
        }
        Message::ColumnResizeMoved(x) => {
            if let Some(active) = &mut ui.column_resize {
                if let Some(last_x) = active.last_x {
                    let delta = x - last_x;
                    let current = ui.column_widths.get(active.column);
                    ui.column_widths.set(active.column, current + delta);
                }
                active.last_x = Some(x);
            }
            (Task::none(), None)
        }
        Message::ColumnResizeEnded => {
            ui.column_resize = None;
            (Task::none(), None)
        }
        Message::ColumnReorderStarted(column) => {
            if ui.column_resize.is_some() {
                return (Task::none(), None);
            }
            let Some(origin_index) = ui.column_order.index_of(column) else {
                return (Task::none(), None);
            };
            ui.column_reorder = Some(ActiveColumnReorder {
                column,
                origin_index,
                insert_at: origin_index.min(3),
                start_x: None,
                dragging: false,
            });
            (Task::none(), None)
        }
        Message::ColumnReorderMoved(x) => {
            let Some(mut active) = ui.column_reorder else {
                return (Task::none(), None);
            };
            let start_x = active.start_x.unwrap_or(x);
            active.start_x = Some(start_x);
            let dx = x - start_x;
            if !active.dragging && dx.abs() >= REORDER_DRAG_THRESHOLD {
                active.dragging = true;
            }
            if active.dragging {
                active.insert_at = ui.column_order.insert_at_for_drag(
                    &ui.column_widths,
                    active.origin_index,
                    dx,
                );
            }
            ui.column_reorder = Some(active);
            (Task::none(), None)
        }
        Message::ColumnReorderEnded => {
            let Some(active) = ui.column_reorder.take() else {
                return (Task::none(), None);
            };
            if active.dragging {
                ui.column_order
                    .move_to(active.origin_index, active.insert_at);
            }
            (Task::none(), None)
        }
        Message::ColumnSortClicked(column) => (apply_sort_click(ui, model, column), None),
        Message::ColumnHandleHovered(column) => {
            ui.hovered_column = Some(column);
            (Task::none(), None)
        }
        Message::ColumnHandleUnhovered(column) => {
            if ui.hovered_column == Some(column) {
                ui.hovered_column = None;
            }
            (Task::none(), None)
        }
    }
}

pub fn view<'a>(ui: &FileList, model: &'a ExplorerState) -> Element<'a, Message> {
    let bundle = model.bundle;
    let empty_label = bundle.tr(ids::FOLDER_EMPTY);
    let resizing = ui.column_resize.map(|active| active.column);
    let labels = ColumnLabels {
        name: bundle.tr(ids::COLUMN_NAME),
        modified: bundle.tr(ids::COLUMN_MODIFIED),
        type_: bundle.tr(ids::COLUMN_TYPE),
        size: bundle.tr(ids::COLUMN_SIZE),
    };

    let header = header::view(
        &labels,
        ui.column_order,
        &ui.column_widths,
        ui.sort,
        resizing,
        ui.column_reorder,
        ui.hovered_column,
    );

    let body: Element<'a, Message> = if model.file_list.loading || ui.sorting {
        crate::ui::loading::view_tr(bundle)
    } else {
        let list = if let Some(error) = model.error_text() {
            column![container(text(error).size(14)).padding([SPACE_LG, PAGE_PADDING_H])]
        } else if model.file_list.entries.is_empty() {
            column![container(text(empty_label).size(14)).padding([SPACE_LG, PAGE_PADDING_H])]
        } else {
            column(
                model
                    .file_list
                    .entries
                    .iter()
                    .enumerate()
                    .map(|(index, entry)| {
                        row::file_row(
                            index,
                            entry,
                            model.file_list.selected_index == Some(index),
                            &bundle,
                            ui.column_order,
                            &ui.column_widths,
                        )
                    })
                    .collect::<Vec<_>>(),
            )
            .spacing(SPACE_XS)
            .padding([SPACE_XS, PAGE_PADDING_H])
        };
        scrollable(list).height(Fill).into()
    };

    column![
        header,
        rule::horizontal(1).style(header::list_header_rule),
        body,
    ]
    .width(Fill)
    .height(Fill)
    .into()
}
