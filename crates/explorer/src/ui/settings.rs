use std::fmt;

use explorer_app::{ids, Language, LanguageBundle};
use iced::theme::Mode;
use iced::widget::{column, container, pick_list, row, rule, text, Space};
use iced::{alignment, Element, Fill, Length, Theme};
use lucide_icons::Icon;

use crate::fluent::{
    DIALOG_WIDTH_SETTINGS, HEIGHT_SETTING_ROW, SPACE_LG, SPACE_MD, SPACE_SM, WIDTH_SETTING_COMBO,
};
use crate::message::Message as AppMessage;
use crate::theme::{theme_options, AppTheme};
use crate::ui::dialog::Dialog;
use crate::ui::style::pick_list_style;
use crate::widget::LucideIcon;

pub mod locale {
    use explorer_app::Language;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Message {
        Selected(Language),
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Toggle,
    Close,
    ThemeSelected(AppTheme),
    SystemThemeChanged(Mode),
}

const THEME_MENU_HEIGHT: f32 = 280.0;

pub struct Settings;

impl Settings {
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
            |option| AppMessage::Settings(Message::ThemeSelected(option.theme)),
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

        let body = container(
            column![
                setting_row(theme_label, Icon::Palette, theme_picker.into()),
                rule::horizontal(1).style(group_divider),
                setting_row(language_label, Icon::Languages, language_picker.into()),
            ]
            .width(Fill),
        )
        .padding([SPACE_MD, SPACE_LG]);

        Dialog::new(title, AppMessage::Settings(Message::Close))
            .width(DIALOG_WIDTH_SETTINGS)
            .body(body)
            .view()
    }
}

impl Default for Settings {
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

fn setting_row(
    label: String,
    icon: Icon,
    control: Element<'_, AppMessage>,
) -> Element<'_, AppMessage> {
    row![
        row![
            LucideIcon::new(icon).size(14.0).muted(0.72),
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
