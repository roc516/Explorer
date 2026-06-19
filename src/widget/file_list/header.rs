use iced::widget::{button, container, row, rule, text, Space};
use iced::{alignment, Element, Length, Theme};
use lucide_icons::Icon;

use crate::fluent::{
    FONT_SIZE_CAPTION, HEIGHT_LIST_ROW, PAGE_PADDING_H, RADIUS_CONTROL, SPACE_XS,
};
use crate::widget::lucide_icon;

use super::columns::{Column, ColumnWidths, COL_ICON};
use super::message::Message;
use super::resize::column_divider;
use super::sort::{SortDirection, SortState};

const HEADER_SORT_ICON: f32 = 12.0;
const HEADER_BUTTON_HEIGHT: f32 = HEIGHT_LIST_ROW - SPACE_XS * 2.0;

pub(crate) fn view(
    column_name: String,
    column_modified: String,
    column_type: String,
    column_size: String,
    widths: &ColumnWidths,
    sort: SortState,
    resizing: Option<Column>,
) -> Element<'static, Message> {
    container(
        row![
            header_column(
                column_name,
                Column::Name,
                COL_ICON + widths.name,
                sort,
            ),
            column_divider(Column::Name, resizing == Some(Column::Name)),
            header_column(
                column_modified,
                Column::Modified,
                widths.modified,
                sort,
            ),
            column_divider(Column::Modified, resizing == Some(Column::Modified)),
            header_column(
                column_type,
                Column::Type,
                widths.type_,
                sort,
            ),
            column_divider(Column::Type, resizing == Some(Column::Type)),
            header_column(
                column_size,
                Column::Size,
                widths.size,
                sort,
            ),
            column_divider(Column::Size, resizing == Some(Column::Size)),
        ]
        .spacing(0)
        .width(iced::Fill),
    )
    .height(Length::Fixed(HEIGHT_LIST_ROW))
    .align_y(alignment::Vertical::Center)
    .padding([0.0, PAGE_PADDING_H])
    .width(iced::Fill)
    .style(list_header_bar)
    .into()
}

pub(crate) fn list_header_rule(theme: &Theme) -> rule::Style {
    let palette = theme.extended_palette();
    rule::Style {
        color: palette.background.strong.color.scale_alpha(0.45),
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
        snap: false,
    }
}

fn header_column(
    label: String,
    column: Column,
    width: f32,
    sort: SortState,
) -> Element<'static, Message> {
    let sorted = sort.column == column;

    container(
        button(
            container(
                row![
                    header_column_content(label, sorted, sort.direction),
                    Space::new().width(iced::Fill),
                ]
                .width(iced::Fill)
                .align_y(alignment::Vertical::Center),
            )
            .width(iced::Fill)
            .height(Length::Fixed(HEADER_BUTTON_HEIGHT))
            .align_y(alignment::Vertical::Center),
        )
        .on_press(Message::ColumnSortClicked(column))
        .width(Length::Fixed(width))
        .height(Length::Fixed(HEADER_BUTTON_HEIGHT))
        .padding(0)
        .style(header_button_style(sorted)),
    )
    .width(Length::Fixed(width))
    .height(Length::Fixed(HEIGHT_LIST_ROW))
    .align_y(alignment::Vertical::Center)
    .into()
}

fn header_column_content(
    label: String,
    sorted: bool,
    direction: SortDirection,
) -> Element<'static, Message> {
    let label_text = text(label)
        .size(FONT_SIZE_CAPTION)
        .style(if sorted {
            header_text_sorted
        } else {
            header_text
        });

    if sorted {
        row![
            label_text,
            sort_indicator(direction),
        ]
        .spacing(SPACE_XS)
        .align_y(alignment::Vertical::Center)
        .into()
    } else {
        row![label_text]
            .align_y(alignment::Vertical::Center)
            .into()
    }
}

fn sort_indicator(direction: SortDirection) -> Element<'static, Message> {
    let icon = match direction {
        SortDirection::Ascending => Icon::ArrowUp,
        SortDirection::Descending => Icon::ArrowDown,
    };
    lucide_icon::icon_muted::<Message>(icon, HEADER_SORT_ICON, 0.72).into()
}

fn list_header_bar(theme: &Theme) -> iced::widget::container::Style {
    let palette = theme.extended_palette();
    iced::widget::container::Style {
        background: Some(iced::Background::Color(
            palette.background.weak.color.scale_alpha(0.35),
        )),
        ..Default::default()
    }
}

fn header_text(theme: &Theme) -> iced::widget::text::Style {
    let palette = theme.extended_palette();
    iced::widget::text::Style {
        color: Some(palette.background.base.text.scale_alpha(0.62)),
    }
}

fn header_text_sorted(theme: &Theme) -> iced::widget::text::Style {
    let palette = theme.extended_palette();
    iced::widget::text::Style {
        color: Some(palette.background.base.text),
    }
}

fn header_button_style(
    sorted: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style + Copy {
    move |theme, status| {
        let palette = theme.extended_palette();
        let radius = RADIUS_CONTROL.into();

        match status {
            button::Status::Hovered => button::Style {
                background: Some(iced::Background::Color(
                    palette.background.strong.color.scale_alpha(0.38),
                )),
                text_color: palette.background.base.text,
                border: iced::Border {
                    radius,
                    ..Default::default()
                },
                ..button::Style::default()
            },
            button::Status::Pressed => button::Style {
                background: Some(iced::Background::Color(
                    palette.primary.weak.color.scale_alpha(0.75),
                )),
                text_color: palette.background.base.text,
                border: iced::Border {
                    radius,
                    ..Default::default()
                },
                ..button::Style::default()
            },
            _ if sorted => button::Style {
                background: Some(iced::Background::Color(
                    palette.background.strong.color.scale_alpha(0.22),
                )),
                text_color: palette.background.base.text,
                border: iced::Border {
                    radius,
                    ..Default::default()
                },
                ..button::Style::default()
            },
            _ => button::Style {
                background: None,
                text_color: palette.background.base.text.scale_alpha(0.62),
                border: iced::Border {
                    radius,
                    ..Default::default()
                },
                ..button::Style::default()
            },
        }
    }
}
