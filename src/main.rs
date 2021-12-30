// #![deny(warnings)]
#![no_main]
#![no_std]

// Required
extern crate panic_semihosting;
use cortex_m_rt::entry;
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::{BinaryColor, Rgb565, RgbColor},
    prelude::*,
    primitives::{
        Circle, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, RoundedRectangle,
        StrokeAlignment, Triangle,
    },
    text::{Alignment, Text},
};
use numtoa::NumToA;
// use std::String;

use profont::PROFONT_24_POINT;
use screen::Stm32F7DiscoDisplay;

use stm32f7xx_hal::{
    self as hal,
    gpio::Speed,
    i2c::{BlockingI2c, I2c, Mode},
    ltdc::{Layer, PixelFormat},
    pac,
    // draw_rectangle,
    prelude::*,
    rcc::{HSEClock, HSEClockMode, Rcc},
};

mod screen;

// DIMENSIONS
const WIDTH: u16 = 480;
const HEIGHT: u16 = 272;

// Graphics framebuffer
const FB_GRAPHICS_SIZE: usize = (WIDTH as usize) * (HEIGHT as usize);
static mut FB_LAYER1: [u16; FB_GRAPHICS_SIZE] = [0; FB_GRAPHICS_SIZE];

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
const KEY_X_OFFSET: i32 = 290;
const KEY_Y_OFFSET: i32 = 60;
const KEY_X_SPACING: i32 = 65;
const KEY_Y_SPACING: i32 = 55;

fn button(x: i32, y: i32, caption: &str, display: &mut Stm32F7DiscoDisplay<u16>) {
    let style = PrimitiveStyleBuilder::new()
        .stroke_width(BUTTON_STROKE_WIDTH)
        .stroke_color(BUTTON_STROKE_COLOR)
        .fill_color(BUTTON_FILL_COLOR)
        .build();

    RoundedRectangle::with_equal_corners(
        Rectangle::new(Point::new(x, y), Size::new(BUTTON_WIDTH, BUTTON_HEIGHT)),
        Size::new(CORNER_RADIUS, CORNER_RADIUS),
    )
    .into_styled(style)
    .draw(display)
    .ok();

    let style = MonoTextStyle::new(&PROFONT_24_POINT, TEXT_COLOR);

    Text::new(
        caption,
        Point::new(x + TEXT_XOFFSET, y + TEXT_YOFFSET),
        style,
    )
    .draw(display)
    .ok();
}

#[entry]
fn main() -> ! {
    let perif = pac::Peripherals::take().unwrap();
    let _cp = cortex_m::Peripherals::take().unwrap();

    let mut rcc_hal: Rcc = perif.RCC.constrain();

    // Set up pins
    let _gpioa = perif.GPIOA.split();
    let _gpiob = perif.GPIOB.split();
    let gpioe = perif.GPIOE.split();
    let gpiog = perif.GPIOG.split();
    let gpioh = perif.GPIOH.split();
    let gpioi = perif.GPIOI.split();
    let gpioj = perif.GPIOJ.split();
    let gpiok = perif.GPIOK.split();

    gpioe.pe4.into_alternate::<14>().set_speed(Speed::VeryHigh); // LTCD_B0

    gpiog.pg12.into_alternate::<9>().set_speed(Speed::VeryHigh); // LTCD_B4

    gpioi.pi9.into_alternate::<14>().set_speed(Speed::VeryHigh); // LTCD_VSYNC
    gpioi.pi10.into_alternate::<14>().set_speed(Speed::VeryHigh); // LTCD_HSYNC
    gpioi.pi13.into_alternate::<14>().set_speed(Speed::VeryHigh);
    gpioi.pi14.into_alternate::<14>().set_speed(Speed::VeryHigh); // LTCD_CLK
    gpioi.pi15.into_alternate::<14>().set_speed(Speed::VeryHigh); // LTCD_R0

    gpioj.pj0.into_alternate::<14>().set_speed(Speed::VeryHigh); // LTCD_R1
    gpioj.pj1.into_alternate::<14>().set_speed(Speed::VeryHigh); // LTCD_R2
    gpioj.pj2.into_alternate::<14>().set_speed(Speed::VeryHigh); // LTCD_R3
    gpioj.pj3.into_alternate::<14>().set_speed(Speed::VeryHigh); // LTCD_R4
    gpioj.pj4.into_alternate::<14>().set_speed(Speed::VeryHigh); // LTCD_R5
    gpioj.pj5.into_alternate::<14>().set_speed(Speed::VeryHigh); // LTCD_R6
    gpioj.pj6.into_alternate::<14>().set_speed(Speed::VeryHigh); // LTCD_R7
    gpioj.pj7.into_alternate::<14>().set_speed(Speed::VeryHigh); // LTCD_G0
    gpioj.pj8.into_alternate::<14>().set_speed(Speed::VeryHigh); // LTCD_G1
    gpioj.pj9.into_alternate::<14>().set_speed(Speed::VeryHigh); // LTCD_G2
    gpioj.pj10.into_alternate::<14>().set_speed(Speed::VeryHigh); // LTCD_G3
    gpioj.pj11.into_alternate::<14>().set_speed(Speed::VeryHigh); // LTCD_G4
    gpioj.pj13.into_alternate::<14>().set_speed(Speed::VeryHigh); // LTCD_B1
    gpioj.pj14.into_alternate::<14>().set_speed(Speed::VeryHigh); // LTCD_B2
    gpioj.pj15.into_alternate::<14>().set_speed(Speed::VeryHigh); // LTCD_B3

    gpiok.pk0.into_alternate::<14>().set_speed(Speed::VeryHigh); // LTCD_G5
    gpiok.pk1.into_alternate::<14>().set_speed(Speed::VeryHigh); // LTCD_G6
    gpiok.pk2.into_alternate::<14>().set_speed(Speed::VeryHigh); // LTCD_G7
    gpiok.pk4.into_alternate::<14>().set_speed(Speed::VeryHigh); // LTCD_B5
    gpiok.pk5.into_alternate::<14>().set_speed(Speed::VeryHigh); // LTCD_B6
    gpiok.pk6.into_alternate::<14>().set_speed(Speed::VeryHigh); // LTCD_D7
    gpiok.pk7.into_alternate::<14>().set_speed(Speed::VeryHigh); // LTCD_E

    // HSE osc out in High Z
    gpioh.ph1.into_floating_input();
    let clocks = rcc_hal
        .cfgr
        .hse(HSEClock::new(25_000_000.Hz(), HSEClockMode::Bypass))
        .sysclk(216_000_000.Hz())
        .hclk(216_000_000.Hz())
        .freeze();

    let scl = gpioh.ph7.into_alternate_open_drain::<4>();
    let sda = gpioh.ph8.into_alternate_open_drain::<4>();

    let i2c = hal::i2c::BlockingI2c::i2c3(
        perif.I2C3,
        (scl, sda),
        Mode::fast(100_000.Hz()),
        clocks,
        &mut rcc_hal.apb1,
        10_000,
    );

    // pub fn i2c3(
    //     i2c: I2C3,
    //     pins: (SCL, SDA),
    //     mode: Mode,
    //     clocks: Clocks,
    //     apb: &mut <I2C3 as RccBus>::Bus,
    //     data_timeout_us: u32
    // ) -> Self

    // LCD enable: set it low first to avoid LCD bleed while setting up timings
    let mut disp_on = gpioi.pi12.into_push_pull_output();
    disp_on.set_low();

    // LCD backlight enable
    let mut backlight = gpiok.pk3.into_push_pull_output();
    backlight.set_high();

    let mut display = screen::Stm32F7DiscoDisplay::new(perif.LTDC, perif.DMA2D);
    display
        .controller
        .config_layer(Layer::L1, unsafe { &mut FB_LAYER1 }, PixelFormat::RGB565);

    display.controller.enable_layer(Layer::L1);
    display.controller.reload();

    // let display = &mut display;

    // LCD enable: activate LCD !
    disp_on.set_high();

    // Example of circle
    // let circle = Circle::new(Point::new(22,22), 20)
    //     .into_styled(PrimitiveStyle::with_stroke(Rgb565::new(0, 0b11110, 0b11011), 3));
    // circle.draw(&mut display);

    let style = PrimitiveStyleBuilder::new()
        // .stroke_color(Rgb565::RED)
        // .stroke_width(3)
        .fill_color(BUTTON_STROKE_COLOR)
        .build();

    // let d = Rectangle::new((32,32), (448,240))
    //     .into_styled(style);
    //     .draw(&mut display);

    let c = BACKGROUND_COLOR;

    let color: u32 =
        c.b() as u32 & 0x1F | ((c.g() as u32 & 0x3F) << 5) | ((c.r() as u32 & 0x1F) << 11);

    unsafe {
        display
            .controller
            .draw_rectangle(Layer::L1, (0, 0), (480, 272), color); //0x003f3 as u32);
    }

    let mut buf = [0u8; 20];
    for i in (0..3).rev() {
        for j in 0..3 {
            let a = match 1 + i * 3 + j {
                0 => "0",
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
            button(
                KEY_X_OFFSET + j * KEY_X_SPACING,
                KEY_Y_OFFSET + i * KEY_Y_SPACING,
                a,
                &mut display,
            );
        }
    }
    button(
        KEY_X_OFFSET,
        KEY_Y_OFFSET + 3 * KEY_Y_SPACING,
        "0",
        &mut display,
    );
    button(
        KEY_X_OFFSET + 1 * KEY_X_SPACING,
        KEY_Y_OFFSET + 3 * KEY_Y_SPACING,
        ".",
        &mut display,
    );
    button(
        KEY_X_OFFSET + 2 * KEY_X_SPACING,
        KEY_Y_OFFSET + 3 * KEY_Y_SPACING,
        "",
        &mut display,
    );

    let style = PrimitiveStyleBuilder::new()
        .stroke_width(BUTTON_STROKE_WIDTH)
        .stroke_color(BUTTON_STROKE_COLOR)
        .fill_color(Rgb565::BLACK)
        .build();

    RoundedRectangle::with_equal_corners(
        Rectangle::new(
            Point::new(KEY_X_OFFSET, KEY_Y_OFFSET - 55),
            Size::new(BUTTON_WIDTH + (2 * KEY_X_SPACING) as u32, BUTTON_HEIGHT),
        ),
        Size::new(CORNER_RADIUS, CORNER_RADIUS),
    )
    .into_styled(style)
    .draw(&mut display)
    .ok();

    let style = MonoTextStyle::new(&PROFONT_24_POINT, Rgb565::YELLOW);

    Text::new(
        "300.89",
        Point::new(KEY_X_OFFSET + TEXT_XOFFSET, 5 + TEXT_YOFFSET),
        style,
    )
    .draw(&mut display)
    .ok();

    // button(20, 40, "1", &mut display);

    // display.controller.draw_rectangle();

    //     top_left = (32,32),
    //     bottom_right = (479, 271),
    //     style = primitive_style!(fill_color = Rgb565::new(0, 0b11110, 0b11011))
    // );
    // r.draw(display).ok();

    // let c1 = egcircle!(
    //     center = (20, 20),
    //     radius = 8,
    //     style = primitive_style!(fill_color = Rgb565::new(0, 63, 0))
    // );

    // let c2 = egcircle!(
    //     center = (25, 20),
    //     radius = 8,
    //     style = primitive_style!(fill_color = Rgb565::new(31, 0, 0))
    // );

    // let t = egtext!(
    //     text = "Hello Rust!",
    //     top_left = (100, 100),
    //     style = text_style!(font = Font6x8, text_color = RgbColor::WHITE)
    // );

    // c1.draw(display).ok();
    // c2.draw(display).ok();
    // t.draw(display).ok();

    for i in 0..300 {
        // let c1 = egcircle!(
        //     center = (20 + i, 20),
        //     radius = 8,
        //     style = primitive_style!(fill_color = RgbColor::GREEN)
        // );
        // c1.draw(display).ok();
    }

    // display.flush().unwrap();

    loop {}
}
