use iced::widget::container;
use iced::{alignment, Element, Length};
use lucide_icons::Icon;

use crate::widget::lucide_icon;

const ICON_SIZE: f32 = 16.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FolderKind {
    Drive,
    Closed,
    Open,
}

pub fn chevron<'a, Message: 'a>(
    expanded: bool,
    loading: bool,
    width: f32,
    height: f32,
) -> Element<'a, Message> {
    let icon = if loading {
        Icon::Loader2
    } else if expanded {
        Icon::ChevronDown
    } else {
        Icon::ChevronRight
    };

    centered_icon(icon, width, height)
}

pub fn folder<'a, Message: 'a>(
    kind: FolderKind,
    width: f32,
    height: f32,
) -> Element<'a, Message> {
    let icon = match kind {
        FolderKind::Drive => Icon::HardDrive,
        FolderKind::Closed => Icon::Folder,
        FolderKind::Open => Icon::FolderOpen,
    };

    centered_icon(icon, width, height)
}

fn centered_icon<'a, Message: 'a>(
    icon: Icon,
    width: f32,
    height: f32,
) -> Element<'a, Message> {
    container(lucide_icon::icon_muted::<Message>(icon, ICON_SIZE, 0.72))
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .into()
}

pub fn folder_kind(row: &explorer_ui::TreeRow) -> FolderKind {
    if row.depth == 0 {
        FolderKind::Drive
    } else if row.expanded && row.expandable {
        FolderKind::Open
    } else {
        FolderKind::Closed
    }
}
