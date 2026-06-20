use std::path::PathBuf;

use explorer_core::{load_tree_children, EPath, DirectoryTree, TreeNode, TreeRow};
use iced::widget::{button, column, container, mouse_area, row, scrollable, text, Space};
use iced::{alignment, Element, Fill, Length, Task, Theme};

use crate::fluent::{
    HEIGHT_LIST_ROW, NAV_PANE_WIDTH, PAGE_PADDING_H, RADIUS_CONTROL, SPACE_LG,
    SPACE_SM, SPACE_XS,
};
use crate::widget::tree_icons;

#[derive(Debug, Clone)]
pub enum Message {
    Toggle(EPath),
    Select(EPath),
    ChildrenLoaded(EPath, Result<Vec<TreeNode>, String>),
}

const INDENT: f32 = 16.0;
const CHEVRON_WIDTH: f32 = 24.0;
const ICON_WIDTH: f32 = 18.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Navigate(EPath),
}

pub struct DirectoryTreeWidget {
    state: DirectoryTree,
    width: Length,
}

impl DirectoryTreeWidget {
    pub fn new() -> Self {
        Self::with_tree(DirectoryTree::new())
    }

    pub fn for_mounted(container: PathBuf) -> Self {
        Self::with_tree(DirectoryTree::for_mounted(container))
    }

    fn with_tree(state: DirectoryTree) -> Self {
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
                self.state.select(path.clone());
                (Task::none(), Some(Action::Navigate(path)))
            }
            Message::ChildrenLoaded(path, result) => {
                self.state.on_children_loaded(path, result);
                (Task::none(), None)
            }
        }
    }

    pub fn sync_path(&mut self, path: &EPath) -> Task<Message> {
        let pending = self.state.sync_selection(path);
        Task::batch(pending.into_iter().map(load_children_task))
    }

    pub fn view(&self, bundle: explorer_core::LanguageBundle) -> Element<'_, Message> {
        let rows = self.state.rows();
        let no_locations = bundle.tr(explorer_core::ids::TREE_NO_LOCATIONS);
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

impl Default for DirectoryTreeWidget {
    fn default() -> Self {
        Self::new()
    }
}

fn load_children_task(path: EPath) -> Task<Message> {
    Task::perform(
        {
            let load_path = path.clone();
            async move { load_tree_children(&load_path) }
        },
        move |result| Message::ChildrenLoaded(path, result),
    )
}

fn view_row(row: TreeRow) -> Element<'static, Message> {
    let chevron = chevron_widget(&row);
    let folder = tree_icons::folder::<Message>(
        tree_icons::folder_kind(&row),
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
    .padding([0.0, SPACE_XS])
    .style(if row.selected {
        selected_row_container
    } else {
        normal_row_container
    })
    .into()
}

fn chevron_widget(row: &TreeRow) -> Element<'static, Message> {
    if !row.expandable {
        return Space::new()
            .width(Length::Fixed(CHEVRON_WIDTH))
            .into();
    }

    button(
        tree_icons::chevron::<Message>(
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