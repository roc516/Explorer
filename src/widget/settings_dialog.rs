use std::fmt;

use explorer_core::{ids, Language, LanguageBundle};
use iced::widget::{button, column, container, mouse_area, pick_list, row, rule, text, Space};
use iced::{alignment, Element, Fill, Length, Theme};
use lucide_icons::Icon;

use crate::fluent::{RADIUS_CONTROL, RADIUS_FLYOUT, SPACE_LG, SPACE_MD, SPACE_SM, SPACE_XS};
use crate::message::{settings, theme, Message as AppMessage};
use crate::theme::{theme_options, AppTheme};
use crate::widget::lucide_icon;

pub mod locale {
    use explorer_core::Language;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Message {
        Selected(Language),
    }
}

const DIALOG_WIDTH: f32 = 400.0;
const THEME_MENU_HEIGHT: f32 = 280.0;
const CLOSE_BUTTON_SIZE: f32 = 32.0;
const CLOSE_ICON_SIZE: f32 = 16.0;

pub struct SettingsDialogWidget;

impl SettingsDialogWidget {
    pub fn new() -> Self {
        Self
    }

    pub fn view(
        &self,
        bundle: LanguageBundle,
        theme_choice: AppTheme,
        language: Language,
    ) -> Element<'_, AppMessage> {
        let title = bundle.tr(ids::SETTINGS_TITLE);
        let theme_label = bundle.tr(ids::TOOLBAR_THEME);
        let language_label = bundle.tr(ids::TOOLBAR_LANGUAGE);

        let themes = theme_options(bundle);
        let selected_theme = themes
            .iter()
            .find(|option| option.theme == theme_choice)
            .cloned();

        let theme_picker = pick_list(
            themes,
            selected_theme,
            |option| AppMessage::Theme(theme::Message::Selected(option.theme)),
        )
        .width(Fill)
        .menu_height(Length::Fixed(THEME_MENU_HEIGHT));

        let languages: Vec<LanguageOption> = Language::ALL
            .iter()
            .copied()
            .map(|lang| LanguageOption {
                language: lang,
                label: bundle.tr(lang.message_id()),
            })
            .collect();
        let selected_language = languages
            .iter()
            .find(|option| option.language == language)
            .cloned();

        let language_picker = pick_list(
            languages,
            selected_language,
            |option| AppMessage::Locale(locale::Message::Selected(option.language)),
        )
        .width(Fill);

        let body = column![
            section_panel(theme_label, Icon::Palette, theme_picker.into()),
            section_panel(language_label, Icon::Languages, language_picker.into()),
        ]
        .spacing(SPACE_MD)
        .width(Fill);

        let dialog = mouse_area(
            container(
                column![
                    header(title),
                    rule::horizontal(1),
                    body.padding([SPACE_MD, SPACE_LG]),
                ]
                .width(Fill),
            )
            .width(DIALOG_WIDTH)
            .style(dialog_container),
        )
        .on_press(AppMessage::Settings(settings::Message::PressInside));

        column![
            Space::new().height(Fill),
            row![
                Space::new().width(Fill),
                dialog,
                Space::new().width(Fill),
            ],
            Space::new().height(Fill),
        ]
        .width(Fill)
        .height(Fill)
        .into()
    }
}

impl Default for SettingsDialogWidget {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Eq)]
struct LanguageOption {
    language: Language,
    label: String,
}

impl PartialEq for LanguageOption {
    fn eq(&self, other: &Self) -> bool {
        self.language == other.language
    }
}

impl fmt::Display for LanguageOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.label)
    }
}

fn header(title: String) -> Element<'static, AppMessage> {
    row![
        lucide_icon::icon_muted::<AppMessage>(Icon::Settings, 16.0, 0.72),
        text(title).size(15),
        Space::new().width(Fill),
        button(
            container(lucide_icon::icon_muted::<AppMessage>(Icon::X, CLOSE_ICON_SIZE, 0.72))
                .width(Length::Fixed(CLOSE_BUTTON_SIZE))
                .height(Length::Fixed(CLOSE_BUTTON_SIZE))
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center),
        )
            .on_press(AppMessage::Settings(settings::Message::Close))
            .width(Length::Fixed(CLOSE_BUTTON_SIZE))
            .height(Length::Fixed(CLOSE_BUTTON_SIZE))
            .padding(0)
            .style(icon_button),
    ]
    .spacing(SPACE_SM)
    .align_y(alignment::Vertical::Center)
    .padding([SPACE_MD, SPACE_LG])
    .width(Fill)
    .into()
}

fn section_panel(
    title: String,
    icon: Icon,
    control: Element<'_, AppMessage>,
) -> Element<'_, AppMessage> {
    container(
        column![
            row![
                lucide_icon::icon_muted::<AppMessage>(icon, 14.0, 0.6),
                text(title).size(12).style(section_title),
            ]
            .spacing(SPACE_XS)
            .align_y(alignment::Vertical::Center),
            control,
        ]
        .spacing(SPACE_SM)
        .width(Fill),
    )
    .padding(SPACE_SM)
    .width(Fill)
    .style(section_container)
    .into()
}

fn section_title(theme: &Theme) -> iced::widget::text::Style {
    let palette = theme.extended_palette();
    iced::widget::text::Style {
        color: Some(palette.background.base.text.scale_alpha(0.65)),
    }
}

fn dialog_container(theme: &Theme) -> iced::widget::container::Style {
    let palette = theme.extended_palette();
    iced::widget::container::Style {
        background: Some(iced::Background::Color(palette.background.base.color)),
        border: iced::Border {
            width: 1.0,
            color: palette.background.strong.color,
            radius: RADIUS_FLYOUT.into(),
            ..Default::default()
        },
        shadow: iced::Shadow {
            color: iced::Color::BLACK.scale_alpha(0.22),
            offset: iced::Vector::new(0.0, 12.0),
            blur_radius: 32.0,
        },
        ..Default::default()
    }
}

fn section_container(theme: &Theme) -> iced::widget::container::Style {
    let palette = theme.extended_palette();
    iced::widget::container::Style {
        background: Some(iced::Background::Color(palette.background.weak.color)),
        border: iced::Border {
            width: 1.0,
            color: palette.background.strong.color.scale_alpha(0.45),
            radius: RADIUS_CONTROL.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn backdrop(_theme: &Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(iced::Background::Color(iced::Color::BLACK.scale_alpha(0.4))),
        ..Default::default()
    }
}

fn icon_button(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    match status {
        button::Status::Hovered => button::Style {
            background: Some(iced::Background::Color(
                palette.background.strong.color.scale_alpha(0.35),
            )),
            text_color: palette.background.base.text,
            border: iced::Border {
                radius: RADIUS_CONTROL.into(),
                ..Default::default()
            },
            ..button::Style::default()
        },
        _ => button::Style {
            background: None,
            text_color: palette.background.base.text,
            ..button::Style::default()
        },
    }
}
