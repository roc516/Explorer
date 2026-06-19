use iced::widget::{container, mouse_area, stack, Space};
use iced::{alignment, Element, Fill, Length};

use crate::message::Message;
use crate::widget::settings_dialog;
use crate::widget::wheel_blocker::WheelBlocker;

pub fn overlay(
    dialog: Element<'_, Message>,
    on_dismiss: Message,
) -> Element<'_, Message> {
    stack![
        mouse_area(
            container(WheelBlocker::new(Space::new().width(Fill).height(Fill)))
                .width(Fill)
                .height(Fill)
                .style(settings_dialog::backdrop),
        )
        .interaction(iced::mouse::Interaction::Idle)
        .on_press(on_dismiss),
        container(dialog)
            .width(Fill)
            .height(Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
