use embedded_graphics::{
    image::ImageRaw,
    mono_font::{mapping::StrGlyphMapping, DecorationDimensions, MonoFont, MonoTextStyle},
    pixelcolor::{Rgb565, RgbColor},
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle, RoundedRectangle},
    text::Text,
};

use stm32f7xx_hal::ltdc::Layer;

#[allow(unused_imports)]
use panic_semihosting;

// use ft5336::Ft5336;
use crate::screen::Stm32F7DiscoDisplay;
use crate::ui;
use profont::PROFONT_24_POINT;

const SEVENT_SEGMENT_FONT: MonoFont = MonoFont {
    image: ImageRaw::new_binary(include_bytes!("assets/seven-segment-font.raw"), 224),
    glyph_mapping: &StrGlyphMapping::new("0123456789", 0),
    character_size: Size::new(22, 40),
    character_spacing: 4,
    baseline: 7,
    underline: DecorationDimensions::default_underline(40),
    strikethrough: DecorationDimensions::default_strikethrough(40),
};

use rtt_target::rprintln;
// crate screen::Stm32F7DiscoDisplay;

// DIMENSIONS
const WIDTH: u16 = 480;
const HEIGHT: u16 = 272;

// Graphics framebuffer
const FB_GRAPHICS_SIZE: usize = (WIDTH as usize) * (HEIGHT as usize);
pub static mut FB_LAYER1: [u16; FB_GRAPHICS_SIZE] = [0; FB_GRAPHICS_SIZE];

const SEVEN_SEG_LEFT: u16 = 1;
const SEVEN_SEG_WIDTH: u16 = 194;
const SEVEN_SEG_HEIGHT: u16 = 56; // Dictated by rectange to fit seven segment font
const SEVEN_SEG_TOP: u16 = 15;
const SEVEN_SEG_VSPACE: u16 = 65;

const BUTTON_WIDTH: u16 = 51;
const BUTTON_HEIGHT: u16 = 49;
const DOUBLE_BUTTON_HEIGHT: u16 = 2 * BUTTON_HEIGHT + (KEY_Y_SPACING - BUTTON_HEIGHT);
const TEXT_XOFFSET: i32 = 18;
const TEXT_YOFFSET: i32 = 31;
const CORNER_RADIUS: u32 = 6;
const BUTTON_STROKE_WIDTH: u32 = 2;
const LIGHT_BLUE: Rgb565 = Rgb565::new(165, 165, 255);
const ORANGE: Rgb565 = Rgb565::new(255, 165, 0);
const BUTTON_STROKE_COLOR: Rgb565 = <Rgb565>::BLUE;
const BUTTON_FILL_COLOR: Rgb565 = <Rgb565>::WHITE;
const BACKGROUND_COLOR: Rgb565 = Rgb565::new(10, 10, 10);
const TEXT_COLOR: Rgb565 = <Rgb565>::BLACK;
const DISPLAY_TEXT_COLOR: Rgb565 = <Rgb565>::YELLOW;
const DISPLAY_BACKGROUND_COLOR: Rgb565 = <Rgb565>::BLACK;
const KEY_X_OFFSET: u16 = 257;
const KEY_X_SPACING: u16 = 57;
const KEY_Y_OFFSET: u16 = 2;
const KEY_Y_SPACING: u16 = 55; //270 / 5;

const MAXKEYS: usize = 30; // No vecs, so touchzones are stored in array

#[derive(Copy, Clone, Debug)]
pub struct Button {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    text_x: u16,
    text_y: u16,
    active: bool,
    msg: ui::Messages,
    fill_color: Rgb565,
    text_color: Rgb565,
    text: Option<&'static str>,
}

impl Button {
    fn new(
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        text: Option<&'static str>,
        msg: ui::Messages,
    ) -> Button {
        let h = PROFONT_24_POINT.character_size.height;
        let w = PROFONT_24_POINT.character_size.width;
        return Button {
            x,
            y,
            width,
            height,
            text_x: x + (width - w as u16) / 2 + 1,
            text_y: y + (height + h as u16) / 2 - 3,
            active: false,
            msg,
            fill_color: BUTTON_FILL_COLOR, // Defaults, use change_colors to change
            text_color: TEXT_COLOR,
            text,
        };
    }

    /// Draw a styled button at an x, y location using the consts defined for width, height etc
    fn draw(&mut self, display: &mut Stm32F7DiscoDisplay<u16>) {
        let style = PrimitiveStyleBuilder::new()
            .stroke_width(BUTTON_STROKE_WIDTH)
            .stroke_color(BUTTON_STROKE_COLOR)
            .fill_color(self.fill_color)
            .build();

        // Converting to Rectangle, which supposedly uses fast code, doesn't
        // make a helpful difference. It is faster, but not fast enough for the ugliness.
        RoundedRectangle::with_equal_corners(
            Rectangle::new(
                Point::new(self.x as i32, self.y as i32),
                Size::new(self.width as u32, self.height as u32),
            ),
            Size::new(CORNER_RADIUS, CORNER_RADIUS),
        )
        .into_styled(style)
        .draw(display)
        .ok();

        let style = MonoTextStyle::new(&PROFONT_24_POINT, self.text_color);
        Text::new(
            self.text.unwrap(),
            Point::new(self.text_x.into(), self.text_y.into()),
            style,
        )
        .draw(display)
        .ok();
    }

    fn change_colors(&mut self, fill: Rgb565, text: Rgb565) {
        self.fill_color = fill;
        self.text_color = text;
    }

    // returns true if coords x and y fall within the edges of the button:
    fn inside(&mut self, x: u16, y: u16) -> bool {
        x >= self.x && x <= (self.x + self.width) && y >= self.y && y <= (self.y + self.height)
    }

    fn get_message(&mut self) -> ui::Messages {
        self.msg
    }
}

pub fn draw_background(display: &mut Stm32F7DiscoDisplay<u16>) {
    let c = BACKGROUND_COLOR;
    let background_color: u32 =
        c.b() as u32 & 0x1F | ((c.g() as u32 & 0x3F) << 5) | ((c.r() as u32 & 0x1F) << 11);

    unsafe {
        display
            .controller
            .draw_rectangle(Layer::L1, (0, 0), (480, 272), background_color);
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Buttons {
    buttons: [Button; MAXKEYS],
    counter: usize,
}

impl Buttons {
    pub fn new() -> Buttons {
        let buttons = [Button {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            text_x: 0,
            text_y: 0,
            active: false,
            msg: ui::Messages::None,
            fill_color: BUTTON_FILL_COLOR,
            text_color: Rgb565::BLACK,
            text: None,
        }; MAXKEYS];
        Buttons {
            buttons,
            counter: 0,
        }
    }
    pub fn add(&mut self, button: Button) {
        self.buttons[self.counter] = button;
        self.counter += 1;
        if self.counter >= MAXKEYS {
            panic!("Too many keys entered into TouchZones");
        };
    }

    fn make_keys(&mut self) {
        for i in 0..3 {
            for j in 0..3 {
                let index = 1 + i * 3 + j;
                let a = match index {
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
                let y = (KEY_Y_OFFSET + (3 - i) * KEY_Y_SPACING) as u16;
                let v = if a != "" { a.chars().last() } else { None };
                self.add(Button::new(
                    x as u16,
                    y as u16,
                    BUTTON_WIDTH as u16,
                    BUTTON_HEIGHT as u16,
                    Some(a),
                    ui::Messages::Key(index as u8),
                ));
            }
        }
        self.add(Button::new(
            KEY_X_OFFSET as u16,
            KEY_Y_OFFSET as u16 + 4 * KEY_Y_SPACING as u16,
            BUTTON_WIDTH as u16,
            BUTTON_HEIGHT as u16,
            Some("0"),
            ui::Messages::Key(0),
        ));
        let mut button = Button::new(
            KEY_X_OFFSET as u16 + 3 * KEY_X_SPACING as u16,
            KEY_Y_OFFSET as u16 + 3 * KEY_Y_SPACING as u16,
            BUTTON_WIDTH as u16,
            DOUBLE_BUTTON_HEIGHT as u16,
            Some(">"),
            ui::Messages::Enter,
        );
        button.change_colors(Rgb565::RED, Rgb565::BLACK);
        self.add(button);

        self.add(Button::new(
            KEY_X_OFFSET as u16 + 1 * KEY_X_SPACING as u16,
            KEY_Y_OFFSET as u16 + 4 * KEY_Y_SPACING as u16,
            BUTTON_WIDTH as u16,
            BUTTON_HEIGHT as u16,
            Some("."),
            ui::Messages::DecimalPoint,
        ));
        self.add(Button::new(
            KEY_X_OFFSET as u16 + 2 * KEY_X_SPACING as u16,
            KEY_Y_OFFSET as u16 + 4 * KEY_Y_SPACING as u16,
            BUTTON_WIDTH as u16,
            BUTTON_HEIGHT as u16,
            Some("±"),
            ui::Messages::PlusMinus,
        ));
        self.add(Button::new(
            KEY_X_OFFSET as u16 + 3 * KEY_X_SPACING as u16,
            KEY_Y_OFFSET as u16 + 2 * KEY_Y_SPACING as u16,
            BUTTON_WIDTH as u16,
            BUTTON_HEIGHT as u16,
            Some("C"),
            ui::Messages::Clear,
        ));
        self.add(Button::new(
            KEY_X_OFFSET as u16 + 3 * KEY_X_SPACING as u16,
            KEY_Y_OFFSET as u16 + 1 * KEY_Y_SPACING as u16,
            BUTTON_WIDTH as u16,
            BUTTON_HEIGHT as u16,
            Some("H"),
            ui::Messages::Halve,
        ));
        let mut button = Button::new(
            KEY_X_OFFSET as u16 + 3 * KEY_X_SPACING as u16,
            KEY_Y_OFFSET as u16 + 0 * KEY_Y_SPACING as u16,
            BUTTON_WIDTH as u16,
            BUTTON_HEIGHT as u16,
            Some("M"),
            ui::Messages::Halve,
        );
        button.change_colors(ORANGE, Rgb565::BLACK);
        self.add(button);
        let mut button = Button::new(
            SEVEN_SEG_WIDTH + 8,
            SEVEN_SEG_TOP + 0 * SEVEN_SEG_VSPACE + (SEVEN_SEG_HEIGHT - BUTTON_HEIGHT) / 2,
            (BUTTON_WIDTH - 1) as u16,
            BUTTON_HEIGHT as u16,
            Some("0"),
            ui::Messages::X0Button,
        );
        button.change_colors(LIGHT_BLUE, Rgb565::BLACK);
        self.add(button);
        let mut button = Button::new(
            SEVEN_SEG_WIDTH + 8,
            SEVEN_SEG_TOP + 1 * SEVEN_SEG_VSPACE + (SEVEN_SEG_HEIGHT - BUTTON_HEIGHT) / 2,
            (BUTTON_WIDTH - 1) as u16,
            BUTTON_HEIGHT as u16,
            Some("0"),
            ui::Messages::Y0Button,
        );
        button.change_colors(LIGHT_BLUE, Rgb565::BLACK);
        self.add(button);
        let mut button = Button::new(
            SEVEN_SEG_WIDTH + 8,
            SEVEN_SEG_TOP + 2 * SEVEN_SEG_VSPACE + (SEVEN_SEG_HEIGHT - BUTTON_HEIGHT) / 2,
            (BUTTON_WIDTH - 1) as u16,
            BUTTON_HEIGHT as u16,
            Some("0"),
            ui::Messages::Z0Button,
        );
        button.change_colors(LIGHT_BLUE, Rgb565::BLACK);
        self.add(button);
        let mut button = Button::new(
            KEY_X_OFFSET,
            1,
            BUTTON_WIDTH as u16,
            BUTTON_HEIGHT as u16,
            Some("X"),
            ui::Messages::XButton,
        );
        button.change_colors(LIGHT_BLUE, Rgb565::BLACK);
        self.add(button);
        let mut button = Button::new(
            KEY_X_OFFSET + 1 * KEY_X_SPACING as u16,
            1,
            BUTTON_WIDTH as u16,
            BUTTON_HEIGHT as u16,
            Some("Y"),
            ui::Messages::YButton,
        );
        button.change_colors(LIGHT_BLUE, Rgb565::BLACK);
        self.add(button);
        let mut button = Button::new(
            KEY_X_OFFSET + 2 * KEY_X_SPACING as u16,
            1,
            BUTTON_WIDTH as u16,
            BUTTON_HEIGHT as u16,
            Some("Z"),
            ui::Messages::ZButton,
        );
        button.change_colors(LIGHT_BLUE, Rgb565::BLACK);
        self.add(button);
    }

    pub fn draw(&mut self, display: &mut Stm32F7DiscoDisplay<u16>) {
        for i in 0..self.counter {
            let mut button = self.buttons[i];
            button.draw(display);
        }
    }

    pub fn locate(&mut self, x: u16, y: u16) -> Option<ui::Messages> {
        for mut button in self.buttons {
            if button.inside(x, y) {
                return Some(button.get_message());
            };
        }
        None
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SevenSegDisplay {
    x: u16,
    y: u16,
    is_metric: bool,
    width: u16,
    height: u16,
    text_x: u16,
    text_y: u16,
    active: bool,
    msg: ui::Messages,
    fill_color: Rgb565,
    text_color: Rgb565,
    text: Option<[char; 6]>,
    value: f32,
    negative: bool,
}

impl SevenSegDisplay {
    fn new(x: u16, y: u16, width: u16, height: u16, msg: ui::Messages) -> SevenSegDisplay {
        let h = PROFONT_24_POINT.character_size.height;
        let w = PROFONT_24_POINT.character_size.width;
        return SevenSegDisplay {
            x,
            y,
            is_metric: true,
            width,
            height,
            text_x: x + 7,
            text_y: y + 20, //height / 2 + (height - h as u16) / 2 + 1,
            active: false,
            msg,
            fill_color: DISPLAY_BACKGROUND_COLOR,
            text_color: DISPLAY_TEXT_COLOR,
            text: None,
            value: 0.0,
            negative: false,
        };
    }

    /// Create a vector of six digits and set the correct sign based on
    /// the incoming float value. Three digits left of decimal point, three
    /// after - generate 999.999 if the number goes out of range.
    fn set_value(&mut self, value: f32) {
        self.negative = value < 0.0;
        self.value = if value < 0.0 { 0.0 - value } else { value }; // abs is in standard
        let mut text: [char; 6] = [' '; 6];

        if self.value >= 1000.0 {
            self.text = Some(['9'; 6]);
        } else {
            let mut digits = (self.value * 1000.0 + 0.499) as u32;
            for i in 0..6 {
                text[5 - i] = match digits % 10 {
                    0 => '0',
                    1 => '1',
                    2 => '2',
                    3 => '3',
                    4 => '4',
                    5 => '5',
                    6 => '6',
                    7 => '7',
                    8 => '8',
                    9 => '9',
                    _ => ' ',
                };
                digits = digits / 10;
            }
            self.text = Some(text);
        }
    }

    fn draw(&mut self, display: &mut Stm32F7DiscoDisplay<u16>) {
        let style = PrimitiveStyleBuilder::new()
            .stroke_width(BUTTON_STROKE_WIDTH)
            .stroke_color(BUTTON_STROKE_COLOR)
            .fill_color(self.fill_color)
            .build();

        // Converting to Rectangle, which supposedly uses fast code, doesn't
        // make a helpful difference. It is faster, but not fast enough for the ugliness.
        RoundedRectangle::with_equal_corners(
            Rectangle::new(
                Point::new(self.x as i32, self.y as i32),
                Size::new(self.width as u32, self.height as u32),
            ),
            Size::new(CORNER_RADIUS, CORNER_RADIUS),
        )
        .into_styled(style)
        .draw(display)
        .ok();

        let c = DISPLAY_TEXT_COLOR;
        let background_color: u32 =
            c.b() as u32 & 0x1F | ((c.g() as u32 & 0x3F) << 5) | ((c.r() as u32 & 0x1F) << 11);

        // Font is 22x40

        // Optional minus sign:
        const MINUS_WIDTH: u16 = 12;
        if self.negative {
            unsafe {
                display.controller.draw_rectangle(
                    Layer::L1,
                    ((self.text_x) as usize, self.text_y as usize + 6),
                    (
                        (self.text_x + MINUS_WIDTH) as usize,
                        (self.text_y + 10) as usize,
                    ),
                    background_color,
                );
            }
        }
        let style = MonoTextStyle::new(&SEVENT_SEGMENT_FONT, self.text_color);
        let mut offset = MINUS_WIDTH + 4;
        let mut lead_zero = true;
        for (i, c) in self.text.unwrap().iter().enumerate() {
            if i == 3 {
                // Insert decimal place into view
                offset += 8;
                unsafe {
                    display.controller.draw_rectangle(
                        Layer::L1,
                        (
                            (self.text_x + 81 + MINUS_WIDTH + 4) as usize,
                            (self.text_y + 24) as usize,
                        ),
                        (
                            (self.text_x + 81 + MINUS_WIDTH + 8) as usize,
                            (self.text_y + 24 + 4) as usize,
                        ),
                        background_color,
                    );
                }
            };
            if !(lead_zero && i == 0 && *c == '0' || lead_zero && i == 1 && *c == '0') {
                lead_zero = false;
                let mut b = [0; 4];
                let txt = c.encode_utf8(&mut b);
                Text::new(
                    &txt,
                    Point::new(
                        (self.text_x + i as u16 * 27 + offset as u16) as i32,
                        (self.text_y - 5) as i32,
                    ),
                    style,
                )
                .draw(display)
                .ok();
            }
        }
    }

    fn change_colors(&mut self, fill: Rgb565, text: Rgb565) {
        self.fill_color = fill;
        self.text_color = text;
    }
}

// Locations etc for the three seven segment displays
#[derive(Copy, Clone, Debug)]
pub struct View {
    buttons: Buttons,
    x: SevenSegDisplay,
    y: SevenSegDisplay,
    z: SevenSegDisplay,
}

impl View {
    pub fn new() -> View {
        let mut x = SevenSegDisplay::new(
            SEVEN_SEG_LEFT,
            SEVEN_SEG_TOP + 0 * SEVEN_SEG_VSPACE,
            SEVEN_SEG_WIDTH,
            SEVEN_SEG_HEIGHT,
            ui::Messages::X(0),
        );
        x.set_value(-1000.);

        let mut y = SevenSegDisplay::new(
            SEVEN_SEG_LEFT,
            SEVEN_SEG_TOP + 1 * SEVEN_SEG_VSPACE,
            SEVEN_SEG_WIDTH,
            SEVEN_SEG_HEIGHT,
            ui::Messages::Y(100000),
        );
        y.set_value(100.0);

        let mut z = SevenSegDisplay::new(
            SEVEN_SEG_LEFT,
            SEVEN_SEG_TOP + 2 * SEVEN_SEG_VSPACE,
            SEVEN_SEG_WIDTH,
            SEVEN_SEG_HEIGHT,
            ui::Messages::Z(200000),
        );
        z.set_value(-200.14540);

        View {
            buttons: Buttons::new(),
            x,
            y,
            z,
        }
    }

    pub fn fill(&mut self) {
        self.buttons.make_keys();
    }

    pub fn update(&mut self, display: &mut Stm32F7DiscoDisplay<u16>) {
        draw_background(display);
        self.buttons.draw(display);
        self.x.draw(display);
        self.y.draw(display);
        self.z.draw(display);
    }

    pub fn coords_in_button(mut self, x: u16, y: u16) -> Option<ui::Messages> {
        self.buttons.locate(x, y)
    }

    pub fn process_message(mut self, msg: ui::Messages, display: &mut Stm32F7DiscoDisplay<u16>) {
        match msg {
            ui::Messages::X0Button => {
                self.x.set_value(10.0);
                self.x.draw(display);
            }
            ui::Messages::Y0Button => {
                self.y.set_value(0.0);
                self.y.draw(display);
            }
            ui::Messages::Z0Button => {
                self.z.set_value(-10.0);
                self.z.draw(display);
            }
            _ => (),
        }
    }
}
