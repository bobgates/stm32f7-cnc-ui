use embedded_graphics::{
    image::ImageRaw,
    mono_font::{mapping::StrGlyphMapping, DecorationDimensions, MonoFont, MonoTextStyle},
    pixelcolor::{Rgb565, RgbColor},
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle, RoundedRectangle},
    text::Text,
};
use rtt_target::rprintln;

use stm32f7xx_hal::ltdc::Layer;

use crate::consts::*;
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

// #[derive(Copy, Clone, Debug)]
// enum DisplayState {
//     // Waiting,
//     FirstKey,
//     NumberEntry,
//     EndsWell,
//     EndsBadly,
// }

#[derive(Copy, Clone, Debug)]
pub struct SevenSegDisplay {
    x: u16,
    y: u16,
    is_metric: bool,
    width: u16,
    height: u16,
    text_x: u16,
    text_y: u16,
    highlight: bool,
    fill_color: Rgb565,
    text_clr: Rgb565,
    highlight_text_color: Rgb565,
    text: Option<[char; 6]>,
    value: f32,
    backup_value: f32,
    decimal_digits: Option<u8>, // None when before decimal, Some(0) when decimal is entered
    // then Some(n) holding the count of decimals
    negative: bool,
}

impl SevenSegDisplay {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> SevenSegDisplay {
        let _h = PROFONT_24_POINT.character_size.height;
        let _w = PROFONT_24_POINT.character_size.width;
        return SevenSegDisplay {
            x,
            y,
            width,
            height,
            text_x: x + 7,
            text_y: y + 20, //height / 2 + (height - h as u16) / 2 + 1,
            is_metric: true,
            highlight: false,
            fill_color: DISPLAY_BACKGROUND_COLOR,
            text_clr: DISPLAY_TEXT_COLOR,
            highlight_text_color: DISPLAY_HIGHLIGHT_TEXT_COLOR,
            text: None,
            value: 0.0,
            backup_value: 0.0,
            negative: false,
            decimal_digits: None,
        };
    }

    pub fn text_color(&self) -> Rgb565 {
        if self.highlight {
            self.highlight_text_color
        } else {
            self.text_clr
        }
    }

    /// Create a vector of six digits and set the correct sign based on
    /// the incoming float value. Three digits left of decimal point, three
    /// after - generate 999.999 if the number goes out of range.
    pub fn set_value(&mut self, value: f32) {
        // rprintln!("value coming in: {:.3}", value);
        self.value = value;
        self.negative = value < 0.0;
        let value = if value < 0.0 { 0.0 - value } else { value }; // can't use abs - it is in standard

        // rprintln!("value after abs {:.3}", value);

        let mut text: [char; 6] = [' '; 6];

        if value >= 1000.0 {
            self.text = Some(['9'; 6]);
        } else {
            let mut digits = (value * 1000.0 + 0.499) as u32;
            // rprintln!("digits: {}", digits);
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
            // rprintln!("text is {:?}", self.text.unwrap());
        }
    }

    pub fn get_value(&mut self) -> f32 {
        self.value
    }

    fn draw_background(&mut self, display: &mut Stm32F7DiscoDisplay<u16>) {
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
    }

    /// Takes number of digits to get place of minus. This can 1-3. Numbers
    /// larger than 3 are treated as 3: minus in LH position
    fn draw_minus(self, n_digits: u16, display: &mut Stm32F7DiscoDisplay<u16>) {
        unsafe {
            let c = self.text_color();
            let background_color: u32 =
                c.b() as u32 & 0x1F | ((c.g() as u32 & 0x3F) << 5) | ((c.r() as u32 & 0x1F) << 11);

            let x_pos = self.text_x + (3 as u16 - n_digits) * 27;

            display.controller.draw_rectangle(
                Layer::L1,
                ((x_pos) as usize, self.text_y as usize + 6),
                ((x_pos + MINUS_WIDTH) as usize, (self.text_y + 10) as usize),
                background_color,
            );
        }
    }

    pub fn set_highlight_text(&mut self) {
        self.highlight = true;
    }

    pub fn clear_highlight_text(&mut self) {
        self.highlight = false;
    }

    pub fn draw(&mut self, display: &mut Stm32F7DiscoDisplay<u16>) {
        self.draw_background(display);

        let c = self.text_color();
        let text_color: u32 =
            c.b() as u32 & 0x1F | ((c.g() as u32 & 0x3F) << 5) | ((c.r() as u32 & 0x1F) << 11);

        // Font is 22x40

        // Optional minus sign:

        if self.value < 0.0 {
            let mut minus_digits = 3;
            if self.value <= -10.0 && self.value > -100.0 {
                minus_digits = 2;
            }
            if self.value <= -1.0 && self.value > -10.0 {
                minus_digits = 1;
            }

            // rprintln!("self.value = {}", self.value);
            // rprintln!("No of digits for minus is {}", minus_digits);
            self.draw_minus(minus_digits, display);
        }

        let style = MonoTextStyle::new(&SEVENT_SEGMENT_FONT, self.text_color());
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
                        text_color,
                    );
                }
            };
            // Don't draw leading zeros - they'll remain blank
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

    pub fn change_colors(&mut self, fill: Rgb565, text: Rgb565) {
        self.fill_color = fill;
        self.text_clr = text;
    }

    pub fn start(&mut self, display: &mut Stm32F7DiscoDisplay<u16>) {
        self.backup_value = self.value;
        self.set_highlight_text();
        self.decimal_digits = None;
        self.set_value(0.0);
        self.draw(display);
    }

    pub fn plus_minus(&mut self, display: &mut Stm32F7DiscoDisplay<u16>) {
        self.clear_highlight_text();
        self.set_value(0.0 - self.value);
        self.draw(display);
    }

    // If the half button is followed by an axis, it halves the
    // contents of that axis
    pub fn half(&mut self, display: &mut Stm32F7DiscoDisplay<u16>) {
        self.clear_highlight_text();
        self.set_value(self.value / 2.0);
        self.draw(display);
    }

    pub fn input(
        &mut self,
        key: ui::Ids,
        display: &mut Stm32F7DiscoDisplay<u16>,
    ) -> Option<Result<f32, u8>> {
        // rprintln!("Display - number entry: {:?}", key);
        match key {
            ui::Ids::Key(key) => {
                match self.decimal_digits {
                    None => self.value = self.value * 10.0 + key as f32,
                    Some(d) => match d {
                        0 => {
                            self.value = self.value + key as f32 / 10.0;
                            self.decimal_digits = Some(1);
                        }
                        1 => {
                            self.value = self.value + key as f32 / 100.0;
                            self.decimal_digits = Some(2);
                        }
                        2 => {
                            self.value = self.value + key as f32 / 1000.0;
                            self.decimal_digits = Some(3);
                        }
                        _ => {}
                    },
                }
                self.set_value(self.value);
                self.draw(display);
                None
            }
            ui::Ids::DecimalPoint => {
                if let Some(_d) = self.decimal_digits {
                } else {
                    self.decimal_digits = Some(0);
                }
                None
            }
            ui::Ids::Clear => {
                // rprintln!("Clear - ends badly");
                self.set_value(self.backup_value);
                self.clear_highlight_text();
                self.draw(display);
                Some(Err(0xFF))
            }
            ui::Ids::PlusMinus => {
                // rprintln!("+-");
                self.set_value(-self.value);
                self.draw(display);
                None
            }
            ui::Ids::Half => {
                // rprintln!("1/2");
                self.set_value(self.value / 2.0);
                self.draw(display);
                None
            }
            ui::Ids::Enter => {
                // rprintln!("Enter");
                // self.value = self.entry_value;
                self.clear_highlight_text();
                self.draw(display);
                // rprintln!("Display - ends well");
                Some(Ok(self.value))
            }
            _ => {
                // rprintln!("Display - ends badly");
                self.set_value(self.backup_value);
                self.clear_highlight_text();
                self.draw(display);
                Some(Err(0xFE))
            }
        }
    }
}
