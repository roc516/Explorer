use explorer_app::{ids, HexPreview, LanguageBundle, PreviewFile};
use iced::widget::{container, row, text, Space};
use iced::{alignment, Element, Fill, Font, Length};

use crate::fluent::{
    FONT_SIZE_CAPTION, HEIGHT_PREVIEW_STATUS_BAR, PAGE_PADDING_H, SPACE_MD, SPACE_XS,
};
use crate::message::preview;
use crate::ui::preview::{preview_status_bar, status_muted_text};

use super::{ascii_char, Hex};

pub fn view(
    bundle: LanguageBundle,
    preview: &HexPreview,
    state: &Hex,
    file: &PreviewFile,
) -> Element<'static, preview::Message> {
    let mut items: Vec<Element<'static, preview::Message>> = vec![
        text(bundle.tr(ids::PREVIEW_HEX))
            .size(FONT_SIZE_CAPTION)
            .style(status_muted_text)
            .into(),
    ];

    if let Some(index) = state.selected {
        if let Some(byte) = preview.bytes.get(index).copied() {
            let ascii = match byte {
                b' '..=b'~' => format!("'{}'", ascii_char(byte)),
                _ => ".".to_string(),
            };
            items.push(
                text(format!("0x{index:08X}"))
                    .size(FONT_SIZE_CAPTION)
                    .font(Font::MONOSPACE)
                    .style(status_muted_text)
                    .into(),
            );
            items.push(
                text(format!("0x{byte:02X} ({byte}) {ascii}"))
                    .size(FONT_SIZE_CAPTION)
                    .font(Font::MONOSPACE)
                    .style(status_muted_text)
                    .into(),
            );
        }
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
