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

use crate::consts::*;
use crate::display::SevenSegDisplay;
use crate::screen::Stm32F7DiscoDisplay;
use crate::ui;
use profont::PROFONT_24_POINT;

pub static mut FB_LAYER1: [u16; FB_GRAPHICS_SIZE] = [0; FB_GRAPHICS_SIZE];

use rtt_target::rprintln;

#[derive(Copy, Clone, Debug)]
pub struct Button {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    text_x: u16,
    text_y: u16,
    active: bool,
    id: ui::Ids,
    fill_color: Rgb565,
    text_color: Rgb565,
    push_fill: Rgb565,
    push_text: Rgb565,
    text: Option<&'static str>,
}

impl Button {
    fn new(
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        text: Option<&'static str>,
        id: ui::Ids,
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
            id,
            fill_color: BUTTON_FILL_COLOR, // Defaults, use change_colors to change
            text_color: TEXT_COLOR,
            push_fill: BUTTON_PUSH_COLOR,
            push_text: TEXT_PUSH_COLOR,
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

    fn change_text_position(&mut self, text_x: u16, text_y: u16) {
        self.text_x = text_x;
        self.text_y = text_y;
    }

    // returns true if coords x and y fall within the edges of the button:
    fn inside(&mut self, x: u16, y: u16) -> bool {
        x >= self.x && x <= (self.x + self.width) && y >= self.y && y <= (self.y + self.height)
    }

    fn get_id(&mut self) -> ui::Ids {
        self.id
    }

    pub fn activate(&mut self, display: &mut Stm32F7DiscoDisplay<u16>) {
        self.active = true;
        self.change_colors(self.push_fill, self.push_text);
        self.draw(display);
    }

    pub fn deactivate(&mut self, display: &mut Stm32F7DiscoDisplay<u16>) {
        self.active = false;
        self.change_colors(self.fill_color, self.text_color);
        self.draw(display);
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
            id: ui::Ids::Empty,
            fill_color: BUTTON_FILL_COLOR,
            text_color: Rgb565::BLACK,
            push_fill: Rgb565::BLACK,
            push_text: BUTTON_FILL_COLOR,
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
                    ui::Ids::Key(index as u8),
                ));
            }
        }
        self.add(Button::new(
            KEY_X_OFFSET as u16,
            KEY_Y_OFFSET as u16 + 4 * KEY_Y_SPACING as u16,
            BUTTON_WIDTH as u16,
            BUTTON_HEIGHT as u16,
            Some("0"),
            ui::Ids::Key(0),
        ));
        let mut button = Button::new(
            KEY_X_OFFSET as u16 + 3 * KEY_X_SPACING as u16,
            KEY_Y_OFFSET as u16 + 3 * KEY_Y_SPACING as u16,
            BUTTON_WIDTH as u16,
            DOUBLE_BUTTON_HEIGHT as u16,
            Some(">"),
            ui::Ids::Enter,
        );
        button.change_colors(Rgb565::RED, Rgb565::BLACK);
        self.add(button);

        self.add(Button::new(
            KEY_X_OFFSET as u16 + 1 * KEY_X_SPACING as u16,
            KEY_Y_OFFSET as u16 + 4 * KEY_Y_SPACING as u16,
            BUTTON_WIDTH as u16,
            BUTTON_HEIGHT as u16,
            Some("."),
            ui::Ids::DecimalPoint,
        ));
        self.add(Button::new(
            KEY_X_OFFSET as u16 + 2 * KEY_X_SPACING as u16,
            KEY_Y_OFFSET as u16 + 4 * KEY_Y_SPACING as u16,
            BUTTON_WIDTH as u16,
            BUTTON_HEIGHT as u16,
            Some("±"),
            ui::Ids::PlusMinus,
        ));
        self.add(Button::new(
            KEY_X_OFFSET as u16 + 3 * KEY_X_SPACING as u16,
            KEY_Y_OFFSET as u16 + 2 * KEY_Y_SPACING as u16,
            BUTTON_WIDTH as u16,
            BUTTON_HEIGHT as u16,
            Some("C"),
            ui::Ids::Clear,
        ));
        self.add(Button::new(
            KEY_X_OFFSET as u16 + 3 * KEY_X_SPACING as u16,
            KEY_Y_OFFSET as u16 + 1 * KEY_Y_SPACING as u16,
            BUTTON_WIDTH as u16,
            BUTTON_HEIGHT as u16,
            Some("H"),
            ui::Ids::Half,
        ));

        button.change_colors(ORANGE, Rgb565::BLACK);
        self.add(button);

        let x = SEVEN_SEG_WIDTH + 8;
        let y = SEVEN_SEG_TOP + 0 * SEVEN_SEG_VSPACE + (SEVEN_SEG_HEIGHT - BUTTON_HEIGHT) / 2;
        let mut button = Button::new(
            x,
            y,
            (BUTTON_WIDTH - 1) as u16,
            BUTTON_HEIGHT as u16,
            Some("X0"),
            ui::Ids::X0Button,
        );

        let x = x + BUTTON_WIDTH * 10 / 50;
        let y = y + BUTTON_HEIGHT * 27 / 40;
        button.change_colors(LIGHT_BLUE, Rgb565::BLACK);
        button.change_text_position(x, y);
        self.add(button);

        let y = y + SEVEN_SEG_VSPACE;
        let mut button = Button::new(
            SEVEN_SEG_WIDTH + 8,
            SEVEN_SEG_TOP + 1 * SEVEN_SEG_VSPACE + (SEVEN_SEG_HEIGHT - BUTTON_HEIGHT) / 2,
            (BUTTON_WIDTH - 1) as u16,
            BUTTON_HEIGHT as u16,
            Some("Y0"),
            ui::Ids::Y0Button,
        );
        button.change_colors(LIGHT_BLUE, Rgb565::BLACK);
        button.change_text_position(x, y);
        self.add(button);

        let y = y + SEVEN_SEG_VSPACE;
        let mut button = Button::new(
            SEVEN_SEG_WIDTH + 8,
            SEVEN_SEG_TOP + 2 * SEVEN_SEG_VSPACE + (SEVEN_SEG_HEIGHT - BUTTON_HEIGHT) / 2,
            (BUTTON_WIDTH - 1) as u16,
            BUTTON_HEIGHT as u16,
            Some("Z0"),
            ui::Ids::Z0Button,
        );
        button.change_colors(LIGHT_BLUE, Rgb565::BLACK);
        button.change_text_position(x, y);
        self.add(button);

        let mut button = Button::new(
            KEY_X_OFFSET,
            1,
            BUTTON_WIDTH as u16,
            BUTTON_HEIGHT as u16,
            Some("X"),
            ui::Ids::XButton,
        );
        button.change_colors(LIGHT_BLUE, Rgb565::BLACK);
        self.add(button);
        let mut button = Button::new(
            KEY_X_OFFSET + 1 * KEY_X_SPACING as u16,
            1,
            BUTTON_WIDTH as u16,
            BUTTON_HEIGHT as u16,
            Some("Y"),
            ui::Ids::YButton,
        );
        button.change_colors(LIGHT_BLUE, Rgb565::BLACK);
        self.add(button);
        let mut button = Button::new(
            KEY_X_OFFSET + 2 * KEY_X_SPACING as u16,
            1,
            BUTTON_WIDTH as u16,
            BUTTON_HEIGHT as u16,
            Some("Z"),
            ui::Ids::ZButton,
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

    pub fn locate(&mut self, x: u16, y: u16) -> Option<ui::Ids> {
        for mut button in self.buttons {
            if button.inside(x, y) {
                return Some(button.id);
            };
        }
        None
    }
}

// #[derive(Copy, Clone, Debug)]
// enum DisplayAxis {
//     X,
//     Y,
//     Z,
// }

// Locations etc for the three seven segment displays

#[derive(Copy, Clone, Debug)]
pub enum Axis {
    X,
    Y,
    Z,
    None,
}
#[derive(Copy, Clone, Debug)]
struct NumberEntryState {
    which_number: Axis,
    state: KeyState,
}

#[derive(Copy, Clone, Debug)]
pub struct View {
    buttons: Buttons,
    x: SevenSegDisplay,
    y: SevenSegDisplay,
    z: SevenSegDisplay,
    pub active_id: Option<ui::Ids>,
    key_state: KeyState,
    current_axis: Axis,
}

// Locations etc for the three seven segment displays
#[derive(Copy, Clone, Debug)]
pub enum KeyState {
    Waiting,
    NumberEntry(Axis),
    PlusMinus,
    Half,
    // UseNumber(Axis),
}

impl View {
    pub fn new() -> View {
        let mut x = SevenSegDisplay::new(
            SEVEN_SEG_LEFT,
            SEVEN_SEG_TOP + 0 * SEVEN_SEG_VSPACE,
            SEVEN_SEG_WIDTH,
            SEVEN_SEG_HEIGHT,
        );
        x.set_value(-900.);

        let mut y = SevenSegDisplay::new(
            SEVEN_SEG_LEFT,
            SEVEN_SEG_TOP + 1 * SEVEN_SEG_VSPACE,
            SEVEN_SEG_WIDTH,
            SEVEN_SEG_HEIGHT,
        );
        y.set_value(100.0);

        let mut z = SevenSegDisplay::new(
            SEVEN_SEG_LEFT,
            SEVEN_SEG_TOP + 2 * SEVEN_SEG_VSPACE,
            SEVEN_SEG_WIDTH,
            SEVEN_SEG_HEIGHT,
        );
        z.set_value(20.14540);

        View {
            buttons: Buttons::new(),
            x,
            y,
            z,
            active_id: None,
            key_state: KeyState::Waiting,
            current_axis: Axis::None,
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

    pub fn button_id_from_coords(mut self, x: u16, y: u16) -> Option<ui::Ids> {
        self.buttons.locate(x, y)
    }

    // Takes an Option<id> and de/activates the button if there
    // is an id and if it is valid.
    pub fn activate_button_from_id(
        self,
        id_in: Option<ui::Ids>,
        display: &mut Stm32F7DiscoDisplay<u16>,
    ) {
        if let Some(id) = id_in {
            for mut button in self.buttons.buttons {
                if button.id == id {
                    button.activate(display);
                }
            }
        }
    }

    // Takes an Option<id> and de/activates the button if there
    // is an id and if it is valid.
    pub fn deactivate_button_from_id(
        self,
        id_in: Option<ui::Ids>,
        display: &mut Stm32F7DiscoDisplay<u16>,
    ) {
        if let Some(id) = id_in {
            for mut button in self.buttons.buttons {
                if button.id == id {
                    button.deactivate(display);
                }
            }
        }
    }

    pub fn use_number(self, axis: Axis, number: f32) {
        // rprintln!("Use number: {:.3}", number);
    }

    pub fn process_button(&mut self, src: Option<ui::Ids>, display: &mut Stm32F7DiscoDisplay<u16>) {
        if self.active_id == src {
            // Button still pushed down - ignore
            return;
        };
        self.activate_button_from_id(src, display);
        self.deactivate_button_from_id(self.active_id, display);
        self.active_id = src;

        // rprintln!("Src: {:?}", src);

        match self.key_state {
            KeyState::Waiting => {
                // rprintln!("View: Waiting");
                if let Some(src) = src {
                    match src {
                        ui::Ids::X0Button => {
                            self.x.set_value(10.0);
                            self.x.draw(display);
                        }
                        ui::Ids::Y0Button => {
                            self.y.set_value(0.0);
                            self.y.draw(display);
                        }
                        ui::Ids::Z0Button => {
                            self.z.set_value(-10.0);
                            self.z.draw(display);
                        }
                        ui::Ids::XButton => {
                            // rprintln!("X");
                            self.x.start(display);
                            self.key_state = KeyState::NumberEntry(Axis::X);
                        }
                        ui::Ids::YButton => {
                            // rprintln!("Y");
                            self.y.start(display);
                            self.key_state = KeyState::NumberEntry(Axis::Y);
                        }
                        ui::Ids::ZButton => {
                            // rprintln!("Z");
                            self.z.start(display);
                            self.key_state = KeyState::NumberEntry(Axis::Z);
                        }
                        ui::Ids::PlusMinus => {
                            self.key_state = KeyState::PlusMinus;
                        }
                        ui::Ids::Half => {
                            self.key_state = KeyState::Half;
                        }
                        _ => (),
                    };
                };
            }

            KeyState::NumberEntry(axis) => {
                // rprintln!("View: Number entry: {:?}", axis);
                if let Some(src) = src {
                    let result = match axis {
                        Axis::X => self.x.input(src, display),
                        Axis::Y => self.y.input(src, display),
                        Axis::Z => self.z.input(src, display),
                        _ => Some(Err(0xFD)),
                    };

                    match result {
                        None => self.key_state = KeyState::NumberEntry(axis),
                        Some(r) => match r {
                            Err(_e) => self.key_state = KeyState::Waiting,
                            Ok(n) => {
                                // rprintln!("In view, number is: {:.3}", n);
                                self.use_number(axis, n);
                                self.key_state = KeyState::Waiting;
                                // rprintln!("Axis: {:?} and number: {}", axis, n);
                            }
                        },
                    }
                }
            }

            KeyState::PlusMinus => {
                // rprintln!("PlusMinus");
                if let Some(src) = src {
                    match src {
                        ui::Ids::XButton => {
                            // rprintln!("X+-");
                            self.x.plus_minus(display);
                        }
                        ui::Ids::YButton => {
                            // rprintln!("Y+-");
                            self.y.plus_minus(display);
                        }
                        ui::Ids::ZButton => {
                            // rprintln!("Z+-");
                            self.z.plus_minus(display);
                        }
                        _ => {}
                    }
                    self.key_state = KeyState::Waiting;
                } else {
                    self.key_state = KeyState::PlusMinus;
                }
            }
            KeyState::Half => {
                // rprintln!("Half");
                if let Some(src) = src {
                    match src {
                        ui::Ids::XButton => {
                            // rprintln!("X/2");
                            self.x.half(display);
                        }
                        ui::Ids::YButton => {
                            // rprintln!("Y/2");
                            self.y.half(display);
                        }
                        ui::Ids::ZButton => {
                            // rprintln!("Z/2");
                            self.z.half(display);
                        }
                        _ => {}
                    }
                    self.key_state = KeyState::Waiting;
                } else {
                    self.key_state = KeyState::Half;
                }
            }
        };
    }
}
