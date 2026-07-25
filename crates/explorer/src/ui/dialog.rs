use iced::widget::{button, column, container, mouse_area, opaque, row, rule, text, Space};
use iced::{alignment, Element, Fill, Length};

use crate::fluent::{SPACE_LG, SPACE_MD, SPACE_SM};
use crate::ui::style::{dialog_backdrop, dialog_container, dialog_divider, icon_button};
use crate::widget::lucide_icon::LucideIcon;
use crate::widget::wheel_blocker::WheelBlocker;
use lucide_icons::Icon;

const CLOSE_BUTTON_SIZE: f32 = 32.0;
const CLOSE_ICON_SIZE: f32 = 16.0;
const HEADER_HEIGHT: f32 = 48.0;

/// Reusable modal dialog shell: backdrop, title bar, close button, and body/footer slots.
pub struct Dialog<'a, Message> {
    title: String,
    on_close: Message,
    width: f32,
    header_action: Option<Element<'a, Message>>,
    body: Option<Element<'a, Message>>,
    footer: Option<Element<'a, Message>>,
}

impl<'a, Message: Clone + 'a> Dialog<'a, Message> {
    pub fn new(title: impl Into<String>, on_close: Message) -> Self {
        Self {
            title: title.into(),
            on_close,
            width: 480.0,
            header_action: None,
            body: None,
            footer: None,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Optional control placed before the close button (e.g. "Open externally").
    pub fn header_action(mut self, action: impl Into<Element<'a, Message>>) -> Self {
        self.header_action = Some(action.into());
        self
    }

    pub fn body(mut self, body: impl Into<Element<'a, Message>>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn footer(mut self, footer: impl Into<Element<'a, Message>>) -> Self {
        self.footer = Some(footer.into());
        self
    }

    pub fn view(self) -> Element<'a, Message> {
        let mut header_items: Vec<Element<'a, Message>> = vec![
            text(self.title).size(14).into(),
            Space::new().width(Fill).into(),
        ];

        if let Some(action) = self.header_action {
            header_items.push(action);
        }

        header_items.push(close_button(self.on_close.clone()));

        let header = row(header_items)
            .spacing(SPACE_SM)
            .align_y(alignment::Vertical::Center)
            .padding([SPACE_MD, SPACE_LG])
            .height(Length::Fixed(HEADER_HEIGHT))
            .width(Fill);

        let mut sections: Vec<Element<'a, Message>> = vec![
            header.into(),
            rule::horizontal(1).style(dialog_divider).into(),
        ];

        if let Some(body) = self.body {
            sections.push(body);
        }

        if let Some(footer) = self.footer {
            sections.push(rule::horizontal(1).style(dialog_divider).into());
            sections.push(footer);
        }

        let panel = container(column(sections).width(Fill))
            .width(self.width)
            .style(dialog_container);

        show_overlay(panel.into(), self.on_close)
    }
}

fn close_button<'a, Message: Clone + 'a>(on_close: Message) -> Element<'a, Message> {
    button(
        container(LucideIcon::new(Icon::X).size(CLOSE_ICON_SIZE).muted(0.72))
            .width(Length::Fixed(CLOSE_BUTTON_SIZE))
            .height(Length::Fixed(CLOSE_BUTTON_SIZE))
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center),
    )
    .on_press(on_close)
    .width(Length::Fixed(CLOSE_BUTTON_SIZE))
    .height(Length::Fixed(CLOSE_BUTTON_SIZE))
    .padding(0)
    .style(icon_button)
    .into()
}

fn show_overlay<'a, Message: Clone + 'a>(
    content: Element<'a, Message>,
    on_dismiss: Message,
) -> Element<'a, Message> {
    let panel = opaque(WheelBlocker::new(content));

    opaque(
        mouse_area(
            container(panel)
                .width(Fill)
                .height(Fill)
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center)
                .style(dialog_backdrop),
        )
        .interaction(iced::mouse::Interaction::Idle)
        .on_press(on_dismiss),
    )
    .into()
}
