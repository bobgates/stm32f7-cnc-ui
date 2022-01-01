// #![deny(warnings)]
#![no_main]
#![no_std]

use core::ops::Range;
use cortex_m_rt::entry;
// use cortex_m_semihosting::{hprint, hprintln};
use embedded_graphics::{
    mono_font::MonoTextStyle,
    pixelcolor::{Rgb565, RgbColor},
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle, RoundedRectangle},
    text::Text,
};

// use ft5336::Ft5336;

#[allow(unused_imports)]
use panic_semihosting;

// use ft5336::Ft5336;
use profont::PROFONT_24_POINT;
use rtt_target::{rprintln, rtt_init_print};
use screen::Stm32F7DiscoDisplay;

use stm32f7xx_hal::{
    delay::Delay,
    gpio::Speed,
    i2c::{BlockingI2c, Mode},
    ltdc::{Layer, PixelFormat},
    pac,
    prelude::_embedded_hal_blocking_delay_DelayMs,
    prelude::*,
    rcc::{HSEClock, HSEClockMode, Rcc},
};

// mod ft5336;
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

fn draw_keypad(display: &mut Stm32F7DiscoDisplay<u16>) {
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
                display,
            );
        }
    }
    button(KEY_X_OFFSET, KEY_Y_OFFSET + 3 * KEY_Y_SPACING, "0", display);
    button(
        KEY_X_OFFSET + 1 * KEY_X_SPACING,
        KEY_Y_OFFSET + 3 * KEY_Y_SPACING,
        ".",
        display,
    );
    button(
        KEY_X_OFFSET + 2 * KEY_X_SPACING,
        KEY_Y_OFFSET + 3 * KEY_Y_SPACING,
        "",
        display,
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
    .draw(display)
    .ok();

    let style = MonoTextStyle::new(&PROFONT_24_POINT, Rgb565::YELLOW);

    Text::new(
        "300.89",
        Point::new(KEY_X_OFFSET + TEXT_XOFFSET, 5 + TEXT_YOFFSET),
        style,
    )
    .draw(display)
    .ok();
}

// impl cortex_m::prelude::_embedded_hal_blocking_delay_DelayUs<i32> for stm32f7xx_hal::delay::Delay {
//     fn delay(&self) {}
// }

#[entry]
fn main() -> ! {
    rtt_init_print!();

    rprintln!("Started");

    let perif = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

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
    let mut delay = Delay::new(cp.SYST, clocks);

    rprintln!("Connecting to I2c");
    let scl = gpioh.ph7.into_alternate_open_drain::<4>(); //LCD_SCL
    let sda = gpioh.ph8.into_alternate_open_drain::<4>(); //LSD_SDA

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
    disp_on.set_high();

    draw_keypad(&mut display);

    let mut i2c = BlockingI2c::i2c3(
        perif.I2C3,
        (scl, sda),
        Mode::fast(100_000.Hz()),
        clocks,
        &mut rcc_hal.apb1,
        10_000,
    );

    // const VALID_ADDR_RANGE: Range<u8> = 0x08..0x78;

    // ****************************************************************************************************

    // for addr in 0x00_u8..0x80 {
    //     // Write the empty array and check the slave response.
    //     let byte: [u8; 1] = [0; 1];
    //     if VALID_ADDR_RANGE.contains(&addr) && i2c.write(addr, &byte).is_ok() {
    //         addresses += 1;
    //         rprintln!("Address: {}", addr);
    //     }
    // }
    // rprintln!("Found {} I2C devices on bus 3", addresses);

    let cmd: [u8; 1] = [0];
    let mut buf: [u8; 1] = [0];

    const FT5336_OK: u8 = 0;
    const FT5336_ERROR: u8 = 0xff;
    const FT5336_TOUCHPAD_ADDR: u8 = 0x38;
    // const FT5336_MAX_NB_TOUCH: u8 = 0x05;
    // I2C device addresses
    const FT5336_DEV_MODE_REG: u8 = 0x00;

    /* Gesture ID register */
    const FT5336_GEST_ID_REG: u8 = 0x01;

    /* Touch Data Status register : gives number of active touch points (0..2) */
    const FT5336_TD_STAT_REG: u8 = 0x02;

    /* P1 X, Y coordinates, weight and misc registers */
    const FT5336_P1_XH_REG: u8 = 0x03;
    const FT5336_P1_XL_REG: u8 = 0x04;
    const FT5336_P1_YH_REG: u8 = 0x05;
    const FT5336_P1_YL_REG: u8 = 0x06;
    const FT5336_P1_WEIGHT_REG: u8 = 0x07;
    const FT5336_P1_MISC_REG: u8 = 0x08;
    const FT5336_P1_XH_TP_BIT_MASK: u8 = 0x0F;
    const FT5336_P1_XH_TP_BIT_POSITION: u8 = 0;
    const FT5336_P1_XL_TP_BIT_MASK: u8 = 0xFF;
    const FT5336_P1_XL_TP_BIT_POSITION: u8 = 0;

    const FT5336_GMODE_REG: u8 = 0xA4;

    /* FT5336 Chip identification register */
    const FT5336_CHIP_ID_REG: u8 = 0xA8;

    /* Release code version */
    const FT5336_RELEASE_CODE_ID_REG: u8 = 0xAF;

    /* Current operating mode the FT5336 system is in (R) */
    const FT5336_STATE_REG: u8 = 0xBC;

    if i2c.write(FT5336_TOUCHPAD_ADDR, &cmd).is_ok() {
        rprintln!("Wrote to touchpad okay");
    }

    if !i2c
        .write_read(FT5336_TOUCHPAD_ADDR, &[FT5336_CHIP_ID_REG], &mut buf)
        .is_ok()
    {
        rprintln!("Attempt to read chip ID Failed");
    } else {
        let v = buf[0] as u8;
        rprintln!("Chip id is {:02x}", v);
    }

    if !i2c
        .write_read(FT5336_TOUCHPAD_ADDR, &[FT5336_STATE_REG], &mut buf)
        .is_ok()
    {
        rprintln!("Attempt to read chip state failed");
    } else {
        let v = buf[0] as u8;
        rprintln!("Chip state is {:02x}", v);
    }

    let mut buf: [u8; 2] = [0, 0];
    if !i2c
        .write_read(FT5336_TOUCHPAD_ADDR, &[FT5336_P1_XH_REG], &mut buf)
        .is_ok()
    {
        rprintln!("Attempt to read P1_XH failed");
    } else {
        let v = buf[0] as u8;
        let w = buf[1] as u8;
        rprintln!("Chip state is {:02x}, {:02x}", v, w);
    }

    let mut step_number: u32 = 0;
    loop {
        let mut buf: [u8; 1] = [0];
        if !i2c
            .write_read(FT5336_TOUCHPAD_ADDR, &[FT5336_TD_STAT_REG], &mut buf)
            .is_ok()
        {
            rprintln!("Attempt to read chip status failed");
        } else {
            if buf[0] == 1 {
                let _v = buf[0] as u8;
                let mut buf: [u8; 5] = [0; 5];
                i2c.write_read(FT5336_TOUCHPAD_ADDR, &[FT5336_P1_XH_REG], &mut buf);
                // let mut x1 = buf[0] as u8;
                // i2c.write_read(FT5336_TOUCHPAD_ADDR, &[FT5336_P1_XL_REG], &mut buf);
                // let x2 = buf[0] as u8;
                // i2c.write_read(FT5336_TOUCHPAD_ADDR, &[FT5336_P1_YH_REG], &mut buf);
                // let mut y1 = buf[0] as u8;
                // i2c.write_read(FT5336_TOUCHPAD_ADDR, &[FT5336_P1_YL_REG], &mut buf);
                // let y2 = buf[0] as u8;
                //
                // if v != 255 && v != 0 {
                //     rprintln!(
                //         "Touch status is {:02x} - Key is {:02x}{:02x}x{:02x}{:02x}",
                //         v,
                //         buf[0] as u8,
                //         buf[1] as u8,
                //         buf[2] as u8,
                //         buf[3] as u8,
                //     );
                // }
                let status: [u8; 1] = [0];
                let a = i2c
                    .write_read(FT5336_TOUCHPAD_ADDR, &[FT5336_GMODE_REG], &mut buf)
                    .is_ok();
                // rprintln!("Status is: {:02x}", status[0]);
                rprintln!(
                    "{:10}: {:03},{:03}  - {:02x}",
                    step_number,
                    buf[2] as u16 * 256 + buf[3] as u16,
                    269 - ((buf[0] & 0x7F) as u16 * 256 + buf[1] as u16),
                    buf[4] as u8,
                );
                // let mut buf: [u8; 1] = [0];
                // i2c.write_read(FT5336_TOUCHPAD_ADDR, &[FT5336_P1_XH_REG], &mut buf);
                // let mut xh = buf[0] as u8;
                // i2c.write_read(FT5336_TOUCHPAD_ADDR, &[FT5336_P1_XL_REG], &mut buf);
                // let xl = buf[0] as u8;
                // i2c.write_read(FT5336_TOUCHPAD_ADDR, &[FT5336_P1_YH_REG], &mut buf);
                // let mut yh = buf[0] as u8;
                // i2c.write_read(FT5336_TOUCHPAD_ADDR, &[FT5336_P1_YL_REG], &mut buf);
                // let yl = buf[0] as u8;
                // if v != 255 && v != 0 {
                //     rprintln!(
                //         "------------ is {:02x} - Key is {:02x}{:02x}x{:02x}{:02x}",
                //         v,
                //         xh as u8,
                //         xl as u8,
                //         yh as u8,
                //         yl as u8,
                //     );
                // }
            } else {
                rprintln!("fingers: {}", buf[0]);
            }
        }
        // delay.delay_us(500);
        step_number += 1;
    }

    // const FT62XX_REG_NUMTOUCHES: u8 = 0x0; // !< Touch X position

    // let byte: [u8; 1] = [0; 1]; //FT62XX_REG_NUMTOUCHES; 1];
    // let mut buffer: [u8; 16] = [0; 16];
    // if !i2c.write(0x1A, &byte).is_ok() {
    //     rprintln!("Error response when sending request for touches");
    // } else {
    //     if !i2c.read(0x38, &mut buffer).is_ok() {
    //         rprintln!("Error in reading for touches");
    //     };
    //     for i in 0..16 {
    //         rprintln!("{}: {}", i, buffer[i] as u8);
    //     }
    // }

    // let ft5336 = Ft5336::new(i2c);

    // rprintln!("On 0x38 the registers contain:");
    // let mut byte: [u8; 10] = [0x01; 10];
    // for i in 0x00_u8..0x10 {
    //     if i2c.write(0x38, &byte).is_ok() {
    //         i2c.read(0x38, &mut byte);
    //         for i in 0..10 {
    //             rprintln!("Buffer {}: {}", i, byte[i] as u8);
    //         }
    //     } else {
    //         rprintln!("unsuccessful at writing");
    //     }
    // }

    // let style = MonoTextStyle::new(&PROFONT_24_POINT, Rgb565::YELLOW);

    // let a = match addresses {
    //     0 => "0",
    //     1 => "1",
    //     2 => "2",
    //     3 => "3",
    //     4 => "4",
    //     5 => "5",
    //     6 => "6",
    //     7 => "7",
    //     8 => "8",
    //     9 => "9",
    //     _ => "More",
    // };

    // loop {}
}
