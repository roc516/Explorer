use explorer_core::FileEntry;
use explorer_ui::FileEntryExt;
use iced::widget::{container, mouse_area, row, Space};
use iced::{alignment, Element, Fill, Length, Theme};
use lucide_icons::Icon;

use crate::fluent::{RADIUS_CONTROL, SPACE_XS};
use crate::widget::lucide_icon;

use super::cell::{clipped_cell, column_gap};
use super::columns::{ColumnWidths, COL_ICON};
use super::message::Message;

pub(crate) fn file_row<'a>(
    index: usize,
    entry: &'a FileEntry,
    selected: bool,
    bundle: &explorer_ui::LanguageBundle,
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
