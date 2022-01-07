// #![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m_rt::entry;
// use embedded_graphics::{
//     mono_font::MonoTextStyle,
//     pixelcolor::{Rgb565, RgbColor},
//     prelude::*,
//     // primitives::{PrimitiveStyleBuilder, Rectangle, RoundedRectangle},
//     // text::Text,
// };

use ft5336::Ft5336;

#[allow(unused_imports)]
use panic_semihosting;

// use ft5336::Ft5336;
// use profont::PROFONT_24_POINT;
use rtt_target::{rprintln, rtt_init_print};
// use screen::Stm32F7DiscoDisplay;

use stm32f7xx_hal::{
    delay::Delay,
    gpio::Speed,
    i2c::{BlockingI2c, Mode},
    ltdc::{Layer, PixelFormat},
    pac,
    prelude::*,
    rcc::{HSEClock, HSEClockMode, Rcc},
};

mod screen;
mod ui;
mod view;

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
    display.controller.config_layer(
        Layer::L1,
        unsafe { &mut view::FB_LAYER1 },
        PixelFormat::RGB565,
    );

    display.controller.enable_layer(Layer::L1);
    display.controller.reload();
    disp_on.set_high();

    let touch_zones = view::draw_keypad(&mut display);

    let mut i2c = BlockingI2c::i2c3(
        perif.I2C3,
        (scl, sda),
        Mode::fast(100_000.Hz()),
        clocks,
        &mut rcc_hal.apb1,
        10_000,
    );

    let mut touch = Ft5336::new(&i2c, 0x38, &mut delay).unwrap();

    loop {
        let mut current_key = ' ';
        let n = touch.detect_touch(&mut i2c).unwrap();
        if n != 0 {
            let t = touch.get_touch(&mut i2c, 1).unwrap();
            match touch_zones.locate(t.y, t.x) {
                Some(e) => {}
                None => {}
            };
        };
        touch.delay_ms(10);
    }
}
