use fontdue::{Font, FontSettings};

const FONT_PATH: &[u8] = include_bytes!("../../fonts/HackGen-Regular.ttf") as &[u8];
const FONT_SIZE: f32 = 17.0;

pub struct HackGenFont {
    size: f32,
    font: Font,
}

impl HackGenFont {
    pub fn new() -> Self {
        Self {
            size: FONT_PATH,
            font: Font::from_bytes(FONT_PATH, FontSettings::default()).unwrap(),
        }
    }

    // draw_string メソッドを実装する？
}
