use explorer_app::{ids, LanguageBundle, PreviewFile, TextEncoding, TextPreview};
use iced::widget::{container, pick_list, row, text, Space};
use iced::{alignment, Element, Fill, Length};

use crate::fluent::{
    FONT_SIZE_CAPTION, HEIGHT_PREVIEW_STATUS_BAR, PAGE_PADDING_H, SPACE_MD, SPACE_XS,
};
use crate::ui::style::{error_text, pick_list_style};
use crate::ui::preview::{preview_status_bar, status_muted_text};

use super::{Message, Text};

pub fn view(
    bundle: LanguageBundle,
    text_state: &Text,
    text_preview: &TextPreview,
    file: &PreviewFile,
) -> Element<'static, Message> {
    let size_label = bundle.format_size(file.size);

    let encoding_error = text_state.encoding_error.as_ref().map(|error| {
        let message = if error == "preview-decode-failed" {
            bundle.tr(ids::PREVIEW_DECODE_FAILED)
        } else {
            error.clone()
        };
        encoding_error_label(message)
    });

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

fn encoding_error_label(message: String) -> Element<'static, Message> {
    text(message)
        .size(FONT_SIZE_CAPTION)
        .style(error_text)
        .into()
}

fn encoding_controls(
    bundle: LanguageBundle,
    selected: TextEncoding,
    text_preview: &TextPreview,
) -> Element<'static, Message> {
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

    let picker = pick_list(options, current, |option| {
        Message::EncodingSelected(option.encoding)
    })
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

    let mut items: Vec<Element<'static, Message>> = vec![
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

impl std::fmt::Display for EncodingOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}
