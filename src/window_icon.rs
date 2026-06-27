use fontdue::{Font, FontSettings};
use iced::window::icon::{self, Icon};
use lucide_icons::{Icon as LucideIcon, LUCIDE_FONT_BYTES};

const SIZE: u32 = 64;
const GLYPH_SCALE: f32 = 48.0;
const ICON_COLOR: [u8; 3] = [59, 130, 246];

pub fn app_icon() -> Icon {
    let rgba = render_lucide(LucideIcon::FolderOpen);
    icon::from_rgba(rgba, SIZE, SIZE).expect("valid window icon")
}

fn render_lucide(icon: LucideIcon) -> Vec<u8> {
    let font =
        Font::from_bytes(LUCIDE_FONT_BYTES, FontSettings::default()).expect("lucide font");
    let (metrics, bitmap) = font.rasterize(char::from(icon), GLYPH_SCALE);

    if metrics.width == 0 || metrics.height == 0 {
        return vec![0u8; (SIZE * SIZE * 4) as usize];
    }

    let mut rgba = vec![0u8; (SIZE * SIZE * 4) as usize];
    let offset_x = (SIZE.saturating_sub(metrics.width as u32)) / 2;
    let offset_y = (SIZE.saturating_sub(metrics.height as u32)) / 2;

    for y in 0..metrics.height {
        for x in 0..metrics.width {
            let alpha = bitmap[y * metrics.width + x];
            if alpha == 0 {
                continue;
            }

            let px = offset_x + x as u32;
            let py = offset_y + y as u32;
            if px >= SIZE || py >= SIZE {
                continue;
            }

            let index = ((py * SIZE + px) * 4) as usize;
            rgba[index] = ICON_COLOR[0];
            rgba[index + 1] = ICON_COLOR[1];
            rgba[index + 2] = ICON_COLOR[2];
            rgba[index + 3] = alpha;
        }
    }

    rgba
}
