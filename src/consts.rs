use embedded_graphics::pixelcolor::{Rgb565, RgbColor};

// DIMENSIONS
const WIDTH: u16 = 480;
const HEIGHT: u16 = 272;

// Graphics framebuffer
pub const FB_GRAPHICS_SIZE: usize = (WIDTH as usize) * (HEIGHT as usize);

pub const SEVEN_SEG_LEFT: u16 = 1;
pub const SEVEN_SEG_WIDTH: u16 = 194;
pub const SEVEN_SEG_HEIGHT: u16 = 56; // Dictated by rectange to fit seven segment font
pub const SEVEN_SEG_TOP: u16 = 15;
pub const SEVEN_SEG_VSPACE: u16 = 65;

pub const BUTTON_WIDTH: u16 = 51;
pub const BUTTON_HEIGHT: u16 = 49;
pub const DOUBLE_BUTTON_HEIGHT: u16 = 2 * BUTTON_HEIGHT + (KEY_Y_SPACING - BUTTON_HEIGHT);
pub const CORNER_RADIUS: u32 = 6;
pub const BUTTON_STROKE_WIDTH: u32 = 2;
pub const LIGHT_BLUE: Rgb565 = Rgb565::new(200, 220, 255);
pub const ORANGE: Rgb565 = Rgb565::new(255, 165, 0);
pub const BUTTON_STROKE_COLOR: Rgb565 = <Rgb565>::BLUE;
pub const BACKGROUND_COLOR: Rgb565 = Rgb565::new(10, 10, 10);
pub const BUTTON_FILL_COLOR: Rgb565 = <Rgb565>::WHITE;
pub const TEXT_COLOR: Rgb565 = <Rgb565>::BLACK;
pub const BUTTON_PUSH_COLOR: Rgb565 = <Rgb565>::BLACK;
pub const TEXT_PUSH_COLOR: Rgb565 = <Rgb565>::WHITE;
pub const MINUS_WIDTH: u16 = 12;

pub const DISPLAY_TEXT_COLOR: Rgb565 = <Rgb565>::YELLOW;
pub const DISPLAY_BACKGROUND_COLOR: Rgb565 = <Rgb565>::BLACK;
pub const DISPLAY_HIGHLIGHT_TEXT_COLOR: Rgb565 = <Rgb565>::CYAN;
pub const KEY_X_OFFSET: u16 = 257;
pub const KEY_X_SPACING: u16 = 57;
pub const KEY_Y_OFFSET: u16 = 2;
pub const KEY_Y_SPACING: u16 = 55; //270 / 5;

pub const MAXKEYS: usize = 30; // No vecs, so touchzones are stored in array
