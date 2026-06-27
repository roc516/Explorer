use std::fmt;

use explorer_ui::ids;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTheme {
    System,
    Light,
    Dark,
    Dracula,
    Nord,
    SolarizedLight,
    SolarizedDark,
    GruvboxLight,
    GruvboxDark,
    CatppuccinLatte,
    CatppuccinFrappe,
    CatppuccinMacchiato,
    CatppuccinMocha,
    TokyoNight,
    TokyoNightStorm,
    TokyoNightLight,
    KanagawaWave,
    KanagawaDragon,
    KanagawaLotus,
    Moonfly,
    Nightfly,
    Oxocarbon,
    Ferra,
}

impl AppTheme {
    pub const OPTIONS: &'static [Self] = &[
        Self::System,
        Self::Light,
        Self::Dark,
        Self::Dracula,
        Self::Nord,
        Self::SolarizedLight,
        Self::SolarizedDark,
        Self::GruvboxLight,
        Self::GruvboxDark,
        Self::CatppuccinLatte,
        Self::CatppuccinFrappe,
        Self::CatppuccinMacchiato,
        Self::CatppuccinMocha,
        Self::TokyoNight,
        Self::TokyoNightStorm,
        Self::TokyoNightLight,
        Self::KanagawaWave,
        Self::KanagawaDragon,
        Self::KanagawaLotus,
        Self::Moonfly,
        Self::Nightfly,
        Self::Oxocarbon,
        Self::Ferra,
    ];

    pub fn message_id(self) -> &'static str {
        match self {
            Self::System => ids::THEME_SYSTEM,
            Self::Light => ids::THEME_LIGHT,
            Self::Dark => ids::THEME_DARK,
            Self::Dracula => ids::THEME_DRACULA,
            Self::Nord => ids::THEME_NORD,
            Self::SolarizedLight => ids::THEME_SOLARIZED_LIGHT,
            Self::SolarizedDark => ids::THEME_SOLARIZED_DARK,
            Self::GruvboxLight => ids::THEME_GRUVBOX_LIGHT,
            Self::GruvboxDark => ids::THEME_GRUVBOX_DARK,
            Self::CatppuccinLatte => ids::THEME_CATPPUCCIN_LATTE,
            Self::CatppuccinFrappe => ids::THEME_CATPPUCCIN_FRAPPE,
            Self::CatppuccinMacchiato => ids::THEME_CATPPUCCIN_MACCHIATO,
            Self::CatppuccinMocha => ids::THEME_CATPPUCCIN_MOCHA,
            Self::TokyoNight => ids::THEME_TOKYO_NIGHT,
            Self::TokyoNightStorm => ids::THEME_TOKYO_NIGHT_STORM,
            Self::TokyoNightLight => ids::THEME_TOKYO_NIGHT_LIGHT,
            Self::KanagawaWave => ids::THEME_KANAGAWA_WAVE,
            Self::KanagawaDragon => ids::THEME_KANAGAWA_DRAGON,
            Self::KanagawaLotus => ids::THEME_KANAGAWA_LOTUS,
            Self::Moonfly => ids::THEME_MOONFLY,
            Self::Nightfly => ids::THEME_NIGHTFLY,
            Self::Oxocarbon => ids::THEME_OXOCARBON,
            Self::Ferra => ids::THEME_FERRA,
        }
    }

    pub fn resolve(self, system_mode: iced::theme::Mode) -> iced::Theme {
        use iced::theme::{Base, Theme as IcedTheme};
        match self {
            Self::System => IcedTheme::default(system_mode),
            Self::Light => IcedTheme::Light,
            Self::Dark => IcedTheme::Dark,
            Self::Dracula => IcedTheme::Dracula,
            Self::Nord => IcedTheme::Nord,
            Self::SolarizedLight => IcedTheme::SolarizedLight,
            Self::SolarizedDark => IcedTheme::SolarizedDark,
            Self::GruvboxLight => IcedTheme::GruvboxLight,
            Self::GruvboxDark => IcedTheme::GruvboxDark,
            Self::CatppuccinLatte => IcedTheme::CatppuccinLatte,
            Self::CatppuccinFrappe => IcedTheme::CatppuccinFrappe,
            Self::CatppuccinMacchiato => IcedTheme::CatppuccinMacchiato,
            Self::CatppuccinMocha => IcedTheme::CatppuccinMocha,
            Self::TokyoNight => IcedTheme::TokyoNight,
            Self::TokyoNightStorm => IcedTheme::TokyoNightStorm,
            Self::TokyoNightLight => IcedTheme::TokyoNightLight,
            Self::KanagawaWave => IcedTheme::KanagawaWave,
            Self::KanagawaDragon => IcedTheme::KanagawaDragon,
            Self::KanagawaLotus => IcedTheme::KanagawaLotus,
            Self::Moonfly => IcedTheme::Moonfly,
            Self::Nightfly => IcedTheme::Nightfly,
            Self::Oxocarbon => IcedTheme::Oxocarbon,
            Self::Ferra => IcedTheme::Ferra,
        }
    }
}

#[derive(Clone, Eq)]
pub struct ThemeOption {
    pub theme: AppTheme,
    pub label: String,
}

impl PartialEq for ThemeOption {
    fn eq(&self, other: &Self) -> bool {
        self.theme == other.theme
    }
}

impl fmt::Display for ThemeOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.label)
    }
}

pub fn theme_options(bundle: explorer_ui::LanguageBundle) -> Vec<ThemeOption> {
    AppTheme::OPTIONS
        .iter()
        .copied()
        .map(|theme| ThemeOption {
            theme,
            label: bundle.tr(theme.message_id()),
        })
        .collect()
}
