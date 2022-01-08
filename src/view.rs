use embedded_graphics::{
    mono_font::MonoTextStyle,
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
use rtt_target::rprintln;
// crate screen::Stm32F7DiscoDisplay;

// DIMENSIONS
const WIDTH: u16 = 480;
const HEIGHT: u16 = 272;

// Graphics framebuffer
const FB_GRAPHICS_SIZE: usize = (WIDTH as usize) * (HEIGHT as usize);
pub static mut FB_LAYER1: [u16; FB_GRAPHICS_SIZE] = [0; FB_GRAPHICS_SIZE];

const BUTTON_WIDTH: u16 = 50;
const SEVEN_SEG_WIDTH: u16 = 200;
const SEVEN_SEG_HEIGHT: u16 = 60;

const BUTTON_HEIGHT: u16 = 45;
const DOUBLE_BUTTON_HEIGHT: u16 = 100;
const TEXT_XOFFSET: i32 = 18;
const TEXT_YOFFSET: i32 = 31;
const CORNER_RADIUS: u32 = 6;
const BUTTON_STROKE_WIDTH: u32 = 2;
const BUTTON_STROKE_COLOR: Rgb565 = <Rgb565>::BLUE;
const BUTTON_FILL_COLOR: Rgb565 = <Rgb565>::WHITE;
const BACKGROUND_COLOR: Rgb565 = Rgb565::new(10, 10, 10);
const TEXT_COLOR: Rgb565 = <Rgb565>::BLACK;
const DISPLAY_TEXT_COLOR: Rgb565 = <Rgb565>::YELLOW;
const DISPLAY_BACKGROUND_COLOR: Rgb565 = <Rgb565>::BLACK;
const KEY_X_OFFSET: u16 = 225;
const KEY_Y_OFFSET: u16 = 60;
const KEY_X_SPACING: u16 = 65;
const KEY_Y_SPACING: u16 = 55;

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
            text_y: y + (height + h as u16) / 2 - 3, //height / 2 + (height - h as u16) / 2 + 1,
            active: false,
            msg,
            fill_color: BUTTON_FILL_COLOR,
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
        for i in (0..3) {
            for j in (0..3) {
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
                let y = (KEY_Y_OFFSET + (2 - i) * KEY_Y_SPACING) as u16;
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
            KEY_Y_OFFSET as u16 + 3 * KEY_Y_SPACING as u16,
            BUTTON_WIDTH as u16,
            BUTTON_HEIGHT as u16,
            Some("0"),
            ui::Messages::Key(0),
        ));
        let mut button = Button::new(
            KEY_X_OFFSET as u16 + 3 * KEY_X_SPACING as u16,
            KEY_Y_OFFSET as u16 + 2 * KEY_Y_SPACING as u16,
            BUTTON_WIDTH as u16,
            DOUBLE_BUTTON_HEIGHT as u16,
            Some(">"),
            ui::Messages::Enter,
        );
        button.change_colors(Rgb565::RED, Rgb565::BLACK);
        self.add(button);

        self.add(Button::new(
            KEY_X_OFFSET as u16 + 1 * KEY_X_SPACING as u16,
            KEY_Y_OFFSET as u16 + 3 * KEY_Y_SPACING as u16,
            BUTTON_WIDTH as u16,
            BUTTON_HEIGHT as u16,
            Some("."),
            ui::Messages::DecimalPoint,
        ));
        self.add(Button::new(
            KEY_X_OFFSET as u16 + 2 * KEY_X_SPACING as u16,
            KEY_Y_OFFSET as u16 + 3 * KEY_Y_SPACING as u16,
            BUTTON_WIDTH as u16,
            BUTTON_HEIGHT as u16,
            Some("±"),
            ui::Messages::PlusMinus,
        ));
        self.add(Button::new(
            KEY_X_OFFSET as u16 + 3 * KEY_X_SPACING as u16,
            KEY_Y_OFFSET as u16 + 1 * KEY_Y_SPACING as u16,
            BUTTON_WIDTH as u16,
            BUTTON_HEIGHT as u16,
            Some("C"),
            ui::Messages::Clear,
        ));
        self.add(Button::new(
            KEY_X_OFFSET as u16 + 3 * KEY_X_SPACING as u16,
            KEY_Y_OFFSET as u16 + 0 * KEY_Y_SPACING as u16,
            BUTTON_WIDTH as u16,
            BUTTON_HEIGHT as u16,
            Some("½"),
            ui::Messages::Halve,
        ));
    }

    pub fn draw(&mut self, display: &mut Stm32F7DiscoDisplay<u16>) {
        for i in 0..self.counter {
            rprintln!("i: {}", i);
            let mut button = self.buttons[i];

            button.draw(display);
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Display {
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

impl Display {
    fn new(
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        text: Option<&'static str>,
        msg: ui::Messages,
    ) -> Display {
        let h = PROFONT_24_POINT.character_size.height;
        let w = PROFONT_24_POINT.character_size.width;
        return Display {
            x,
            y,
            width,
            height,
            text_x: x + 20,
            text_y: y + height - 5, //height / 2 + (height - h as u16) / 2 + 1,
            active: false,
            msg,
            fill_color: DISPLAY_BACKGROUND_COLOR,
            text_color: DISPLAY_TEXT_COLOR,
            text: Some("000.000"),
        };
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
}

#[derive(Copy, Clone, Debug)]
pub struct View {
    buttons: Buttons,
    x: Display,
    y: Display,
    z: Display,
    working: Display,
    // touch_zones: TouchZones,
}

impl View {
    pub fn new() -> View {
        rprintln!("Going to define view");
        let x = Display::new(
            5,
            30,
            SEVEN_SEG_WIDTH,
            SEVEN_SEG_HEIGHT,
            Some("000.000"),
            ui::Messages::X(0),
        );
        let y = Display::new(
            5,
            100,
            SEVEN_SEG_WIDTH,
            SEVEN_SEG_HEIGHT,
            Some("100.000"),
            ui::Messages::X(100000),
        );
        let z = Display::new(
            5,
            170,
            SEVEN_SEG_WIDTH,
            SEVEN_SEG_HEIGHT,
            Some("200.000"),
            ui::Messages::X(200000),
        );
        let working = Display::new(
            KEY_X_OFFSET,
            5,
            SEVEN_SEG_WIDTH,
            45,
            Some("200.000"),
            ui::Messages::X(200000),
        );

        View {
            buttons: Buttons::new(),
            x,
            y,
            z,
            working, // touch_zones: TouchZones::new(),
        }
    }

    pub fn fill(&mut self) {
        self.buttons.make_keys();
    }

    pub fn update(&mut self, display: &mut Stm32F7DiscoDisplay<u16>) {
        rprintln!("Before background draw");

        draw_background(display);
        rprintln!("Before buttons");
        // let buttons/*, touch_zones)*/ = make_keys();
        self.buttons.draw(display);
        self.x.draw(display);
        self.y.draw(display);
        self.z.draw(display);
        self.working.draw(display);

        /*
                draw_background
                for i in buttons
                    draw_button
                for i in displays
                    draw_displays
                place cursor



                draw_keypad(display);

        */
    }
}
