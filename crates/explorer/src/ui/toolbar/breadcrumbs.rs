use explorer_core::Mounter;
use explorer_app::{breadcrumbs, mount_root_label, ExplorerModel};
use iced::window as iced_window;
use iced::widget::{button, container, mouse_area, row, scrollable, text};
use iced::{alignment, Element, Fill, Theme};

use crate::fluent::{FONT_SIZE_ADDRESS, FONT_SIZE_BREADCRUMB_SEP, RADIUS_CONTROL, SPACE_SM};
use crate::message::{window as window_msg, Message as AppMessage};

use super::Message;

pub fn breadcrumb_bar(
    model: &ExplorerModel,
    window_id: iced_window::Id,
) -> Element<'static, AppMessage> {
    let root_label = Mounter::mount_ref(model.location())
        .ok()
        .map(|(container, _)| mount_root_label(container));
    let crumbs = breadcrumbs(&model.current_path, root_label);
    let last_index = crumbs.len().saturating_sub(1);
    let mut items: Vec<Element<'static, AppMessage>> = Vec::new();

    for (index, crumb) in crumbs.into_iter().enumerate() {
        if index > 0 {
            items.push(
                text("›")
                    .size(FONT_SIZE_BREADCRUMB_SEP)
                    .style(breadcrumb_separator)
                    .into(),
            );
        }

        items.push(breadcrumb_button(
            crumb.label,
            crumb.path,
            index == last_index,
            window_id,
        ));
    }

    let trail = row(items)
        .spacing(SPACE_SM)
        .align_y(alignment::Vertical::Center);

    mouse_area(
        container(
            scrollable(trail)
                .width(Fill)
                .direction(scrollable::Direction::Horizontal(
                    scrollable::Scrollbar::default(),
                )),
        )
        .width(Fill)
        .align_y(alignment::Vertical::Center),
    )
    .on_double_click(AppMessage::Window(
        window_id,
        window_msg::Message::Explorer(Message::AddressEditStart),
    ))
    .into()
}

fn breadcrumb_button(
    label: String,
    path: std::path::PathBuf,
    is_last: bool,
    window_id: iced_window::Id,
) -> Element<'static, AppMessage> {
    button(text(label).size(FONT_SIZE_ADDRESS).style(if is_last {
        breadcrumb_current_text
    } else {
        breadcrumb_link_text
    }))
    .on_press(AppMessage::Window(
        window_id,
        window_msg::Message::Explorer(Message::BreadcrumbNavigate(path)),
    ))
    .padding([2.0, SPACE_SM])
    .style(breadcrumb_button_style)
    .into()
}

fn breadcrumb_link_text(theme: &Theme) -> iced::widget::text::Style {
    let palette = theme.extended_palette();
    iced::widget::text::Style {
        color: Some(palette.primary.strong.color),
    }
}

fn breadcrumb_current_text(theme: &Theme) -> iced::widget::text::Style {
    let palette = theme.extended_palette();
    iced::widget::text::Style {
        color: Some(palette.background.base.text),
    }
}

fn breadcrumb_separator(theme: &Theme) -> iced::widget::text::Style {
    let palette = theme.extended_palette();
    iced::widget::text::Style {
        color: Some(palette.background.base.text.scale_alpha(0.45)),
    }
}

fn breadcrumb_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let radius = RADIUS_CONTROL.into();

    match status {
        button::Status::Hovered => button::Style {
            background: Some(iced::Background::Color(
                palette.background.strong.color.scale_alpha(0.4),
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
                palette.primary.weak.color.scale_alpha(0.85),
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
            text_color: palette.background.base.text,
            border: iced::Border {
                radius,
                ..Default::default()
            },
            ..button::Style::default()
        },
    }
}
