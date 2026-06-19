use explorer_core::{ids, LanguageBundle};
use iced::widget::{container, row, text_input};
use iced::{alignment, Element, Fill};

use crate::fluent::{HEIGHT_COMMAND_BAR, PAGE_PADDING_H, SPACE_MD, SPACE_XS};
use crate::message::{explorer, settings, Message};
use crate::widget::toolbar_icons::{self, NavIcon};

pub struct ToolbarWidget;

impl ToolbarWidget {
    pub fn new() -> Self {
        Self
    }

    pub fn view(
        &self,
        bundle: LanguageBundle,
        address_input: &str,
        can_go_back: bool,
        can_go_forward: bool,
        can_go_up: bool,
    ) -> Element<'_, Message> {
        let address_placeholder = bundle.tr(ids::TOOLBAR_ADDRESS_PLACEHOLDER);

        let nav_buttons = row![
            toolbar_icons::nav_button(
                NavIcon::Back,
                can_go_back,
                can_go_back.then_some(Message::Explorer(explorer::Message::GoBack)),
            ),
            toolbar_icons::nav_button(
                NavIcon::Forward,
                can_go_forward,
                can_go_forward.then_some(Message::Explorer(explorer::Message::GoForward)),
            ),
            toolbar_icons::nav_button(
                NavIcon::Up,
                can_go_up,
                can_go_up.then_some(Message::Explorer(explorer::Message::GoUp)),
            ),
            toolbar_icons::nav_button(
                NavIcon::Refresh,
                true,
                Some(Message::Explorer(explorer::Message::Refresh)),
            ),
            toolbar_icons::nav_button(
                NavIcon::Settings,
                true,
                Some(Message::Settings(settings::Message::Toggle)),
            ),
        ]
        .spacing(SPACE_XS);

        let address_bar = container(
            text_input(&address_placeholder, address_input)
                .on_input(|value| Message::Explorer(explorer::Message::AddressEdited(value)))
                .on_submit(Message::Explorer(explorer::Message::AddressSubmit))
                .width(Fill),
        )
        .padding([SPACE_XS, SPACE_MD])
        .width(Fill);

        container(
            row![nav_buttons, address_bar]
                .spacing(SPACE_MD)
                .align_y(alignment::Vertical::Center)
                .width(Fill),
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
