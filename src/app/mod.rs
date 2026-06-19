mod update;

use explorer_core::{detect_system_locale, ids, ExplorerModel, Language, LanguageBundle, Locale};
use iced::theme::Mode;
use iced::{keyboard, system, Element, Subscription, Task, Theme};

use crate::message::{input, theme, Message};
use crate::theme::AppTheme;
use crate::view;
use crate::widget::{
    directory_tree::DirectoryTreeWidget, file_list::FileListWidget,
    settings_dialog::SettingsDialogWidget, status_bar::StatusBarWidget,
    toolbar::ToolbarWidget,
};

pub struct ExplorerApp {
    pub model: ExplorerModel,
    pub toolbar: ToolbarWidget,
    pub directory_tree: DirectoryTreeWidget,
    pub file_list: FileListWidget,
    pub status_bar: StatusBarWidget,
    pub settings_dialog: SettingsDialogWidget,
    pub theme_choice: AppTheme,
    pub system_mode: Mode,
    pub language: Language,
    pub system_locale: Locale,
    pub settings_open: bool,
}
impl ExplorerApp {
    pub fn new() -> (Self, Task<Message>) {
        let system_locale = detect_system_locale();
        let language = Language::default();
        let mut model = ExplorerModel::new();
        model.set_locale(language.resolve(system_locale));
        let initial_path = model.current_path.clone();

        (
            Self {
                model,
                toolbar: ToolbarWidget::new(),
                directory_tree: DirectoryTreeWidget::new(),
                file_list: FileListWidget::new(),
                status_bar: StatusBarWidget::new(),
                settings_dialog: SettingsDialogWidget::new(),
                theme_choice: AppTheme::System,
                system_mode: Mode::default(),
                language,
                system_locale,
                settings_open: false,
            },
            Task::batch([
                crate::widget::file_list::load_directory_task(initial_path)
                    .map(Message::FileList),
                system::theme().map(|mode| Message::Theme(theme::Message::SystemChanged(mode))),
            ]),
        )
    }

    pub fn effective_locale(&self) -> Locale {
        self.language.resolve(self.system_locale)
    }

    pub fn bundle(&self) -> LanguageBundle {
        self.model.bundle
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Explorer(message) => update::explorer(self, message),
            Message::FileList(message) => update::file_list(self, message),
            Message::Tree(message) => update::tree(self, message),
            Message::Theme(message) => update::theme(self, message),
            Message::Locale(message) => update::locale(self, message),
            Message::Settings(message) => update::settings(self, message),
            Message::Input(message) => update::input(self, message),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        view::view(self)
    }

    pub fn subscription(_state: &Self) -> Subscription<Message> {
        Subscription::batch([
            keyboard::listen().filter_map(|event| {
                if let keyboard::Event::KeyPressed { key, modifiers, .. } = event {
                    Some(Message::Input(input::Message::KeyPressed(key, modifiers)))
                } else {
                    None
                }
            }),
            system::theme_changes()
                .map(|mode| Message::Theme(theme::Message::SystemChanged(mode))),
        ])
    }

    pub fn theme(state: &Self) -> Theme {
        state.theme_choice.resolve(state.system_mode)
    }

    pub fn title(&self) -> String {
        self.model.bundle.tr(ids::WINDOW_TITLE)
    }
}
