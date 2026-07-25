use explorer_app::FileEntry;
use iced::widget::{container, mouse_area, row, Space};
use iced::{alignment, Element, Fill, Length, Theme};
use lucide_icons::Icon;

use crate::fluent::{RADIUS_CONTROL, SPACE_XS};
use crate::widget::LucideIcon;

use super::cell::{clipped_cell, column_gap};
use super::columns::{Column, ColumnOrder, ColumnWidths, COL_ICON};
use super::message::Message;

pub(crate) fn file_row<'a>(
    index: usize,
    entry: &'a FileEntry,
    selected: bool,
    bundle: &explorer_app::LanguageBundle,
    order: ColumnOrder,
    widths: &ColumnWidths,
) -> Element<'a, Message> {
    let modified = entry.modified_label(bundle);
    let type_label = entry.type_label(bundle);
    let size = entry.size_label(bundle);

    let mut cells: Vec<Element<'a, Message>> = Vec::new();
    for (i, &column) in order.as_slice().iter().enumerate() {
        if i > 0 {
            cells.push(column_gap());
        }
        cells.push(match column {
            Column::Name => name_cell(entry, widths.name),
            Column::Modified => clipped_cell(modified.clone(), widths.modified, 13.0),
            Column::Type => clipped_cell(type_label.clone(), widths.type_, 13.0),
            Column::Size => clipped_cell(size.clone(), widths.size, 13.0),
        });
    }
    cells.push(Space::new().width(Fill).into());

    let content = row(cells)
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

fn name_cell<'a>(entry: &'a FileEntry, name_width: f32) -> Element<'a, Message> {
    row![
        container(LucideIcon::new(if entry.is_dir() {
            Icon::Folder
        } else {
            Icon::File
        }))
        .width(Length::Fixed(COL_ICON))
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center),
        clipped_cell(entry.name(), name_width, 14.0),
    ]
    .spacing(0)
    .align_y(alignment::Vertical::Center)
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
