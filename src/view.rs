use embedded_graphics::{
    mono_font::MonoTextStyle,
    pixelcolor::{Rgb565, RgbColor},
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle, RoundedRectangle},
    text::Text,
};

use stm32f7xx_hal::ltdc::{Layer, PixelFormat};

#[allow(unused_imports)]
use panic_semihosting;

// use ft5336::Ft5336;
use crate::screen::Stm32F7DiscoDisplay;
use profont::PROFONT_24_POINT;
use rtt_target::rprintln;
// crate screen::Stm32F7DiscoDisplay;

// DIMENSIONS
const WIDTH: u16 = 480;
const HEIGHT: u16 = 272;

// Graphics framebuffer
const FB_GRAPHICS_SIZE: usize = (WIDTH as usize) * (HEIGHT as usize);
pub static mut FB_LAYER1: [u16; FB_GRAPHICS_SIZE] = [0; FB_GRAPHICS_SIZE];

const BUTTON_WIDTH: u32 = 50;
const BUTTON_HEIGHT: u32 = 45;
const TEXT_XOFFSET: i32 = 18;
const TEXT_YOFFSET: i32 = 31;
const CORNER_RADIUS: u32 = 6;
const BUTTON_STROKE_WIDTH: u32 = 2;
const BUTTON_STROKE_COLOR: Rgb565 = <Rgb565>::BLUE;
const BUTTON_FILL_COLOR: Rgb565 = <Rgb565>::WHITE;
const BACKGROUND_COLOR: Rgb565 = Rgb565::new(10, 10, 10);
const TEXT_COLOR: Rgb565 = <Rgb565>::BLACK;
const KEY_X_OFFSET: i16 = 290;
const KEY_Y_OFFSET: i16 = 60;
const KEY_X_SPACING: i16 = 65;
const KEY_Y_SPACING: i16 = 55;

const MAXKEYS: usize = 30; // No vecs, so touchzones are stored in array
#[derive(Clone, Copy, Debug)]
pub struct TouchZone {
    xl: u16,
    xr: u16,
    yt: u16,
    yb: u16,
    value: Option<char>,
}
impl TouchZone {
    pub fn new(x: u16, y: u16, width: u16, height: u16, value: Option<char>) -> TouchZone {
        TouchZone {
            xl: x,
            yb: y,
            xr: x + width,
            yt: y + height,
            value,
        }
    }
}

/// Array holding coordinates for each key and character corresponding
/// to key. It also carries a number, n, showing how many keys are stored.
#[derive(Clone, Copy, Debug)]
pub struct TouchZones {
    tz: [TouchZone; MAXKEYS],
    n: usize,
}

impl TouchZones {
    pub fn new() -> TouchZones {
        TouchZones {
            tz: [TouchZone::new(0, 0, 0, 0, None); MAXKEYS],
            n: 0,
        }
    }
    pub fn add(&mut self, adding: TouchZone) {
        rprintln!("tz: {:?} - {}", self.tz, self.n);

        // let self.tz

        self.tz[self.n] = adding;
        self.n = self.n + 1;
        if self.n >= MAXKEYS {
            panic!("Too many keys entered into TouchZones");
        };
    }

    pub fn locate(&self, x: u16, y: u16) -> Option<char> {
        let value = self
            .tz
            .iter()
            .position(|tz| x >= tz.xl && x <= tz.xr && y >= tz.yb && y <= tz.yt);
        match value {
            Some(v) => self.tz[v].value,
            None => None,
        }
    }
}

/// Draw a styled button at an x, y location using the consts defined for width, height etc
fn button(x: i16, y: i16, caption: &str, display: &mut Stm32F7DiscoDisplay<u16>) {
    let style = PrimitiveStyleBuilder::new()
        .stroke_width(BUTTON_STROKE_WIDTH)
        .stroke_color(BUTTON_STROKE_COLOR)
        .fill_color(BUTTON_FILL_COLOR)
        .build();

    RoundedRectangle::with_equal_corners(
        Rectangle::new(
            Point::new(x as i32, y as i32),
            Size::new(BUTTON_WIDTH, BUTTON_HEIGHT),
        ),
        Size::new(CORNER_RADIUS, CORNER_RADIUS),
    )
    .into_styled(style)
    .draw(display)
    .ok();

    let style = MonoTextStyle::new(&PROFONT_24_POINT, TEXT_COLOR);

    Text::new(
        caption,
        Point::new(x as i32 + TEXT_XOFFSET, y as i32 + TEXT_YOFFSET),
        style,
    )
    .draw(display)
    .ok();
}

pub fn draw_keypad(display: &mut Stm32F7DiscoDisplay<u16>) -> TouchZones {
    let mut touch_zones = TouchZones::new();
    let c = BACKGROUND_COLOR;
    let background_color: u32 =
        c.b() as u32 & 0x1F | ((c.g() as u32 & 0x3F) << 5) | ((c.r() as u32 & 0x1F) << 11);

    unsafe {
        display
            .controller
            .draw_rectangle(Layer::L1, (0, 0), (480, 272), background_color);
    }

    for i in (0..3).rev() {
        for j in 0..3 {
            let a = match 1 + i * 3 + j {
                1 => "1",
                2 => "2",
                3 => "3",
                4 => "4",
                5 => "5",
                6 => "6",
                7 => "7",
                8 => "8",
                9 => "9",
                _ => "",
            };
            let x = (KEY_X_OFFSET + j * KEY_X_SPACING) as u16;
            let y = (KEY_Y_OFFSET + i * KEY_Y_SPACING) as u16;
            let v = if a != "" { a.chars().last() } else { None };
            button(x as i16, y as i16, a, display);
            touch_zones.add(TouchZone::new(
                x,
                y,
                BUTTON_WIDTH as u16,
                BUTTON_HEIGHT as u16,
                v,
            ));
        }
    }
    button(KEY_X_OFFSET, KEY_Y_OFFSET + 3 * KEY_Y_SPACING, "0", display);
    touch_zones.add(TouchZone::new(
        KEY_X_OFFSET as u16,
        KEY_Y_OFFSET as u16 + 3 * KEY_Y_SPACING as u16,
        BUTTON_WIDTH as u16,
        BUTTON_HEIGHT as u16,
        Some('0'),
    ));
    button(
        KEY_X_OFFSET + 1 * KEY_X_SPACING,
        KEY_Y_OFFSET + 3 * KEY_Y_SPACING,
        ".",
        display,
    );
    touch_zones.add(TouchZone::new(
        KEY_X_OFFSET as u16 + 1 * KEY_X_SPACING as u16,
        KEY_Y_OFFSET as u16 + 3 * KEY_Y_SPACING as u16,
        BUTTON_WIDTH as u16,
        BUTTON_HEIGHT as u16,
        Some('.'),
    ));
    button(
        KEY_X_OFFSET + 2 * KEY_X_SPACING,
        KEY_Y_OFFSET + 3 * KEY_Y_SPACING,
        "C",
        display,
    );
    touch_zones.add(TouchZone::new(
        KEY_X_OFFSET as u16 + 2 * KEY_X_SPACING as u16,
        KEY_Y_OFFSET as u16 + 3 * KEY_Y_SPACING as u16,
        BUTTON_WIDTH as u16,
        BUTTON_HEIGHT as u16,
        Some('C'),
    ));

    let style = PrimitiveStyleBuilder::new()
        .stroke_width(BUTTON_STROKE_WIDTH)
        .stroke_color(BUTTON_STROKE_COLOR)
        .fill_color(Rgb565::BLACK)
        .build();

    RoundedRectangle::with_equal_corners(
        Rectangle::new(
            Point::new(KEY_X_OFFSET as i32, KEY_Y_OFFSET as i32 - 55),
            Size::new(BUTTON_WIDTH + (2 * KEY_X_SPACING) as u32, BUTTON_HEIGHT),
        ),
        Size::new(CORNER_RADIUS, CORNER_RADIUS),
    )
    .into_styled(style)
    .draw(display)
    .ok();

    let style = MonoTextStyle::new(&PROFONT_24_POINT, Rgb565::YELLOW);

    Text::new(
        "300.89",
        Point::new(KEY_X_OFFSET as i32 + TEXT_XOFFSET, 5 + TEXT_YOFFSET),
        style,
    )
    .draw(display)
    .ok();

    touch_zones
}
