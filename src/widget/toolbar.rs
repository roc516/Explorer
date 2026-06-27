use explorer_core::EPath;
use explorer_ui::{ids, LanguageBundle};
use iced::window as iced_window;
use iced::widget::{button, container, mouse_area, row, scrollable, text, text_input};
use iced::{alignment, Element, Fill, Theme};

use crate::fluent::{
    FONT_SIZE_ADDRESS, FONT_SIZE_BREADCRUMB_SEP, HEIGHT_COMMAND_BAR, PAGE_PADDING_H,
    RADIUS_CONTROL, SPACE_MD, SPACE_SM, SPACE_XS,
};
use crate::message::{settings, window as window_msg, Message as AppMessage};
use crate::widget::toolbar_icons::{self, NavIcon};

pub const ADDRESS_INPUT_ID: iced::widget::Id = iced::widget::Id::new("toolbar-address-input");

#[derive(Debug, Clone)]
pub enum Message {
    GoUp,
    GoBack,
    GoForward,
    Refresh,
    AddressEdited(String),
    AddressSubmit,
    AddressEditStart,
    BreadcrumbNavigate(EPath),
}

pub struct ToolbarWidget;

impl ToolbarWidget {
    pub fn new() -> Self {
        Self
    }

    pub fn view(
        &self,
        bundle: LanguageBundle,
        current_path: &EPath,
        address_input: &str,
        address_editing: bool,
        can_go_back: bool,
        can_go_forward: bool,
        can_go_up: bool,
        window_id: iced_window::Id,
    ) -> Element<'_, AppMessage> {
        let address_placeholder = bundle.tr(ids::TOOLBAR_ADDRESS_PLACEHOLDER);

        let nav_buttons = row![
            toolbar_icons::nav_button(
                NavIcon::Back,
                can_go_back,
                can_go_back.then_some(AppMessage::Window(
                    window_id,
                    window_msg::Message::Explorer(Message::GoBack),
                )),
            ),
            toolbar_icons::nav_button(
                NavIcon::Forward,
                can_go_forward,
                can_go_forward.then_some(AppMessage::Window(
                    window_id,
                    window_msg::Message::Explorer(Message::GoForward),
                )),
            ),
            toolbar_icons::nav_button(
                NavIcon::Up,
                can_go_up,
                can_go_up.then_some(AppMessage::Window(
                    window_id,
                    window_msg::Message::Explorer(Message::GoUp),
                )),
            ),
            toolbar_icons::nav_button(
                NavIcon::Refresh,
                true,
                Some(AppMessage::Window(
                    window_id,
                    window_msg::Message::Explorer(Message::Refresh),
                )),
            ),
            toolbar_icons::nav_button(
                NavIcon::Settings,
                true,
                Some(AppMessage::Settings(settings::Message::Toggle)),
            ),
        ]
        .spacing(SPACE_XS);

        let address_bar = container(if address_editing {
            text_input(&address_placeholder, address_input)
                .id(ADDRESS_INPUT_ID)
                .on_input(move |value| {
                    AppMessage::Window(
                        window_id,
                        window_msg::Message::Explorer(Message::AddressEdited(value)),
                    )
                })
                .on_submit(AppMessage::Window(
                    window_id,
                    window_msg::Message::Explorer(Message::AddressSubmit),
                ))
                .size(FONT_SIZE_ADDRESS)
                .width(Fill)
                .into()
        } else {
            breadcrumb_bar(current_path, window_id)
        })
        .padding([0.0, SPACE_MD])
        .width(Fill)
        .height(Fill)
        .align_y(alignment::Vertical::Center);

        container(
            row![nav_buttons, address_bar]
                .spacing(SPACE_MD)
                .align_y(alignment::Vertical::Center)
                .width(Fill)
                .height(Fill),
        )
        .padding([SPACE_XS, PAGE_PADDING_H])
        .width(Fill)
        .height(HEIGHT_COMMAND_BAR)
        .into()
    }
}

impl Default for ToolbarWidget {
    fn default() -> Self {
        Self::new()
    }
}

fn breadcrumb_bar(current_path: &EPath, window_id: iced_window::Id) -> Element<'static, AppMessage> {
    let crumbs = current_path.breadcrumbs();
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
    path: EPath,
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
