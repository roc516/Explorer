use explorer_core::{ids, LanguageBundle, PreviewFile, PreviewKind};
use fluent::{FluentArgs, FluentValue};
use iced::widget::{container, row, text, Space};
use iced::widget::text_editor;
use iced::{alignment, Element, Fill, Length};

use crate::fluent::{
    FONT_SIZE_CAPTION, HEIGHT_PREVIEW_STATUS_BAR, PAGE_PADDING_H, SPACE_MD, SPACE_XS,
};
use crate::message::{preview, Message as AppMessage};

use super::{preview_message, preview_status_bar, read_only_editor, status_muted_text};

#[derive(Debug, Clone)]
pub struct Document {
    pub editor: Option<text_editor::Content>,
}

impl Document {
    pub fn for_file(file: &PreviewFile) -> Option<Self> {
        let content = match &file.kind {
            PreviewKind::Word(word) => Some(word.content.as_str()),
            PreviewKind::Ppt(ppt) => Some(ppt.content.as_str()),
            PreviewKind::Pdf(pdf) => Some(pdf.content.as_str()),
            _ => None,
        }?;

        Some(Self {
            editor: Some(text_editor::Content::with_text(content)),
        })
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
}

pub fn view<'a>(bundle: LanguageBundle, document: &'a Document) -> Element<'a, AppMessage> {
    let Some(content) = document.editor.as_ref() else {
        return preview_message(bundle.tr(ids::PREVIEW_LOADING), false);
    };

    read_only_editor(content, |action| AppMessage::Preview(preview::Message::DocumentEditor(action)))
}

pub fn status_bar(bundle: LanguageBundle, file: &PreviewFile) -> Element<'static, AppMessage> {
    let kind_label = match &file.kind {
        PreviewKind::Word(_) => bundle.tr(ids::PREVIEW_WORD_DOCUMENT),
        PreviewKind::Ppt(_) => bundle.tr(ids::PREVIEW_PPT_DOCUMENT),
        PreviewKind::Pdf(_) => bundle.tr(ids::PREVIEW_PDF_DOCUMENT),
        _ => String::new(),
    };

    let detail = match &file.kind {
        PreviewKind::Ppt(ppt) if ppt.slide_count > 0 => {
            let mut args = FluentArgs::new();
            args.set("count", FluentValue::from(ppt.slide_count as i32));
            Some(bundle.tr_with(ids::PREVIEW_PPT_SLIDES, args))
        }
        PreviewKind::Pdf(pdf) if pdf.page_count > 0 => {
            let mut args = FluentArgs::new();
            args.set("count", FluentValue::from(pdf.page_count as i32));
            Some(bundle.tr_with(ids::PREVIEW_PDF_PAGES, args))
        }
        _ => None,
    };

    let mut items: Vec<Element<'static, AppMessage>> = vec![
        text(kind_label)
            .size(FONT_SIZE_CAPTION)
            .style(status_muted_text)
            .into(),
    ];

    if let Some(detail_label) = detail {
        items.push(
            text(detail_label)
                .size(FONT_SIZE_CAPTION)
                .style(status_muted_text)
                .into(),
        );
    }

    items.push(Space::new().width(Fill).into());
    items.push(
        text(bundle.format_size(file.size))
            .size(FONT_SIZE_CAPTION)
            .style(status_muted_text)
            .into(),
    );

    container(
        row(items)
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
