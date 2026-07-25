use iced::Theme;

pub fn error_text(theme: &Theme) -> iced::widget::text::Style {
    let palette = theme.extended_palette();
    iced::widget::text::Style {
        color: Some(palette.danger.strong.color),
    }
}
