use core::ops::Range;
// use rtt_target::{rprintln, rtt_init_print};
use stm32f7xx_hal::{
    gpio::Speed,
    i2c::{BlockingI2c, I2c, Mode, PinScl, PinSda},
    ltdc::{Layer, PixelFormat},
    pac,
    pac::Peripherals,
    pac::I2C3,
    prelude::*,
    rcc::{Clocks, HSEClock, HSEClockMode, Rcc},
};
#[allow(dead_code)]
const VALID_ADDR_RANGE: Range<u8> = 0x08..0x78;

// pub struct Ft5336<I2C3, SCL, SDA> {
//     i2c: BlockingI2c<{BlockingI2c<I2C3, SCL, SDA> as Trait}>::i2c3,
// }

// impl<I2C> Ft5336<I2C3, SCL, SDA> {
//     //<I2c, Sda, Scl> {
//     pub fn new(i2c: I2C3) -> Ft5336<I2C3, SCL, SDA> {
//         let mut addresses = 0;

//         for addr in 0x00_u8..0x80 {
//             // Write the empty array and check the slave response.
//             let byte: [u8; 1] = [0; 1];
//             if VALID_ADDR_RANGE.contains(&addr) && i2c.write(addr, &byte).is_ok() {
//                 addresses += 1;
//                 rprintln!("Address: {}", addr);
//             }
//         }
//         rprintln!("Found {} I2C devices on bus 3", addresses);
//     }

//     pub fn detect_touch(self) {
//         // i2c
//     }
// }
