mod icons;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use explorer_app::{load_tree_children, TreeState, TreeNode, TreeRow};
use explorer_core::DirEntry;
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

/// UI 状态：展开/加载/选中节点，业务数据由 `TreeState` 参数传入。
pub struct TreeUi {
    expanded: BTreeSet<PathBuf>,
    loading: BTreeSet<PathBuf>,
    selected: Option<PathBuf>,
    width: Length,
}

impl TreeUi {
    pub fn new() -> Self {
        Self {
            expanded: BTreeSet::new(),
            loading: BTreeSet::new(),
            selected: None,
            width: Length::Fixed(NAV_PANE_WIDTH),
        }
    }
}

impl Default for TreeUi {
    fn default() -> Self {
        Self::new()
    }
}

pub fn update(
    ui: &mut TreeUi,
    data: &mut TreeState,
    message: Message,
) -> (Task<Message>, Option<Action>) {
    match message {
        Message::Toggle(path) => {
            let task = toggle(ui, data, path)
                .map(load_children_task)
                .unwrap_or_else(Task::none);
            (task, None)
        }
        Message::Select(path) => {
            let action = select(ui, data, path).map(Action::Navigate);
            (Task::none(), action)
        }
        Message::ChildrenLoaded(path, result) => {
            let task = on_children_loaded(ui, data, path, result)
                .map(load_children_task)
                .unwrap_or_else(Task::none);
            (task, None)
        }
    }
}

pub fn sync_path(ui: &mut TreeUi, data: &TreeState, path: &Path) -> Task<Message> {
    sync_selection(ui, data, path)
        .map(load_children_task)
        .unwrap_or_else(Task::none)
}

pub fn refresh(ui: &mut TreeUi, data: &mut TreeState) -> Task<Message> {
    data.clear_children();
    ui.loading.clear();
    next_pending_load(ui, data)
        .map(load_children_task)
        .unwrap_or_else(Task::none)
}

pub fn view(
    ui: &TreeUi,
    data: &TreeState,
    bundle: explorer_app::LanguageBundle,
) -> Element<'static, Message> {
    let rows = rows(ui, data);
    let no_locations = bundle.tr(explorer_app::ids::TREE_NO_LOCATIONS);
    let content: Element<'static, Message> = if rows.is_empty() {
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
        .width(ui.width)
        .height(Fill)
        .style(panel_container)
        .into()
}

// —— 内部：UI 状态变更 + 业务数据查询 ——

fn toggle(ui: &mut TreeUi, data: &TreeState, path: PathBuf) -> Option<Arc<dyn DirEntry>> {
    if ui.expanded.contains(&path) {
        ui.expanded.remove(&path);
        return None;
    }

    ui.expanded.insert(path.clone());
    if data.has_children(&path) {
        None
    } else {
        begin_load(ui, data, &path)
    }
}

fn select(ui: &mut TreeUi, data: &TreeState, path: PathBuf) -> Option<Arc<dyn DirEntry>> {
    ui.selected = Some(path.clone());
    data.find_entry(&path).cloned()
}

fn on_children_loaded(
    ui: &mut TreeUi,
    data: &mut TreeState,
    path: PathBuf,
    result: Result<Vec<TreeNode>, String>,
) -> Option<Arc<dyn DirEntry>> {
    ui.loading.remove(&path);

    match result {
        Ok(children) => {
            data.insert_children(path, children);
        }
        Err(_) => {
            ui.expanded.remove(&path);
        }
    }

    next_pending_load(ui, data)
}

fn sync_selection(ui: &mut TreeUi, data: &TreeState, current: &Path) -> Option<Arc<dyn DirEntry>> {
    ui.selected = Some(current.to_path_buf());
    next_sync_load(ui, data, current)
}

fn next_sync_load(ui: &mut TreeUi, data: &TreeState, current: &Path) -> Option<Arc<dyn DirEntry>> {
    for path in ancestors_and_self(current) {
        ui.expanded.insert(path.clone());
        if data.has_children(&path) || ui.loading.contains(&path) {
            continue;
        }
        return begin_load(ui, data, &path);
    }
    None
}

fn next_pending_load(ui: &mut TreeUi, data: &TreeState) -> Option<Arc<dyn DirEntry>> {
    if let Some(selected) = ui.selected.clone() {
        if let Some(entry) = next_sync_load(ui, data, selected.as_path()) {
            return Some(entry);
        }
    }

    let mut candidates: Vec<_> = ui.expanded.iter().cloned().collect();
    candidates.sort_by_key(|path| path.components().count());
    for path in candidates {
        if data.has_children(&path) || ui.loading.contains(&path) {
            continue;
        }
        if let Some(entry) = begin_load(ui, data, &path) {
            return Some(entry);
        }
    }
    None
}

fn begin_load(ui: &mut TreeUi, data: &TreeState, path: &Path) -> Option<Arc<dyn DirEntry>> {
    let entry = data.find_entry(path)?.clone();
    ui.loading.insert(path.to_path_buf());
    Some(entry)
}

fn rows(ui: &TreeUi, data: &TreeState) -> Vec<TreeRow> {
    let mut rows = Vec::new();
    append_rows(ui, data.roots(), 0, data, &mut rows);
    rows
}

fn append_rows(
    ui: &TreeUi,
    nodes: &[TreeNode],
    depth: usize,
    data: &TreeState,
    rows: &mut Vec<TreeRow>,
) {
    for node in nodes {
        let path = node.path().to_path_buf();
        let expanded = ui.expanded.contains(&path);
        rows.push(TreeRow {
            path: path.clone(),
            name: node.name().to_string(),
            depth,
            expanded,
            loading: ui.loading.contains(&path),
            selected: ui.selected.as_ref() == Some(&path),
            expandable: is_expandable(ui, data, &path),
        });

        if expanded {
            if let Some(children) = data.children_of(&path) {
                append_rows(ui, children, depth + 1, data, rows);
            }
        }
    }
}

fn is_expandable(ui: &TreeUi, data: &TreeState, path: &Path) -> bool {
    if ui.loading.contains(path) {
        return true;
    }

    match data.children_of(path) {
        Some(children) => !children.is_empty(),
        None => true,
    }
}

fn ancestors_and_self(path: &Path) -> Vec<PathBuf> {
    let mut chain = Vec::new();
    let mut current = path.to_path_buf();
    loop {
        chain.push(current.clone());
        match current.parent() {
            Some(parent) if parent != current.as_path() => {
                current = parent.to_path_buf();
            }
            _ => break,
        }
    }
    chain.reverse();
    chain
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
