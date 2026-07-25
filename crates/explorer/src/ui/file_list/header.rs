use iced::mouse::Interaction;
use iced::widget::{button, container, mouse_area, row, rule, text, Space};
use iced::{alignment, Element, Length, Theme};
use lucide_icons::Icon;

use crate::fluent::{
    FONT_SIZE_CAPTION, HEIGHT_LIST_ROW, PAGE_PADDING_H, RADIUS_CONTROL, SPACE_XS,
};
use crate::widget::LucideIcon;

use super::columns::{ActiveColumnReorder, Column, ColumnOrder, ColumnWidths};
use super::message::Message;
use super::resize::{column_divider, insert_indicator};
use super::sort::{SortDirection, SortState};

const HEADER_SORT_ICON: f32 = 12.0;
const HEADER_GRIP_WIDTH: f32 = 16.0;
const HEADER_BUTTON_HEIGHT: f32 = HEIGHT_LIST_ROW - SPACE_XS * 2.0;
const HANDLE_DOT: f32 = 2.5;
const HANDLE_DOT_GAP: f32 = 2.0;

pub(crate) struct ColumnLabels {
    pub name: String,
    pub modified: String,
    pub type_: String,
    pub size: String,
}

impl ColumnLabels {
    fn label(&self, column: Column) -> String {
        match column {
            Column::Name => self.name.clone(),
            Column::Modified => self.modified.clone(),
            Column::Type => self.type_.clone(),
            Column::Size => self.size.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HeaderVisual {
    Idle,
    HandleHovered,
    Dragging,
}

pub(crate) fn view(
    labels: &ColumnLabels,
    order: ColumnOrder,
    widths: &ColumnWidths,
    sort: SortState,
    resizing: Option<Column>,
    reorder: Option<ActiveColumnReorder>,
    handle_hovered: Option<Column>,
) -> Element<'static, Message> {
    let gap_before = reorder.and_then(|active| {
        if !active.dragging {
            return None;
        }
        let mut preview = order;
        preview.move_to(active.origin_index, active.insert_at);
        if preview == order {
            return None;
        }
        Some(if active.insert_at <= active.origin_index {
            active.insert_at
        } else {
            active.insert_at + 1
        })
    });

    let mut children: Vec<Element<'static, Message>> = Vec::new();
    for (index, &column) in order.as_slice().iter().enumerate() {
        if gap_before == Some(index) {
            children.push(insert_indicator(true));
        }

        let visual = header_visual(column, reorder, handle_hovered);
        children.push(header_column(
            labels.label(column),
            column,
            widths.display_width(column),
            sort,
            visual,
        ));
        children.push(column_divider(column, resizing == Some(column)));
    }

    if gap_before == Some(order.as_slice().len()) {
        children.push(insert_indicator(true));
    }

    container(row(children).spacing(0).width(iced::Fill))
        .height(Length::Fixed(HEIGHT_LIST_ROW))
        .align_y(alignment::Vertical::Center)
        .padding([0.0, PAGE_PADDING_H])
        .width(iced::Fill)
        .style(list_header_bar)
        .into()
}

fn header_visual(
    column: Column,
    reorder: Option<ActiveColumnReorder>,
    handle_hovered: Option<Column>,
) -> HeaderVisual {
    if reorder.is_some_and(|active| active.column == column) {
        return HeaderVisual::Dragging;
    }
    if handle_hovered == Some(column) {
        return HeaderVisual::HandleHovered;
    }
    HeaderVisual::Idle
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
    visual: HeaderVisual,
) -> Element<'static, Message> {
    let sorted = sort.column == column;
    let sort_width = (width - HEADER_GRIP_WIDTH).max(24.0);

    container(
        row![
            drag_handle(column, visual),
            sort_button(
                label,
                column,
                sort_width,
                sorted,
                sort.direction,
                visual,
            ),
        ]
        .spacing(0)
        .align_y(alignment::Vertical::Center)
        .width(Length::Fixed(width)),
    )
    .width(Length::Fixed(width))
    .height(Length::Fixed(HEIGHT_LIST_ROW))
    .align_y(alignment::Vertical::Center)
    .style(move |theme| column_shell_style(theme, visual))
    .into()
}

fn drag_handle(column: Column, visual: HeaderVisual) -> Element<'static, Message> {
    mouse_area(
        container(handle_dots(visual))
            .width(Length::Fixed(HEADER_GRIP_WIDTH))
            .height(Length::Fixed(HEADER_BUTTON_HEIGHT))
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
            .style(move |theme| handle_style(theme, visual)),
    )
    .on_press(Message::ColumnReorderStarted(column))
    .on_enter(Message::ColumnHandleHovered(column))
    .on_exit(Message::ColumnHandleUnhovered(column))
    .interaction(Interaction::Grab)
    .into()
}

fn handle_dots(visual: HeaderVisual) -> Element<'static, Message> {
    let column_of_dots = || {
        iced::widget::column![
            handle_dot(visual),
            handle_dot(visual),
            handle_dot(visual),
        ]
        .spacing(HANDLE_DOT_GAP)
        .align_x(alignment::Horizontal::Center)
    };

    row![column_of_dots(), column_of_dots()]
        .spacing(HANDLE_DOT_GAP)
        .align_y(alignment::Vertical::Center)
        .into()
}

fn handle_dot(visual: HeaderVisual) -> Element<'static, Message> {
    container(Space::new())
        .width(Length::Fixed(HANDLE_DOT))
        .height(Length::Fixed(HANDLE_DOT))
        .style(move |theme| handle_dot_style(theme, visual))
        .into()
}

fn sort_button(
    label: String,
    column: Column,
    width: f32,
    sorted: bool,
    direction: SortDirection,
    visual: HeaderVisual,
) -> Element<'static, Message> {
    button(
        container(
            row![
                header_column_content(label, sorted, direction),
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
    .style(header_sort_button_style(sorted, visual))
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
        row![label_text, sort_indicator(direction),]
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
    LucideIcon::new(icon)
        .size(HEADER_SORT_ICON)
        .muted(0.72)
        .into()
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

fn column_shell_style(
    theme: &Theme,
    visual: HeaderVisual,
) -> iced::widget::container::Style {
    let palette = theme.extended_palette();
    let background = match visual {
        HeaderVisual::Dragging => Some(iced::Background::Color(
            palette.primary.weak.color.scale_alpha(0.45),
        )),
        _ => None,
    };

    iced::widget::container::Style {
        background,
        border: iced::Border {
            radius: RADIUS_CONTROL.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn handle_style(theme: &Theme, visual: HeaderVisual) -> iced::widget::container::Style {
    let palette = theme.extended_palette();
    let radius = RADIUS_CONTROL.into();

    let background = match visual {
        HeaderVisual::HandleHovered => Some(iced::Background::Color(
            palette.background.strong.color.scale_alpha(0.4),
        )),
        HeaderVisual::Dragging => Some(iced::Background::Color(
            palette.primary.weak.color.scale_alpha(0.65),
        )),
        HeaderVisual::Idle => None,
    };

    iced::widget::container::Style {
        background,
        border: iced::Border {
            radius,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn handle_dot_style(theme: &Theme, visual: HeaderVisual) -> iced::widget::container::Style {
    let palette = theme.extended_palette();
    let alpha = match visual {
        HeaderVisual::Dragging => 0.9,
        HeaderVisual::HandleHovered => 0.72,
        HeaderVisual::Idle => 0.38,
    };

    iced::widget::container::Style {
        background: Some(iced::Background::Color(
            palette.background.base.text.scale_alpha(alpha),
        )),
        border: iced::Border {
            radius: 10.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn header_sort_button_style(
    sorted: bool,
    visual: HeaderVisual,
) -> impl Fn(&Theme, button::Status) -> button::Style + Copy {
    move |theme, status| {
        let palette = theme.extended_palette();
        let radius = RADIUS_CONTROL.into();

        if visual == HeaderVisual::Dragging {
            return button::Style {
                background: None,
                text_color: palette.background.base.text,
                border: iced::Border {
                    radius,
                    ..Default::default()
                },
                ..button::Style::default()
            };
        }

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
