use std::collections::BTreeMap;

use explorer_core::BlockDevice;
use explorer_app::{detect_system_locale, ids, Language, Locale};
use iced::keyboard;
use iced::theme::Mode;
use iced::widget::stack;
use iced::window;
use iced::window::settings::PlatformSpecific;
use iced::{Element, Fill, Subscription, Task, Theme};

use crate::message::{input, window as window_msg, Message, Launch};
use crate::theme::AppTheme;
use crate::ui::explorer::Explorer;
use crate::ui::file_list;
use crate::ui::settings::{self as settings_ui, Settings};

const WINDOW_WIDTH: f32 = 1200.0;
const WINDOW_HEIGHT: f32 = 760.0;

pub struct App {
    windows: BTreeMap<window::Id, Explorer>,
    focused_window: Option<window::Id>,
    settings: Settings,
    settings_open: bool,
    theme_choice: AppTheme,
    system_mode: Mode,
    language: Language,
    system_locale: Locale,
}

impl App {
    pub fn boot() -> (Self, Task<Message>) {
        explorer_core::ensure_host_registered(Box::new(explorer_fs_folder::FolderBackend));
        explorer_core::ensure_backends_registered(|registry| {
            registry.register(Box::new(explorer_fs_zip::ZipBackend));
        });

        let system_locale = detect_system_locale();
        let language = Language::default();
        let (_, open) = window::open(window_settings());

        (
            Self {
                windows: BTreeMap::new(),
                focused_window: None,
                settings: Settings::new(),
                settings_open: false,
                theme_choice: AppTheme::System,
                system_mode: Mode::default(),
                language,
                system_locale,
            },
            open.map(|id| Message::WindowOpened(id, Launch::Local)),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Window(id, message) => self.update_window(id, message),
            Message::WindowOpened(id, launch) => self.on_window_opened(id, launch),
            Message::WindowClosed(id) => self.on_window_closed(id),
            Message::WindowFocused(id) => {
                self.focused_window = Some(id);
                Task::none()
            }
            Message::Locale(message) => self.update_locale(message),
            Message::Settings(message) => self.update_settings(message),
        }
    }

    pub fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        let Some(window) = self.windows.get(&window_id) else {
            return iced::widget::Space::new().into();
        };

        // Keep Stack as the stable root. Toggling settings must only add/remove the
        // overlay child — switching root from Column to Stack rebuilds the whole UI
        // tree (file list + directory tree) and feels like a hitch on open.
        let mut layers = vec![window.view(window_id)];
        if self.settings_open {
            layers.push(self.settings.view(
                window.model.bundle.clone(),
                self.theme_choice,
                self.language,
            ));
        }

        stack(layers).width(Fill).height(Fill).into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = vec![
            window::close_events().map(Message::WindowClosed),
            window::events().filter_map(|(id, event)| {
                (event == window::Event::Focused).then_some(Message::WindowFocused(id))
            }),
            iced::system::theme_changes()
                .map(|mode| Message::Settings(settings_ui::Message::SystemThemeChanged(mode))),
        ];

        subscriptions.push(
            keyboard::listen()
                .with(self.focused_window)
                .filter_map(|(focused, event)| {
                    let id = focused?;
                    if let keyboard::Event::KeyPressed { key, modifiers, .. } = event {
                        Some(Message::Window(
                            id,
                            window_msg::Message::Input(input::Message::KeyPressed(key, modifiers)),
                        ))
                    } else {
                        None
                    }
                }),
        );

        let file_list_subscriptions: Vec<_> = self
            .windows
            .iter()
            .map(|(id, window)| {
                (
                    *id,
                    window.file_list.subscription(),
                )
            })
            .collect();

        for (id, subscription) in file_list_subscriptions {
            subscriptions.push(
                subscription
                    .with(id)
                    .map(|(id, message)| Message::Window(id, window_msg::Message::FileList(message))),
            );
        }

        Subscription::batch(subscriptions)
    }

    pub fn theme(&self, _window: window::Id) -> Option<Theme> {
        Some(self.theme_choice.resolve(self.system_mode))
    }

    pub fn title(&self, window_id: window::Id) -> String {
        self.windows
            .get(&window_id)
            .map(|window| {
                let title = window.model.bundle.tr(ids::WINDOW_TITLE);
                format!("{title} — {}", window.model.display_path())
            })
            .unwrap_or_else(|| "Explorer".to_string())
    }

    fn on_window_opened(&mut self, id: window::Id, launch: Launch) -> Task<Message> {
        let locale = self.language.resolve(self.system_locale);
        let mut explorer = match launch {
            Launch::Local => Explorer::new_local(locale),
            Launch::Archive(device) => Explorer::new_mounted(device, locale),
        };

        let load_dir = explorer.model.current_dir().clone();
        let init_path = explorer.model.current_path().to_path_buf();
        let init_tree_task = explorer
            .directory_tree
            .sync_path(&init_path)
            .map(move |msg| Message::Window(id, window_msg::Message::Tree(msg)));

        self.focused_window = Some(id);
        self.windows.insert(id, explorer);

        let load_task = file_list::load_directory_from_dir(load_dir)
            .map(move |message| Message::Window(id, window_msg::Message::FileList(message)));

        Task::batch(vec![init_tree_task, load_task])
    }

    fn on_window_closed(&mut self, id: window::Id) -> Task<Message> {
        self.windows.remove(&id);
        if self.focused_window == Some(id) {
            self.focused_window = self.windows.keys().next().copied();
        }

        if self.windows.is_empty() {
            iced::exit()
        } else {
            Task::none()
        }
    }

    fn open_mounted_window(&self, device: BlockDevice) -> Task<Message> {
        let (_, open) = window::open(window_settings());
        open.map(move |id| Message::WindowOpened(id, Launch::Archive(device.clone())))
    }

    fn update_window(&mut self, id: window::Id, message: window_msg::Message) -> Task<Message> {
        let Some(window) = self.windows.get_mut(&id) else {
            return Task::none();
        };

        let task = match message {
            window_msg::Message::Explorer(message) => window.update_explorer(message),
            window_msg::Message::FileList(message) => {
                let (task, _open_mounted) = window.update_file_list(message);
                if let Some(container) = _open_mounted {
                    return Task::batch([
                        task.map(move |message| Message::Window(id, message)),
                        self.open_mounted_window(container),
                    ]);
                }
                task
            }
            window_msg::Message::Tree(message) => window.update_tree(message),
            window_msg::Message::Preview(message) => window.update_preview(message),
            window_msg::Message::Input(message) => {
                window.update_input(message, self.settings_open)
            }
        };

        task.map(move |message| Message::Window(id, message))
    }

    fn update_settings(&mut self, message: settings_ui::Message) -> Task<Message> {
        match message {
            settings_ui::Message::Toggle => {
                self.settings_open = !self.settings_open;
            }
            settings_ui::Message::Close => {
                self.settings_open = false;
            }
            settings_ui::Message::ThemeSelected(choice) => {
                self.theme_choice = choice;
            }
            settings_ui::Message::SystemThemeChanged(mode) => {
                self.system_mode = mode;
            }
        }
        Task::none()
    }

    fn update_locale(&mut self, message: settings_ui::locale::Message) -> Task<Message> {
        let settings_ui::locale::Message::Selected(language) = message;
        self.language = language;
        let locale = self.language.resolve(self.system_locale);
        for window in self.windows.values_mut() {
            window.model.set_locale(locale);
        }
        Task::none()
    }
}

fn window_settings() -> window::Settings {
    window::Settings {
        size: iced::Size::new(WINDOW_WIDTH, WINDOW_HEIGHT),
        min_size: Some(iced::Size::new(800.0, 500.0)),
        icon: Some(crate::window_icon::app_icon()),
        #[cfg(target_os = "linux")]
        platform_specific: PlatformSpecific {
            application_id: "org.explorer.app".to_string(),
            ..Default::default()
        },
        ..Default::default()
    }
}
