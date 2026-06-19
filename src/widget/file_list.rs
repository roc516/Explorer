use std::path::PathBuf;

use explorer_core::{ids, ExplorerModel, FileEntry};
use iced::event::{self, Status};
use iced::mouse::{self, Interaction};
use iced::widget::{column, container, mouse_area, row, rule, scrollable, stack, text, Space};
use iced::widget::text::Wrapping;
use iced::{alignment, Element, Event, Fill, Length, Subscription, Task, Theme};
use lucide_icons::Icon;

use crate::fluent::{PAGE_PADDING_H, RADIUS_CONTROL, SPACE_LG, SPACE_SM, SPACE_XS};
use crate::widget::lucide_icon;

const COL_ICON: f32 = 24.0;
const DEFAULT_COL_NAME: f32 = 248.0;
const DEFAULT_COL_MODIFIED: f32 = 160.0;
const DEFAULT_COL_TYPE: f32 = 120.0;
const DEFAULT_COL_SIZE: f32 = 90.0;
const MIN_COL_WIDTH: f32 = 48.0;
const DIVIDER_LINE_HEIGHT: f32 = 16.0;
const ELLIPSIS: char = '…';

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Column {
    Name,
    Modified,
    Type,
    Size,
}

#[derive(Debug, Clone)]
struct ColumnWidths {
    name: f32,
    modified: f32,
    type_: f32,
    size: f32,
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
    fn get(&self, column: Column) -> f32 {
        match column {
            Column::Name => self.name,
            Column::Modified => self.modified,
            Column::Type => self.type_,
            Column::Size => self.size,
        }
    }

    fn set(&mut self, column: Column, width: f32) {
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
struct ActiveColumnResize {
    column: Column,
    last_x: Option<f32>,
}

#[derive(Debug, Clone)]
pub enum Message {
    EntryClicked(usize),
    EntryDoubleClicked(usize),
    DirectoryLoaded(Result<(PathBuf, Vec<FileEntry>), String>),
    ColumnResizeStarted(Column),
    ColumnResizeMoved(f32),
    ColumnResizeEnded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    DirectoryChanged(std::path::PathBuf),
}

pub struct FileListWidget {
    column_widths: ColumnWidths,
    column_resize: Option<ActiveColumnResize>,
}

impl FileListWidget {
    pub fn new() -> Self {
        Self {
            column_widths: ColumnWidths::default(),
            column_resize: None,
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        if self.column_resize.is_none() {
            return Subscription::none();
        }

        event::listen_with(column_resize_listener)
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
        let widths = &self.column_widths;
        let resizing = self.column_resize.map(|active| active.column);

        let header = container(
            row![
                Space::new().width(Length::Fixed(COL_ICON)),
                clipped_cell(column_name, widths.name, 12.0),
                column_divider(Column::Name, resizing == Some(Column::Name)),
                clipped_cell(column_modified, widths.modified, 12.0),
                column_divider(Column::Modified, resizing == Some(Column::Modified)),
                clipped_cell(column_type, widths.type_, 12.0),
                column_divider(Column::Type, resizing == Some(Column::Type)),
                header_last_cell(column_size, widths.size, resizing == Some(Column::Size)),
            ]
            .spacing(0)
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
                        file_row(
                            index,
                            entry,
                            model.selected_index == Some(index),
                            &bundle,
                            widths,
                        )
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

fn column_resize_listener(
    event: Event,
    _status: Status,
    _window: iced::window::Id,
) -> Option<Message> {
    match event {
        Event::Mouse(mouse::Event::CursorMoved { position }) => {
            Some(Message::ColumnResizeMoved(position.x))
        }
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
            Some(Message::ColumnResizeEnded)
        }
        _ => None,
    }
}

fn file_row<'a>(
    index: usize,
    entry: &'a FileEntry,
    selected: bool,
    bundle: &explorer_core::LanguageBundle,
    widths: &ColumnWidths,
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
        clipped_cell(&entry.name, widths.name, 14.0),
        column_gap(),
        clipped_cell(modified, widths.modified, 13.0),
        column_gap(),
        clipped_cell(type_label, widths.type_, 13.0),
        column_gap(),
        clipped_cell(size, widths.size, 13.0),
        Space::new().width(Fill),
    ]
    .spacing(0)
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

fn header_last_cell(label: String, width: f32, active: bool) -> Element<'static, Message> {
    stack![
        clipped_cell(label, width, 12.0),
        container(column_divider(Column::Size, active))
            .width(Fill)
            .height(Fill)
            .align_x(alignment::Horizontal::Right)
            .align_y(alignment::Vertical::Center),
    ]
    .width(Length::Fixed(width))
    .into()
}

fn column_gap() -> Element<'static, Message> {
    Space::new()
        .width(Length::Fixed(SPACE_SM))
        .into()
}

fn column_divider(column: Column, active: bool) -> Element<'static, Message> {
    mouse_area(
        container(divider_line(active))
            .width(Length::Fixed(SPACE_SM))
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center),
    )
    .on_press(Message::ColumnResizeStarted(column))
    .interaction(Interaction::ResizingColumn)
    .into()
}

fn divider_line(active: bool) -> Element<'static, Message> {
    container(Space::new())
        .width(Length::Fixed(1.0))
        .height(Length::Fixed(DIVIDER_LINE_HEIGHT))
        .style(move |theme| divider_line_style(theme, active))
        .into()
}

fn divider_line_style(theme: &Theme, active: bool) -> iced::widget::container::Style {
    let palette = theme.extended_palette();
    let color = if active {
        palette.primary.strong.color
    } else {
        palette.background.strong.color
    };

    iced::widget::container::Style {
        background: Some(iced::Background::Color(color)),
        ..Default::default()
    }
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
