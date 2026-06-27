use std::fmt;

use explorer_ui::{ids, Language, LanguageBundle};
use iced::widget::{button, column, container, mouse_area, pick_list, row, rule, text, Space};
use iced::{alignment, Element, Fill, Length, Theme};
use lucide_icons::Icon;

use crate::fluent::{
    DIALOG_WIDTH_SETTINGS, HEIGHT_SETTING_ROW, SPACE_LG, SPACE_MD, SPACE_SM, WIDTH_SETTING_COMBO,
};
use crate::message::{settings, theme, Message as AppMessage};
use crate::theme::{theme_options, AppTheme};
use crate::widget::lucide_icon;
use crate::widget::style::{dialog_container, dialog_divider, icon_button, pick_list_style};

pub mod locale {
        use explorer_ui::Language;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Message {
        Selected(Language),
    }
}

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
        .menu_height(Length::Fixed(THEME_MENU_HEIGHT))
        .style(pick_list_style);

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
        .width(Fill)
        .style(pick_list_style);

        let body = column![
            setting_row(theme_label, Icon::Palette, theme_picker.into()),
            rule::horizontal(1).style(group_divider),
            setting_row(language_label, Icon::Languages, language_picker.into()),
        ]
        .width(Fill);

        let dialog = mouse_area(
            container(
                column![
                    header(title),
                    rule::horizontal(1).style(dialog_divider),
                    container(body).padding([SPACE_MD, SPACE_LG]),
                ]
                .width(Fill),
            )
            .width(DIALOG_WIDTH_SETTINGS)
            .style(dialog_container),
        )
        .on_press(AppMessage::Settings(settings::Message::PressInside));

        dialog.into()
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
        text(title).size(14),
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
    .height(Length::Fixed(HEIGHT_SETTING_ROW))
    .width(Fill)
    .into()
}

fn setting_row(
    label: String,
    icon: Icon,
    control: Element<'_, AppMessage>,
) -> Element<'_, AppMessage> {
    row![
        row![
            lucide_icon::icon_muted::<AppMessage>(icon, 14.0, 0.72),
            text(label).size(13),
        ]
        .spacing(SPACE_SM)
        .align_y(alignment::Vertical::Center),
        Space::new().width(Fill),
        container(control)
            .width(Length::Fixed(WIDTH_SETTING_COMBO))
            .align_y(alignment::Vertical::Center),
    ]
    .align_y(alignment::Vertical::Center)
    .height(Length::Fixed(HEIGHT_SETTING_ROW))
    .width(Fill)
    .into()
}

fn group_divider(theme: &Theme) -> rule::Style {
    let palette = theme.extended_palette();
    rule::Style {
        color: palette.background.strong.color.scale_alpha(0.35),
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
        snap: false,
    }
}

pub fn backdrop(theme: &Theme) -> iced::widget::container::Style {
    let palette = theme.extended_palette();
    let dim = if palette.is_dark {
        iced::Color::BLACK.scale_alpha(0.45)
    } else {
        iced::Color::BLACK.scale_alpha(0.32)
    };

    iced::widget::container::Style {
        background: Some(iced::Background::Color(dim)),
        ..Default::default()
    }
}
