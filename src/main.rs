mod app;
mod fluent;
mod message;
mod theme;
mod widget;
mod window_icon;

use app::App;
use iced::{application, window, Size};
use lucide_icons::LUCIDE_FONT_BYTES;

use crate::window_icon::app_icon;

fn main() -> iced::Result {
    application(App::new, App::update, App::view)
        .font(LUCIDE_FONT_BYTES)
        .title(App::title)
        .theme(App::theme)
        .subscription(App::subscription)
        .window(window::Settings {
            size: Size::new(1200.0, 760.0),
            min_size: Some(Size::new(800.0, 500.0)),
            icon: Some(app_icon()),
            ..Default::default()
        })
        .run()
}
