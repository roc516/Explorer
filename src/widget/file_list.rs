use std::path::PathBuf;

use explorer_core::{ids, ExplorerModel, FileEntry};
use iced::widget::{column, container, mouse_area, row, rule, scrollable, text, Space};
use iced::widget::text::Wrapping;
use iced::{alignment, Element, Fill, Length, Task, Theme};
use lucide_icons::Icon;

use crate::fluent::{PAGE_PADDING_H, RADIUS_CONTROL, SPACE_LG, SPACE_SM, SPACE_XS};
use crate::widget::lucide_icon;

const COL_ICON: f32 = 24.0;
const COL_NAME: f32 = 248.0;
const COL_MODIFIED: f32 = 160.0;
const COL_TYPE: f32 = 120.0;
const COL_SIZE: f32 = 90.0;
const ELLIPSIS: char = '…';

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
                Space::new().width(Length::Fixed(COL_ICON)),
                header_cell(column_name, COL_NAME),
                header_cell(column_modified, COL_MODIFIED),
                header_cell(column_type, COL_TYPE),
                header_cell(column_size, COL_SIZE),
            ]
            .spacing(SPACE_SM)
            .align_y(alignment::Vertical::Center)
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
    let modified = entry.modified_label(bundle);
    let type_label = entry.type_label(bundle);
    let size = entry.size_label(bundle);

    let content = row![
        container(lucide_icon::icon::<Message>(
            if entry.is_dir {
                Icon::Folder
            } else {
                Icon::File
            },
            16.0,
        ))
        .width(Length::Fixed(COL_ICON))
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center),
        clipped_cell(&entry.name, COL_NAME, 14.0),
        clipped_cell(modified, COL_MODIFIED, 13.0),
        clipped_cell(type_label, COL_TYPE, 13.0),
        clipped_cell(size, COL_SIZE, 13.0),
        Space::new().width(Fill),
    ]
    .spacing(SPACE_SM)
    .align_y(alignment::Vertical::Center)
    .width(Fill);

    mouse_area(
        container(content)
            .padding([SPACE_XS, 0.0])
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
    clipped_cell(label, width, 12.0)
}

fn clipped_cell(label: impl AsRef<str>, width: f32, size: f32) -> Element<'static, Message> {
    let display = truncate_to_width(label.as_ref(), width, size);
    container(
        text(display)
            .size(size)
            .wrapping(Wrapping::None),
    )
    .width(Length::Fixed(width))
    .clip(true)
    .into()
}

fn truncate_to_width(text: &str, width: f32, font_size: f32) -> String {
    let max_units = (width / (font_size * 0.58)).floor() as usize;
    if max_units == 0 {
        return ELLIPSIS.to_string();
    }

    let total_units: usize = text.chars().map(char_width_units).sum();
    if total_units <= max_units {
        return text.to_string();
    }

    let ellipsis_units = char_width_units(ELLIPSIS);
    let budget = max_units.saturating_sub(ellipsis_units);
    let mut used = 0usize;
    let mut result = String::new();

    for ch in text.chars() {
        let units = char_width_units(ch);
        if used + units > budget {
            break;
        }
        used += units;
        result.push(ch);
    }

    result.push(ELLIPSIS);
    result
}

fn char_width_units(ch: char) -> usize {
    if ch.is_ascii() {
        1
    } else {
        2
    }
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
