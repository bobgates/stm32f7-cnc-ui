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
    gpio::Speed,
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

fn button(x: i32, y: i32, caption: &str, display: &mut Stm32F7DiscoDisplay<u16>) {
    const BUTTON_WIDTH: u32 = 45;
    const BUTTON_HEIGHT: u32 = 50;
    const TEXT_XOFFSET: i32 = 15;
    const TEXT_YOFFSET: i32 = 32;
    const CORNER_RADIUS: u32 = 6;
    const BUTTON_STROKE_WIDTH: u32 = 3;
    const BUTTON_STROKE_COLOR: Rgb565 = <Rgb565>::BLUE;
    const BUTTON_FILL_COLOR: Rgb565 = <Rgb565>::WHITE;
    const TEXT_COLOR: Rgb565 = <Rgb565>::BLACK;

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

    let rcc_hal: Rcc = perif.RCC.constrain();

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
    let _clocks = rcc_hal
        .cfgr
        .hse(HSEClock::new(25_000_000.Hz(), HSEClockMode::Bypass))
        .sysclk(216_000_000.Hz())
        .hclk(216_000_000.Hz())
        .freeze();

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
        .fill_color(Rgb565::BLUE)
        .build();

    // let d = Rectangle::new((32,32), (448,240))
    //     .into_styled(style);
    //     .draw(&mut display);

    let c = Rgb565::RED;

    let color: u32 =
        c.b() as u32 & 0x1F | ((c.g() as u32 & 0x3F) << 5) | ((c.r() as u32 & 0x1F) << 11);

    unsafe {
        display
            .controller
            .draw_rectangle(Layer::L1, (0, 0), (480, 272), color); //0x003f3 as u32);
    }

    let mut buf = [0u8; 20];
    for i in 0..3 {
        for j in 0..3 {
            let a = match (1 + i * 3 + j) {
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
            button(250 + j * 60, 40 + i * 70, a, &mut display);
        }
    }

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
