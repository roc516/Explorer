use std::fmt;

use explorer_core::{ids, LanguageBundle, PreviewFile, PreviewKind, TextEncoding, TextPreview};
use iced::widget::{container, pick_list, row, text, Space};
use iced::widget::text_editor;
use iced::{alignment, Element, Fill, Length};

use crate::fluent::{
    FONT_SIZE_CAPTION, HEIGHT_PREVIEW_STATUS_BAR, PAGE_PADDING_H, SPACE_MD, SPACE_XS,
};
use crate::message::{preview, Message as AppMessage};
use crate::widget::style::{error_text, pick_list_style};

use super::{preview_message, preview_status_bar, read_only_editor, status_muted_text};

#[derive(Debug, Clone)]
pub struct Text {
    pub encoding: TextEncoding,
    pub encoding_error: Option<String>,
    pub editor: Option<text_editor::Content>,
}

impl Text {
    pub fn for_file(file: &PreviewFile) -> Option<Self> {
        if !matches!(file.kind, PreviewKind::Text(_)) {
            return None;
        }

        let mut state = Self::new();
        state.sync_editor(Some(file));
        Some(state)
    }

    pub fn select_encoding(
        &mut self,
        file: &mut PreviewFile,
        encoding: TextEncoding,
        decode_failed: impl FnOnce() -> String,
        load_failed: impl FnOnce() -> String,
    ) {
        self.encoding = encoding;
        self.encoding_error = None;

        let PreviewKind::Text(text_preview) = &mut file.kind else {
            return;
        };

        if let Err(code) = text_preview.redecode(encoding) {
            self.encoding_error = Some(if code == "preview-decode-failed" {
                decode_failed()
            } else {
                load_failed()
            });
        } else {
            self.sync_editor(Some(file));
        }
    }

    pub fn handle_editor_action(&mut self, action: text_editor::Action) {
        let Some(content) = &mut self.editor else {
            return;
        };

        match action {
            text_editor::Action::Edit(_) => {}
            _ => content.perform(action),
        }
    }

    pub fn new() -> Self {
        Self {
            encoding: TextEncoding::Auto,
            encoding_error: None,
            editor: None,
        }
    }

    fn sync_editor(&mut self, file: Option<&PreviewFile>) {
        self.editor = file.and_then(|file| {
            if let PreviewKind::Text(text_preview) = &file.kind {
                Some(text_editor::Content::with_text(&text_preview.content))
            } else {
                None
            }
        });
    }
}

impl Default for Text {
    fn default() -> Self {
        Self::new()
    }
}

pub fn view<'a>(bundle: LanguageBundle, text: &'a Text) -> Element<'a, AppMessage> {
    let Some(content) = text.editor.as_ref() else {
        return preview_message(bundle.tr(ids::PREVIEW_LOADING), false);
    };

    read_only_editor(content, |action| AppMessage::Preview(preview::Message::TextEditor(action)))
}

pub fn status_bar(
    bundle: LanguageBundle,
    text_state: &Text,
    text_preview: &TextPreview,
    file: &PreviewFile,
) -> Element<'static, AppMessage> {
    let size_label = bundle.format_size(file.size);

    let encoding_error = text_state
        .encoding_error
        .as_ref()
        .map(|error| encoding_error_label(error.clone()));

    container(
        row![
            encoding_controls(bundle, text_state.encoding, text_preview),
            if let Some(error_label) = encoding_error {
                error_label
            } else {
                Space::new().width(0).into()
            },
            Space::new().width(Fill),
            text(size_label)
                .size(FONT_SIZE_CAPTION)
                .style(status_muted_text),
        ]
        .spacing(SPACE_MD)
        .align_y(alignment::Vertical::Center)
        .width(Fill),
    )
    .padding([SPACE_XS, PAGE_PADDING_H])
    .width(Fill)
    .height(Length::Fixed(HEIGHT_PREVIEW_STATUS_BAR))
    .style(preview_status_bar)
    .into()
}

fn encoding_error_label(message: String) -> Element<'static, AppMessage> {
    text(message)
        .size(FONT_SIZE_CAPTION)
        .style(error_text)
        .into()
}

fn encoding_controls(
    bundle: LanguageBundle,
    selected: TextEncoding,
    text_preview: &TextPreview,
) -> Element<'static, AppMessage> {
    let label = bundle.tr(ids::PREVIEW_ENCODING_LABEL);
    let options: Vec<EncodingOption> = TextEncoding::SELECTABLE
        .iter()
        .copied()
        .map(|encoding| EncodingOption {
            encoding,
            label: bundle.tr(encoding.message_id()),
        })
        .collect();
    let current = options
        .iter()
        .find(|option| option.encoding == selected)
        .cloned();

    let picker = pick_list(
        options,
        current,
        |option| AppMessage::Preview(preview::Message::EncodingSelected(option.encoding)),
    )
    .text_size(FONT_SIZE_CAPTION)
    .padding([2, 8])
    .width(Length::Shrink)
    .style(pick_list_style);

    let detected_hint = if selected == TextEncoding::Auto {
        let detected = bundle.tr(text_preview.resolved_encoding.message_id());
        Some(
            text(format!("· {detected}"))
                .size(FONT_SIZE_CAPTION)
                .style(status_muted_text)
                .into(),
        )
    } else {
        None
    };

    let mut items: Vec<Element<'static, AppMessage>> = vec![
        text(label)
            .size(FONT_SIZE_CAPTION)
            .style(status_muted_text)
            .into(),
        picker.into(),
    ];

    if let Some(hint) = detected_hint {
        items.push(hint);
    }

    row(items)
        .spacing(SPACE_MD)
        .align_y(alignment::Vertical::Center)
        .into()
}

#[derive(Clone, Eq)]
struct EncodingOption {
    encoding: TextEncoding,
    label: String,
}

impl PartialEq for EncodingOption {
    fn eq(&self, other: &Self) -> bool {
        self.encoding == other.encoding
    }
}

impl fmt::Display for EncodingOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.label)
    }
}
