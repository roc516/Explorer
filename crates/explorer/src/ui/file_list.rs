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

use explorer_app::{ids, ExplorerModel};
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

    pub fn subscription(&self) -> Subscription<Message> {
        if self.column_resize.is_some() {
            return event::listen_with(resize::column_resize_listener);
        }
        if self.column_reorder.is_some() {
            return event::listen_with(resize::column_reorder_listener);
        }
        Subscription::none()
    }

    fn begin_sort(&mut self, model: &ExplorerModel) -> Task<Message> {
        if model.entries.is_empty() {
            self.sorting = false;
            return Task::none();
        }
        self.sort_id = self.sort_id.wrapping_add(1);
        self.sorting = true;
        sort_entries_task(model, self.sort, self.sort_id)
    }

    fn apply_sort_click(&mut self, model: &ExplorerModel, column: columns::Column) -> Task<Message> {
        if self.sort.column == column {
            self.sort.direction = match self.sort.direction {
                SortDirection::Ascending => SortDirection::Descending,
                SortDirection::Descending => SortDirection::Ascending,
            };
        } else {
            self.sort.column = column;
            self.sort.direction = SortDirection::Ascending;
        }
        self.begin_sort(model)
    }

    pub fn update(
        &mut self,
        model: &mut ExplorerModel,
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
                (self.begin_sort(model), action)
            }
            Message::EntriesSorted {
                id,
                entries,
                selected_index,
            } => {
                if id == self.sort_id {
                    model.entries = entries;
                    model.selected_index = selected_index;
                    self.sorting = false;
                }
                (Task::none(), None)
            }
            Message::ColumnResizeStarted(column) => {
                self.column_reorder = None;
                self.column_resize = Some(ActiveColumnResize {
                    column,
                    last_x: None,
                });
                (Task::none(), None)
            }
            Message::ColumnResizeMoved(x) => {
                if let Some(active) = &mut self.column_resize {
                    if let Some(last_x) = active.last_x {
                        let delta = x - last_x;
                        let current = self.column_widths.get(active.column);
                        self.column_widths.set(active.column, current + delta);
                    }
                    active.last_x = Some(x);
                }
                (Task::none(), None)
            }
            Message::ColumnResizeEnded => {
                self.column_resize = None;
                (Task::none(), None)
            }
            Message::ColumnReorderStarted(column) => {
                if self.column_resize.is_some() {
                    return (Task::none(), None);
                }
                let Some(origin_index) = self.column_order.index_of(column) else {
                    return (Task::none(), None);
                };
                self.column_reorder = Some(ActiveColumnReorder {
                    column,
                    origin_index,
                    insert_at: origin_index.min(3),
                    start_x: None,
                    dragging: false,
                });
                (Task::none(), None)
            }
            Message::ColumnReorderMoved(x) => {
                let Some(mut active) = self.column_reorder else {
                    return (Task::none(), None);
                };
                let start_x = active.start_x.unwrap_or(x);
                active.start_x = Some(start_x);
                let dx = x - start_x;
                if !active.dragging && dx.abs() >= REORDER_DRAG_THRESHOLD {
                    active.dragging = true;
                }
                if active.dragging {
                    active.insert_at = self.column_order.insert_at_for_drag(
                        &self.column_widths,
                        active.origin_index,
                        dx,
                    );
                }
                self.column_reorder = Some(active);
                (Task::none(), None)
            }
            Message::ColumnReorderEnded => {
                let Some(active) = self.column_reorder.take() else {
                    return (Task::none(), None);
                };
                if active.dragging {
                    self.column_order
                        .move_to(active.origin_index, active.insert_at);
                }
                (Task::none(), None)
            }
            Message::ColumnSortClicked(column) => (self.apply_sort_click(model, column), None),
            Message::ColumnHandleHovered(column) => {
                self.hovered_column = Some(column);
                (Task::none(), None)
            }
            Message::ColumnHandleUnhovered(column) => {
                if self.hovered_column == Some(column) {
                    self.hovered_column = None;
                }
                (Task::none(), None)
            }
        }
    }

    pub fn view<'a>(&self, model: &'a ExplorerModel) -> Element<'a, Message> {
        let bundle = model.bundle;
        let empty_label = bundle.tr(ids::FOLDER_EMPTY);
        let resizing = self.column_resize.map(|active| active.column);
        let labels = ColumnLabels {
            name: bundle.tr(ids::COLUMN_NAME),
            modified: bundle.tr(ids::COLUMN_MODIFIED),
            type_: bundle.tr(ids::COLUMN_TYPE),
            size: bundle.tr(ids::COLUMN_SIZE),
        };

        let header = header::view(
            &labels,
            self.column_order,
            &self.column_widths,
            self.sort,
            resizing,
            self.column_reorder,
            self.hovered_column,
        );

        let body: Element<'a, Message> = if model.loading || self.sorting {
            crate::ui::loading::view_tr(bundle)
        } else {
            let list = if let Some(error) = model.error_text() {
                column![container(text(error).size(14)).padding([SPACE_LG, PAGE_PADDING_H])]
            } else if model.entries.is_empty() {
                column![container(text(empty_label).size(14)).padding([SPACE_LG, PAGE_PADDING_H])]
            } else {
                column(
                    model
                        .entries
                        .iter()
                        .enumerate()
                        .map(|(index, entry)| {
                            row::file_row(
                                index,
                                entry,
                                model.selected_index == Some(index),
                                &bundle,
                                self.column_order,
                                &self.column_widths,
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
}

impl Default for FileList {
    fn default() -> Self {
        Self::new()
    }
}
