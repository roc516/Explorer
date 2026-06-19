use iced::window::icon::{self, Icon};

const SIZE: u32 = 32;

pub fn app_icon() -> Icon {
    icon::from_rgba(build_rgba(), SIZE, SIZE).expect("valid window icon")
}

fn build_rgba() -> Vec<u8> {
    let mut rgba = vec![0u8; (SIZE * SIZE * 4) as usize];

    for y in 0..SIZE {
        for x in 0..SIZE {
            let color = pixel_color(x, y);
            let index = ((y * SIZE + x) * 4) as usize;
            rgba[index] = color[0];
            rgba[index + 1] = color[1];
            rgba[index + 2] = color[2];
            rgba[index + 3] = color[3];
        }
    }

    rgba
}

fn pixel_color(x: u32, y: u32) -> [u8; 4] {
    let tab = (8..=18).contains(&x) && (8..=11).contains(&y);
    let body = (5..=26).contains(&x) && (11..=25).contains(&y);
    let flap = (5..=18).contains(&x) && (11..=14).contains(&y);

    if tab {
        return [96, 165, 250, 255];
    }

    if body {
        let highlight = flap || (x <= 7 && y <= 16);
        if highlight {
            return [147, 197, 253, 255];
        }
        return [59, 130, 246, 255];
    }

    [0, 0, 0, 0]
}
