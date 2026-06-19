use std::path::PathBuf;

use explorer_core::{ids, ExplorerModel, FileEntry};
use iced::widget::{column, container, mouse_area, row, rule, scrollable, text};
use iced::{alignment, Element, Fill, Length, Task, Theme};
use lucide_icons::Icon;

use crate::fluent::{PAGE_PADDING_H, RADIUS_CONTROL, SPACE_LG, SPACE_SM, SPACE_XS};
use crate::widget::lucide_icon;

#[derive(Debug, Clone)]
pub enum Message {
    EntryClicked(usize),
    EntryDoubleClicked(usize),
    DirectoryLoaded(Result<(PathBuf, Vec<FileEntry>), String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    DirectoryChanged(std::path::PathBuf),
}

pub struct FileListWidget;

impl FileListWidget {
    pub fn new() -> Self {
        Self
    }

    pub fn update(
        &self,
        model: &mut ExplorerModel,
        message: Message,
    ) -> (Task<Message>, Option<Action>) {
        match message {
            Message::EntryClicked(index) => {
                model.select_entry(index);
                (Task::none(), None)
            }
            Message::EntryDoubleClicked(index) => {
                let task = model
                    .open_entry(index)
                    .map(load_directory_task)
                    .unwrap_or_else(Task::none);
                (task, None)
            }
            Message::DirectoryLoaded(result) => {
                let action = result
                    .as_ref()
                    .ok()
                    .map(|(path, _)| Action::DirectoryChanged(path.clone()));
                model.on_directory_loaded(result);
                (Task::none(), action)
            }
        }
    }

    pub fn view<'a>(&self, model: &'a ExplorerModel) -> Element<'a, Message> {
        let bundle = model.bundle;
        let column_name = bundle.tr(ids::COLUMN_NAME);
        let column_modified = bundle.tr(ids::COLUMN_MODIFIED);
        let column_type = bundle.tr(ids::COLUMN_TYPE);
        let column_size = bundle.tr(ids::COLUMN_SIZE);
        let loading_label = bundle.tr(ids::STATUS_LOADING);
        let empty_label = bundle.tr(ids::FOLDER_EMPTY);

        let header = container(
            row![
                header_cell(column_name, 280.0),
                header_cell(column_modified, 160.0),
                header_cell(column_type, 120.0),
                header_cell(column_size, 90.0),
            ]
            .width(Fill),
        )
        .padding([SPACE_SM, PAGE_PADDING_H]);

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
                        file_row(index, entry, model.selected_index == Some(index), &bundle)
                    })
                    .collect::<Vec<_>>(),
            )
            .spacing(SPACE_XS)
            .padding([SPACE_XS, PAGE_PADDING_H])
        };

        column![header, rule::horizontal(1), scrollable(list).height(Fill)]
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

pub fn load_directory_task(path: std::path::PathBuf) -> Task<Message> {
    use explorer_core::read_directory;
    Task::perform(
        async move { read_directory(&path).map(|entries| (path, entries)) },
        Message::DirectoryLoaded,
    )
}

fn file_row<'a>(
    index: usize,
    entry: &'a FileEntry,
    selected: bool,
    bundle: &explorer_core::LanguageBundle,
) -> Element<'a, Message> {
    let content = row![
        container(lucide_icon::icon::<Message>(
            if entry.is_dir {
                Icon::Folder
            } else {
                Icon::File
            },
            16.0,
        ))
        .width(Length::Fixed(24.0))
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center),
        text(&entry.name).width(Length::Fixed(248.0)).size(14),
        text(entry.modified_label(bundle))
            .width(Length::Fixed(160.0))
            .size(13),
        text(entry.type_label(bundle))
            .width(Length::Fixed(120.0))
            .size(13),
        text(entry.size_label(bundle)).width(Length::Fixed(90.0)).size(13),
    ]
    .spacing(SPACE_SM)
    .align_y(alignment::Vertical::Center)
    .width(Fill);

    mouse_area(
        container(content)
            .padding([SPACE_XS, SPACE_SM])
            .width(Fill)
            .style(if selected {
                selected_row
            } else {
                normal_row
            }),
    )
    .on_press(Message::EntryClicked(index))
    .on_double_click(Message::EntryDoubleClicked(index))
    .into()
}

fn header_cell(label: String, width: f32) -> Element<'static, Message> {
    text(label)
        .size(12)
        .width(Length::Fixed(width))
        .into()
}

fn selected_row(theme: &Theme) -> iced::widget::container::Style {
    let palette = theme.extended_palette();
    iced::widget::container::Style {
        background: Some(iced::Background::Color(palette.primary.weak.color)),
        border: iced::Border {
            radius: RADIUS_CONTROL.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn normal_row(_theme: &Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        border: iced::Border {
            radius: RADIUS_CONTROL.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}
