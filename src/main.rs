mod app;
mod fluent;
mod message;
mod theme;
mod view;
mod widget;
mod window_icon;

use app::ExplorerApp;
use iced::{application, window, Size};
use lucide_icons::LUCIDE_FONT_BYTES;

use crate::window_icon::app_icon;

fn main() -> iced::Result {
    application(ExplorerApp::new, ExplorerApp::update, ExplorerApp::view)
        .font(LUCIDE_FONT_BYTES)
        .title(ExplorerApp::title)
        .theme(ExplorerApp::theme)
        .subscription(ExplorerApp::subscription)
        .window(window::Settings {
            size: Size::new(1200.0, 760.0),
            min_size: Some(Size::new(800.0, 500.0)),
            icon: Some(app_icon()),
            ..Default::default()
        })
        .run()
}
