mod icons;

use explorer_core::EPath;
use explorer_app::{
    breadcrumbs, ids, ExplorerModel, LanguageBundle, ModelError, NavigationHistory,
};
use iced::window as iced_window;
use iced::widget::{button, container, mouse_area, row, scrollable, text, text_input};
use iced::{alignment, Element, Fill, Task, Theme};
use iced::widget::operation::{focus, select_all};

use crate::fluent::{
    FONT_SIZE_ADDRESS, FONT_SIZE_BREADCRUMB_SEP, HEIGHT_COMMAND_BAR, PAGE_PADDING_H,
    RADIUS_CONTROL, SPACE_MD, SPACE_SM, SPACE_XS,
};
use crate::message::{settings, window as window_msg, Message as AppMessage};

use icons::{self as toolbar_icons, NavIcon};

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

#[derive(Debug, Clone)]
pub enum Action {
    Load(EPath),
}

pub struct Toolbar {
    navigation: NavigationHistory,
    address_input: String,
    address_editing: bool,
    reveal_path: Option<EPath>,
}

impl Toolbar {
    pub fn new(initial_path: &EPath) -> Self {
        Self {
            navigation: NavigationHistory::new(initial_path.clone()),
            address_input: initial_path.display(),
            address_editing: false,
            reveal_path: None,
        }
    }

    pub fn push_history(&mut self, path: EPath) {
        self.navigation.push(path);
    }

    pub fn is_address_editing(&self) -> bool {
        self.address_editing
    }

    pub fn cancel_address_edit(&mut self, current_path: &EPath) {
        self.address_editing = false;
        self.address_input = current_path.display();
    }

    /// Sync address bar after a directory finished loading.
    /// Returns a path that should be selected in the file list, if any.
    pub fn on_directory_loaded(&mut self, path: &EPath) -> Option<EPath> {
        self.address_editing = false;
        if let Some(reveal) = self.reveal_path.take() {
            self.address_input = reveal.display();
            Some(reveal)
        } else {
            self.address_input = path.display();
            None
        }
    }

    pub fn update(
        &mut self,
        message: Message,
        model: &mut ExplorerModel,
    ) -> (Task<window_msg::Message>, Option<Action>) {
        match message {
            Message::GoUp => {
                let action = model.go_up().map(|path| {
                    self.push_history(path.clone());
                    Action::Load(path)
                });
                (Task::none(), action)
            }
            Message::GoBack => {
                let action = self.navigation.go_back().map(|path| {
                    model.begin_load();
                    Action::Load(path)
                });
                (Task::none(), action)
            }
            Message::GoForward => {
                let action = self.navigation.go_forward().map(|path| {
                    model.begin_load();
                    Action::Load(path)
                });
                (Task::none(), action)
            }
            Message::Refresh => {
                let action = model.refresh().map(Action::Load);
                (Task::none(), action)
            }
            Message::AddressEdited(value) => {
                self.address_input = value;
                (Task::none(), None)
            }
            Message::AddressEditStart => {
                self.address_editing = true;
                self.address_input = model.current_path.display();
                (
                    focus::<window_msg::Message>(ADDRESS_INPUT_ID)
                        .chain(select_all(ADDRESS_INPUT_ID)),
                    None,
                )
            }
            Message::BreadcrumbNavigate(path) => {
                self.address_editing = false;
                let action = model.navigate(path).map(|path| {
                    self.push_history(path.clone());
                    Action::Load(path)
                });
                (Task::none(), action)
            }
            Message::AddressSubmit => (Task::none(), self.submit_address(model)),
        }
    }

    pub fn view(
        &self,
        bundle: LanguageBundle,
        current_path: &EPath,
        can_go_up: bool,
        window_id: iced_window::Id,
    ) -> Element<'_, AppMessage> {
        let address_placeholder = bundle.tr(ids::TOOLBAR_ADDRESS_PLACEHOLDER);
        let can_go_back = self.navigation.can_go_back();
        let can_go_forward = self.navigation.can_go_forward();

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

        let address_bar = container(if self.address_editing {
            text_input(&address_placeholder, &self.address_input)
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

    fn submit_address(&mut self, model: &mut ExplorerModel) -> Option<Action> {
        self.address_editing = false;
        let path = EPath::from_address(&self.address_input, &model.current_path);

        if !path.exists() {
            model.set_path_error(ModelError::InvalidPath);
            return None;
        }

        if path.is_directory() {
            self.reveal_path = None;
            return model.navigate(path).map(|path| {
                self.push_history(path.clone());
                Action::Load(path)
            });
        }

        if path.is_file() {
            let Some(parent) = path.parent() else {
                model.set_path_error(ModelError::InvalidPath);
                return None;
            };

            if parent == model.current_path {
                self.address_input = path.display();
                model.error = None;
                model.select_path(&path);
                model.status = explorer_app::StatusInfo::ItemCount(model.entries.len());
                return None;
            }

            self.reveal_path = Some(path);
            return model.navigate(parent).map(|path| {
                self.push_history(path.clone());
                Action::Load(path)
            });
        }

        model.set_path_error(ModelError::InvalidPath);
        None
    }
}

impl Default for Toolbar {
    fn default() -> Self {
        Self::new(&EPath::local(std::path::PathBuf::from("/")))
    }
}

fn breadcrumb_bar(current_path: &EPath, window_id: iced_window::Id) -> Element<'static, AppMessage> {
    let crumbs = breadcrumbs(current_path);
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
