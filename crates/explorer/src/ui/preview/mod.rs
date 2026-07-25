mod document;
mod image;
mod text;

use explorer_core::EPath;
use explorer_app::{ids, LanguageBundle, PreviewFile, PreviewKind};
use fluent::{FluentArgs, FluentValue};
use iced::widget::{
    button, container, scrollable,
    text as text_widget, text_editor as text_editor_widget,
};
use iced::widget::{scrollable::Direction, text_editor};
use iced::{alignment, Element, Fill, Length, Task, Theme};

use crate::fluent::{
    DIALOG_WIDTH_PREVIEW, HEIGHT_PREVIEW_BODY, HEIGHT_PREVIEW_STATUS_BAR, SPACE_LG, SPACE_MD,
};
use crate::message::preview;
use crate::ui::dialog::Dialog;
use crate::ui::style::{error_text, secondary_button};

#[derive(Debug, Clone)]
pub struct PreviewState {
    pub open_path: EPath,
    pub name: String,
    pub loading: bool,
    pub file: Option<PreviewFile>,
    pub error: Option<String>,
    pub text: Option<text::Text>,
    pub image: Option<image::Image>,
    pub document: Option<document::Document>,
}

impl PreviewState {
    pub fn opening(open_path: EPath, name: String) -> Self {
        Self { open_path,
            name,
            loading: true,
            file: None,
            error: None,
            text: None,
            image: None,
            document: None,
        }
    }

    pub fn set_loaded_file(&mut self, file: PreviewFile) {
        self.error = None;
        self.text = text::Text::for_file(&file);
        self.image = image::Image::for_file(&file);
        self.document = document::Document::for_file(&file);
        self.file = Some(file);
    }
}

pub fn load_preview_task(path: EPath) -> Task<preview::Message> {
    Task::perform(
        async move { explorer_app::load_preview(&path) },
        preview::Message::Loaded,
    )
}

pub struct Preview;

impl Preview {
    pub fn new() -> Self {
        Self
    }

    pub fn view<'a>(&self, bundle: LanguageBundle, state: &'a PreviewState) -> Element<'a, preview::Message> {
        let open_label = bundle.tr(ids::PREVIEW_OPEN_EXTERNAL);
        let loading_label = bundle.tr(ids::PREVIEW_LOADING);

        let body: Element<'a, preview::Message> = if state.loading {
            container(text_widget(loading_label).size(14))
                .width(Fill)
                .height(Fill)
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center)
                .into()
        } else if let Some(error) = &state.error {
            preview_message(error.clone(), true)
        } else if let Some(file) = &state.file {
            body_for_file(bundle, file, state)
        } else {
            preview_message(bundle.tr(ids::PREVIEW_LOAD_FAILED), true)
        };

        let show_status_bar = !state.loading
            && state.error.is_none()
            && matches!(
                state.file.as_ref().map(|file| &file.kind),
                Some(PreviewKind::Text(_))
                    | Some(PreviewKind::Image(_))
                    | Some(PreviewKind::Word(_))
                    | Some(PreviewKind::Ppt(_))
                    | Some(PreviewKind::Pdf(_))
            );

        let body_height = if show_status_bar {
            HEIGHT_PREVIEW_BODY - HEIGHT_PREVIEW_STATUS_BAR - 1.0
        } else {
            HEIGHT_PREVIEW_BODY
        };

        let body = container(body)
            .padding(SPACE_LG)
            .width(Fill)
            .height(Length::Fixed(body_height));

        let footer = if show_status_bar {
            state.file.as_ref().and_then(|file| match &file.kind {
                PreviewKind::Text(text_preview) => state.text.as_ref().map(|text| {
                    text::status_bar(bundle, text, text_preview, file)
                }),
                PreviewKind::Image(image_preview) => state.image.as_ref().map(|image| {
                    image::status_bar(bundle, image, image_preview, file)
                }),
                PreviewKind::Word(_) | PreviewKind::Ppt(_) | PreviewKind::Pdf(_) => {
                    state.document.as_ref().map(|_| document::status_bar(bundle, file))
                }
                PreviewKind::Unsupported { .. } => None,
            })
        } else {
            None
        };

        let mut dialog = Dialog::new(state.name.clone(), preview::Message::Close)
            .width(DIALOG_WIDTH_PREVIEW)
            .body(body);

        if !state.loading {
            dialog = dialog.header_action(open_external_button(open_label));
        }

        if let Some(footer) = footer {
            dialog = dialog.footer(footer);
        }

        dialog.view()
    }
}

impl Default for Preview {
    fn default() -> Self {
        Self::new()
    }
}

fn open_external_button<'a>(label: String) -> Element<'a, preview::Message> {
    button(
        container(text_widget(label).size(13).line_height(iced::Pixels(18.0)))
            .height(Length::Fixed(32.0))
            .padding([0.0, SPACE_MD])
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center),
    )
    .on_press(preview::Message::OpenExternal)
    .height(Length::Fixed(32.0))
    .padding(0)
    .style(secondary_button)
    .into()
}

fn body_for_file<'a>(
    bundle: LanguageBundle,
    file: &'a PreviewFile,
    state: &'a PreviewState,
) -> Element<'a, preview::Message> {
    match &file.kind {
        PreviewKind::Text(_) => state
            .text
            .as_ref()
            .map(|text| text::view(bundle, text))
            .unwrap_or_else(|| preview_message(bundle.tr(ids::PREVIEW_LOAD_FAILED), true)),
        PreviewKind::Image(image_preview) => state
            .image
            .as_ref()
            .map(|image| image::view(image_preview, image.zoom))
            .unwrap_or_else(|| preview_message(bundle.tr(ids::PREVIEW_LOAD_FAILED), true)),
        PreviewKind::Word(_) | PreviewKind::Ppt(_) | PreviewKind::Pdf(_) => state
            .document
            .as_ref()
            .map(|document| document::view(bundle, document))
            .unwrap_or_else(|| preview_message(bundle.tr(ids::PREVIEW_LOAD_FAILED), true)),
        PreviewKind::Unsupported { extension } => unsupported_message(bundle, extension),
    }
}

pub(super) fn read_only_editor<'a>(
    content: &'a text_editor::Content,
    on_action: impl Fn(text_editor::Action) -> preview::Message + 'a,
) -> Element<'a, preview::Message> {
    scrollable(
        text_editor_widget(content)
            .on_action(on_action)
            .size(13)
            .line_height(iced::widget::text::LineHeight::Absolute(iced::Pixels(20.0)))
            .height(Length::Shrink)
            .style(document_editor_style),
    )
    .direction(Direction::Vertical(scrollable::Scrollbar::default()))
    .width(Fill)
    .height(Fill)
    .into()
}

pub(super) fn document_editor_style(
    theme: &Theme,
    status: text_editor::Status,
) -> text_editor::Style {
    let palette = theme.extended_palette();
    let mut style = text_editor::default(theme, status);
    style.background = iced::Background::Color(iced::Color::TRANSPARENT);
    style.border = iced::Border {
        width: 0.0,
        radius: 0.0.into(),
        color: iced::Color::TRANSPARENT,
    };
    style.value = palette.background.base.text;
    style
}

pub(super) fn preview_message(message: String, is_error: bool) -> Element<'static, preview::Message> {
    container(
        text_widget(message)
            .size(14)
            .style(if is_error {
                error_text
            } else {
                |theme: &Theme| {
                    let palette = theme.extended_palette();
                    iced::widget::text::Style {
                        color: Some(palette.background.base.text.scale_alpha(0.72)),
                    }
                }
            }),
    )
    .width(Fill)
    .height(Fill)
    .align_x(alignment::Horizontal::Center)
    .align_y(alignment::Vertical::Center)
    .into()
}

fn unsupported_message(
    bundle: LanguageBundle,
    extension: &Option<String>,
) -> Element<'static, preview::Message> {
    let label = if let Some(ext) = extension {
        let mut args = FluentArgs::new();
        args.set("extension", FluentValue::from(ext.clone()));
        bundle.tr_with(ids::PREVIEW_UNSUPPORTED_TYPE, args)
    } else {
        bundle.tr(ids::PREVIEW_UNSUPPORTED)
    };
    preview_message(label, false)
}

pub(super) fn preview_status_bar(theme: &Theme) -> iced::widget::container::Style {
    let palette = theme.extended_palette();
    iced::widget::container::Style {
        background: Some(iced::Background::Color(
            palette.background.strong.color.scale_alpha(0.16),
        )),
        ..Default::default()
    }
}

pub(super) fn status_muted_text(theme: &Theme) -> iced::widget::text::Style {
    let palette = theme.extended_palette();
    iced::widget::text::Style {
        color: Some(palette.background.base.text.scale_alpha(0.55)),
    }
}
