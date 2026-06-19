use iced::widget::{column, container, mouse_area, row, rule, stack};
use iced::{Element, Fill};

use crate::app::ExplorerApp;
use crate::message::{settings, Message};
use crate::widget::settings_dialog;

pub fn view(app: &ExplorerApp) -> Element<'_, Message> {
    let bundle = app.bundle();

    let main = column![
        app.toolbar.view(
            bundle,
            &app.model.address_input,
            app.model.can_go_back(),
            app.model.can_go_forward(),
            app.model.can_go_up(),
        ),
        rule::horizontal(1),
        row![
            app.directory_tree.view(bundle).map(Message::Tree),
            rule::vertical(1),
            app.file_list.view(&app.model).map(Message::FileList),
        ]
        .spacing(0)
        .width(Fill)
        .height(Fill),
        rule::horizontal(1),
        app.status_bar.view(&app.model),
    ]
    .width(Fill)
    .height(Fill);

    if app.settings_open {
        let overlay = mouse_area(
            container(
                app.settings_dialog
                    .view(bundle, app.theme_choice, app.language),
            )
            .width(Fill)
            .height(Fill)
            .style(settings_dialog::backdrop),
        )
        .on_press(Message::Settings(settings::Message::Close));

        stack![main, overlay]
            .width(Fill)
            .height(Fill)
            .into()
    } else {
        main.into()
    }
}
