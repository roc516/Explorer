mod app;
mod fluent;
mod message;
mod theme;
mod widget;
mod window_icon;

use app::App;
use iced::{daemon, Size};
use lucide_icons::LUCIDE_FONT_BYTES;

fn main() -> iced::Result {
    daemon(App::boot, App::update, App::view)
        .font(LUCIDE_FONT_BYTES)
        .title(App::title)
        .theme(App::theme)
        .subscription(App::subscription)
        .run()
}
