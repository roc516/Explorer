mod icons;

use std::path::PathBuf;
use std::sync::Arc;

use explorer_core::{BlockDevice, DirEntry};
use explorer_app::{load_tree_children, TreeState, TreeNode, TreeRow};
use iced::widget::{button, column, container, mouse_area, row, scrollable, text, Space};
use iced::{alignment, Element, Fill, Length, Task, Theme};

use crate::fluent::{
    HEIGHT_LIST_ROW, NAV_PANE_WIDTH, PAGE_PADDING_H, RADIUS_CONTROL, SPACE_LG,
    SPACE_SM, SPACE_XS,
};

#[derive(Debug, Clone)]
pub enum Message {
    Toggle(PathBuf),
    Select(PathBuf),
    ChildrenLoaded(PathBuf, Result<Vec<TreeNode>, String>),
}

const INDENT: f32 = 16.0;
const CHEVRON_WIDTH: f32 = 24.0;
const ICON_WIDTH: f32 = 18.0;

#[derive(Debug, Clone)]
pub enum Action {
    Navigate(Arc<dyn DirEntry>),
}

pub struct DirectoryTree {
    state: TreeState,
    width: Length,
}

impl DirectoryTree {
    pub fn new() -> Self {
        Self::with_tree(TreeState::new())
    }

    pub fn for_mounted(device: BlockDevice) -> Self {
        Self::with_tree(TreeState::for_mounted(device))
    }

    fn with_tree(state: TreeState) -> Self {
        Self {
            state,
            width: Length::Fixed(NAV_PANE_WIDTH),
        }
    }

    pub fn update(&mut self, message: Message) -> (Task<Message>, Option<Action>) {
        match message {
            Message::Toggle(path) => {
                let task = self
                    .state
                    .toggle(path)
                    .map(load_children_task)
                    .unwrap_or_else(Task::none);
                (task, None)
            }
            Message::Select(path) => {
                let action = self.state.select(path).map(Action::Navigate);
                (Task::none(), action)
            }
            Message::ChildrenLoaded(path, result) => {
                let task = self
                    .state
                    .on_children_loaded(path, result)
                    .map(load_children_task)
                    .unwrap_or_else(Task::none);
                (task, None)
            }
        }
    }

    pub fn sync_path(&mut self, path: &std::path::Path) -> Task<Message> {
        self.state
            .sync_selection(path)
            .map(load_children_task)
            .unwrap_or_else(Task::none)
    }

    pub fn refresh(&mut self) -> Task<Message> {
        self.state
            .refresh()
            .map(load_children_task)
            .unwrap_or_else(Task::none)
    }

    pub fn view(&self, bundle: explorer_app::LanguageBundle) -> Element<'_, Message> {
        let rows = self.state.rows();
        let no_locations = bundle.tr(explorer_app::ids::TREE_NO_LOCATIONS);
        let content: Element<'_, Message> = if rows.is_empty() {
            column![container(text(no_locations).size(13).style(empty_hint)).padding([
                SPACE_LG, PAGE_PADDING_H
            ])]
            .into()
        } else {
            column(rows.into_iter().map(view_row).collect::<Vec<_>>())
                .spacing(SPACE_XS)
                .padding([SPACE_XS, PAGE_PADDING_H])
                .into()
        };

        container(scrollable(content).width(Fill).height(Fill))
            .width(self.width)
            .height(Fill)
            .style(panel_container)
            .into()
    }
}

impl Default for DirectoryTree {
    fn default() -> Self {
        Self::new()
    }
}

fn load_children_task(dir: Arc<dyn DirEntry>) -> Task<Message> {
    let nav = dir.path().to_path_buf();
    Task::perform(
        async move { load_tree_children(&dir) },
        move |result| Message::ChildrenLoaded(nav, result),
    )
}

fn view_row(row: TreeRow) -> Element<'static, Message> {
    let chevron = chevron_widget(&row);
    let folder = icons::folder::<Message>(
        icons::folder_kind(&row),
        ICON_WIDTH,
        HEIGHT_LIST_ROW,
    );
    let name = text(row.name).size(13);

    let label = mouse_area(
        row![folder, name]
            .spacing(SPACE_SM)
            .align_y(alignment::Vertical::Center)
            .width(Fill),
    )
    .on_press(Message::Select(row.path.clone()));

    container(
        row![
            Space::new().width(Length::Fixed(row.depth as f32 * INDENT)),
            chevron,
            label,
        ]
        .spacing(0)
        .align_y(alignment::Vertical::Center)
        .width(Fill),
    )
    .height(Length::Fixed(HEIGHT_LIST_ROW))
    .width(Fill)
    .style(if row.selected {
        selected_row_container
    } else {
        normal_row_container
    })
    .padding([0.0, SPACE_XS])
    .into()
}

fn chevron_widget(row: &TreeRow) -> Element<'static, Message> {
    if !row.expandable {
        return Space::new()
            .width(Length::Fixed(CHEVRON_WIDTH))
            .into();
    }

    button(
        icons::chevron::<Message>(
            row.expanded,
            row.loading,
            CHEVRON_WIDTH,
            HEIGHT_LIST_ROW,
        ),
    )
    .on_press(Message::Toggle(row.path.clone()))
    .width(Length::Fixed(CHEVRON_WIDTH))
    .height(Length::Fixed(HEIGHT_LIST_ROW))
    .padding(0)
    .style(chevron_button)
    .into()
}

fn empty_hint(theme: &Theme) -> iced::widget::text::Style {
    let palette = theme.extended_palette();
    iced::widget::text::Style {
        color: Some(palette.background.base.text.scale_alpha(0.55)),
    }
}

fn panel_container(_theme: &Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        border: iced::Border {
            width: 0.0,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn normal_row_container(_theme: &Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        border: iced::Border {
            radius: RADIUS_CONTROL.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn selected_row_container(theme: &Theme) -> iced::widget::container::Style {
    let palette = theme.extended_palette();
    iced::widget::container::Style {
        background: Some(iced::Background::Color(palette.primary.weak.color)),
        border: iced::Border {
            radius: RADIUS_CONTROL.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn chevron_button(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let radius = RADIUS_CONTROL.into();

    match status {
        button::Status::Hovered => button::Style {
            background: Some(iced::Background::Color(
                palette.background.strong.color.scale_alpha(0.35),
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
                palette.background.strong.color.scale_alpha(0.45),
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
