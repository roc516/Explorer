mod cell;
mod columns;
mod header;
mod message;
mod resize;
mod row;
mod sort;
mod task;

pub use message::{Action, Message};
pub use task::load_directory_task;

use explorer_ui::{ids, ExplorerModel};
use iced::event;
use iced::widget::{column, container, rule, scrollable, text};
use iced::{Element, Fill, Subscription, Task};

use crate::fluent::{SPACE_LG, SPACE_XS, PAGE_PADDING_H};

use columns::{ActiveColumnResize, ColumnWidths};
use sort::{apply_sort, SortDirection, SortState};

pub struct FileListWidget {
    column_widths: ColumnWidths,
    column_resize: Option<ActiveColumnResize>,
    sort: SortState,
}

impl FileListWidget {
    pub fn new() -> Self {
        Self {
            column_widths: ColumnWidths::default(),
            column_resize: None,
            sort: SortState::default(),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        if self.column_resize.is_none() {
            return Subscription::none();
        }

        event::listen_with(resize::column_resize_listener)
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
                    Some(explorer_ui::OpenEntryAction::Navigate(path)) => (
                        load_directory_task(path.clone()),
                        Some(Action::DirectoryChanged(path)),
                    ),
                    Some(explorer_ui::OpenEntryAction::Preview(path)) => {
                        (Task::none(), Some(Action::PreviewFile(path)))
                    }
                    Some(explorer_ui::OpenEntryAction::OpenArchive(path)) => {
                        (Task::none(), Some(Action::OpenArchive(path)))
                    }
                    Some(explorer_ui::OpenEntryAction::OpenedSystem { .. }) => {
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
                    .map(|(path, _)| Action::DirectoryChanged(path.clone()));
                model.on_directory_loaded(result);
                apply_sort(model, self.sort);
                (Task::none(), action)
            }
            Message::ColumnResizeStarted(column) => {
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
            Message::ColumnSortClicked(column) => {
                if self.sort.column == column {
                    self.sort.direction = match self.sort.direction {
                        SortDirection::Ascending => SortDirection::Descending,
                        SortDirection::Descending => SortDirection::Ascending,
                    };
                } else {
                    self.sort.column = column;
                    self.sort.direction = SortDirection::Ascending;
                }
                apply_sort(model, self.sort);
                (Task::none(), None)
            }
        }
    }

    pub fn view<'a>(&self, model: &'a ExplorerModel) -> Element<'a, Message> {
        let bundle = model.bundle;
        let loading_label = bundle.tr(ids::STATUS_LOADING);
        let empty_label = bundle.tr(ids::FOLDER_EMPTY);
        let resizing = self.column_resize.map(|active| active.column);

        let header = header::view(
            bundle.tr(ids::COLUMN_NAME),
            bundle.tr(ids::COLUMN_MODIFIED),
            bundle.tr(ids::COLUMN_TYPE),
            bundle.tr(ids::COLUMN_SIZE),
            &self.column_widths,
            self.sort,
            resizing,
        );

        let list = if model.loading {
            column![container(text(loading_label).size(14)).padding([SPACE_LG, PAGE_PADDING_H])]
        } else if let Some(error) = model.error_text() {
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
                            &self.column_widths,
                        )
                    })
                    .collect::<Vec<_>>(),
            )
            .spacing(SPACE_XS)
            .padding([SPACE_XS, PAGE_PADDING_H])
        };

        column![
            header,
            rule::horizontal(1).style(header::list_header_rule),
            scrollable(list).height(Fill)
        ]
        .width(Fill)
        .height(Fill)
        .into()
    }
}

impl Default for FileListWidget {
    fn default() -> Self {
        Self::new()
    }
}
