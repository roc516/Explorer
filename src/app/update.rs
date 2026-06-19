use iced::Task;

use crate::message::{explorer, file_list as file_list_msg, input, tree, Message};
use crate::widget::directory_tree::Action as TreeAction;
use crate::widget::file_list::{load_directory_task, Action as FileListAction};

use super::ExplorerApp;

fn load_directory(path: std::path::PathBuf) -> Task<Message> {
    load_directory_task(path).map(Message::FileList)
}

pub fn explorer(app: &mut ExplorerApp, message: explorer::Message) -> Task<Message> {
    match message {
        explorer::Message::GoUp => app
            .model
            .go_up()
            .map(load_directory)
            .unwrap_or_else(Task::none),
        explorer::Message::GoBack => app
            .model
            .go_back()
            .map(load_directory)
            .unwrap_or_else(Task::none),
        explorer::Message::GoForward => app
            .model
            .go_forward()
            .map(load_directory)
            .unwrap_or_else(Task::none),
        explorer::Message::Refresh => app
            .model
            .refresh()
            .map(load_directory)
            .unwrap_or_else(Task::none),
        explorer::Message::AddressEdited(value) => {
            app.model.set_address(value);
            Task::none()
        }
        explorer::Message::AddressSubmit => app
            .model
            .submit_address()
            .map(load_directory)
            .unwrap_or_else(Task::none),
    }
}

pub fn file_list(app: &mut ExplorerApp, message: file_list_msg::Message) -> Task<Message> {
    let (task, action) = app.file_list.update(&mut app.model, message);
    let mut tasks = vec![task.map(Message::FileList)];

    if let Some(FileListAction::DirectoryChanged(path)) = action {
        tasks.push(app.directory_tree.sync_path(&path).map(Message::Tree));
    }

    Task::batch(tasks)
}

pub fn tree(app: &mut ExplorerApp, message: tree::Message) -> Task<Message> {
    let (task, action) = app.directory_tree.update(message);
    let mut tasks = vec![task.map(Message::Tree)];

    if let Some(TreeAction::Navigate(path)) = action {
        if let Some(load_path) = app.model.navigate(path) {
            tasks.push(load_directory(load_path));
        }
    }

    Task::batch(tasks)
}

pub fn theme(app: &mut ExplorerApp, message: crate::message::theme::Message) -> Task<Message> {
    match message {
        crate::message::theme::Message::Selected(choice) => {
            app.theme_choice = choice;
            Task::none()
        }
        crate::message::theme::Message::SystemChanged(mode) => {
            app.system_mode = mode;
            Task::none()
        }
    }
}

pub fn settings(app: &mut ExplorerApp, message: crate::message::settings::Message) -> Task<Message> {
    match message {
        crate::message::settings::Message::Toggle => {
            app.settings_open = !app.settings_open;
        }
        crate::message::settings::Message::Close => {
            app.settings_open = false;
        }
        crate::message::settings::Message::PressInside => {}
    }
    Task::none()
}

pub fn locale(app: &mut ExplorerApp, message: crate::message::locale::Message) -> Task<Message> {
    let crate::message::locale::Message::Selected(language) = message;
    app.language = language;
    app.model.set_locale(app.effective_locale());
    Task::none()
}

pub fn input(app: &mut ExplorerApp, message: input::Message) -> Task<Message> {
    let input::Message::KeyPressed(key, modifiers) = message;

    if modifiers.control() {
        return Task::none();
    }

    match key {
        iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape) if app.settings_open => {
            return settings(app, crate::message::settings::Message::Close);
        }
        iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter) => {
            if let Some(index) = app.model.selected_index {
                return file_list(
                    app,
                    file_list_msg::Message::EntryDoubleClicked(index),
                );
            }
        }
        iced::keyboard::Key::Named(iced::keyboard::key::Named::Backspace) => {
            return explorer(app, explorer::Message::GoUp);
        }
        iced::keyboard::Key::Named(iced::keyboard::key::Named::F5) => {
            return explorer(app, explorer::Message::Refresh);
        }
        iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowLeft) => {
            return explorer(app, explorer::Message::GoBack);
        }
        iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowRight) => {
            return explorer(app, explorer::Message::GoForward);
        }
        _ => {}
    }

    Task::none()
}
