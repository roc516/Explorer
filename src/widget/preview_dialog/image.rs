use explorer_core::{ids, ImagePreview, LanguageBundle, PreviewFile, PreviewKind};
use iced::mouse;
use iced::widget::{button, container, image, mouse_area, row, scrollable, text, Space};
use iced::widget::scrollable::Direction;
use iced::{alignment, ContentFit, Element, Fill, Length};
use lucide_icons::Icon;

use crate::fluent::{
    DIALOG_WIDTH_PREVIEW, FONT_SIZE_CAPTION, HEIGHT_PREVIEW_BODY, HEIGHT_PREVIEW_STATUS_BAR,
    PAGE_PADDING_H, SPACE_LG, SPACE_MD, SPACE_SM, SPACE_XS,
};
use crate::message::preview;
use crate::widget::lucide_icon;
use crate::widget::style::{icon_button, secondary_button};

use super::{preview_status_bar, status_muted_text};

const ZOOM_MIN: f32 = 0.1;
const ZOOM_MAX: f32 = 8.0;
const ZOOM_STEP: f32 = 1.25;
const WHEEL_STEP: f32 = 1.1;
const ZOOM_BUTTON_SIZE: f32 = 24.0;
const ZOOM_ICON_SIZE: f32 = 14.0;

#[derive(Debug, Clone)]
pub struct Image {
    pub zoom: f32,
    pub fit_zoom: f32,
}

impl Image {
    pub fn for_file(file: &PreviewFile) -> Option<Self> {
        let PreviewKind::Image(image) = &file.kind else {
            return None;
        };

        let mut state = Self::new();
        let fit = fit_zoom(image);
        state.fit_zoom = fit;
        state.zoom = fit;
        Some(state)
    }

    pub fn wheel_zoom(&mut self, factor: f32) {
        if (factor - 1.0).abs() > f32::EPSILON {
            self.zoom_wheel(factor);
        }
    }

    pub fn new() -> Self {
        Self {
            zoom: 1.0,
            fit_zoom: 1.0,
        }
    }

    pub fn zoom_in(&mut self) {
        self.apply_factor(ZOOM_STEP);
    }

    pub fn zoom_out(&mut self) {
        self.apply_factor(1.0 / ZOOM_STEP);
    }

    pub fn zoom_wheel(&mut self, factor: f32) {
        self.apply_factor(factor);
    }

    pub fn reset(&mut self) {
        self.zoom = self.fit_zoom;
    }

    fn apply_factor(&mut self, factor: f32) {
        self.zoom = (self.zoom * factor).clamp(ZOOM_MIN, ZOOM_MAX);
    }
}

impl Default for Image {
    fn default() -> Self {
        Self::new()
    }
}

pub fn view(preview: &ImagePreview, zoom: f32) -> Element<'static, preview::Message> {
    let display_w = (preview.width as f32 * zoom).max(1.0);
    let display_h = (preview.height as f32 * zoom).max(1.0);

    let picture = mouse_area(
        image(image::Handle::from_bytes(preview.bytes.clone()))
            .width(Length::Fixed(display_w))
            .height(Length::Fixed(display_h))
            .content_fit(ContentFit::Fill),
    )
    .on_scroll(|delta| {
        preview::Message::ImageWheelZoom(scroll_delta_to_zoom_factor(delta))
    });

    let needs_scroll = display_w > viewport_width() || display_h > viewport_height();

    if needs_scroll {
        scrollable(
            container(picture)
                .width(Length::Fixed(display_w))
                .height(Length::Fixed(display_h)),
        )
        .direction(Direction::Both {
            vertical: Default::default(),
            horizontal: Default::default(),
        })
        .width(Fill)
        .height(Fill)
        .into()
    } else {
        container(picture)
            .width(Fill)
            .height(Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
            .into()
    }
}

pub fn status_bar(
    bundle: LanguageBundle,
    image_state: &Image,
    image: &ImagePreview,
    file: &PreviewFile,
) -> Element<'static, preview::Message> {
    let zoom_label = format_zoom_percent(image_state.zoom, image_state.fit_zoom);
    let dimensions = format!("{} × {}", image.width, image.height);
    let size_label = bundle.format_size(file.size);
    let fit_label = bundle.tr(ids::PREVIEW_ZOOM_FIT);

    container(
        row![
            zoom_button(Icon::ZoomOut, preview::Message::ImageZoomOut),
            text(zoom_label)
                .size(FONT_SIZE_CAPTION)
                .style(status_muted_text),
            zoom_button(Icon::ZoomIn, preview::Message::ImageZoomIn),
            button(
                container(text(fit_label).size(FONT_SIZE_CAPTION).line_height(iced::Pixels(16.0)))
                    .height(Length::Fixed(ZOOM_BUTTON_SIZE))
                    .padding([0.0, SPACE_SM])
                    .align_x(alignment::Horizontal::Center)
                    .align_y(alignment::Vertical::Center),
            )
            .on_press(preview::Message::ImageZoomReset)
            .height(Length::Fixed(ZOOM_BUTTON_SIZE))
            .padding(0)
            .style(secondary_button),
            text(dimensions)
                .size(FONT_SIZE_CAPTION)
                .style(status_muted_text),
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

fn fit_zoom(image: &ImagePreview) -> f32 {
    let avail_w = viewport_width();
    let avail_h = viewport_height();
    let scale_w = avail_w / image.width.max(1) as f32;
    let scale_h = avail_h / image.height.max(1) as f32;
    scale_w.min(scale_h).min(1.0)
}

fn viewport_width() -> f32 {
    DIALOG_WIDTH_PREVIEW - 2.0 * SPACE_LG
}

fn viewport_height() -> f32 {
    HEIGHT_PREVIEW_BODY - HEIGHT_PREVIEW_STATUS_BAR - 1.0 - 2.0 * SPACE_LG
}

fn format_zoom_percent(zoom: f32, fit_zoom: f32) -> String {
    let relative = if fit_zoom > f32::EPSILON {
        zoom / fit_zoom
    } else {
        zoom
    };
    format!("{:.0}%", relative * 100.0)
}

fn scroll_delta_to_zoom_factor(delta: mouse::ScrollDelta) -> f32 {
    let y = match delta {
        mouse::ScrollDelta::Lines { y, .. } => y,
        mouse::ScrollDelta::Pixels { y, .. } => y / 60.0,
    };

    if y > 0.0 {
        WHEEL_STEP
    } else if y < 0.0 {
        1.0 / WHEEL_STEP
    } else {
        1.0
    }
}

fn zoom_button(icon: Icon, message: preview::Message) -> Element<'static, preview::Message> {
    button(
        container(lucide_icon::icon_muted::<preview::Message>(icon, ZOOM_ICON_SIZE, 0.72))
            .width(Length::Fixed(ZOOM_BUTTON_SIZE))
            .height(Length::Fixed(ZOOM_BUTTON_SIZE))
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center),
    )
    .on_press(message)
    .width(Length::Fixed(ZOOM_BUTTON_SIZE))
    .height(Length::Fixed(ZOOM_BUTTON_SIZE))
    .padding(0)
    .style(icon_button)
    .into()
}
