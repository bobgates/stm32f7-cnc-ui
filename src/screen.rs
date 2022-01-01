use embedded_graphics::{
    // drawable::Pixel,
    geometry::Size,
    pixelcolor::{Rgb565, RgbColor},
    prelude::*,
    primitives::Rectangle,
    Pixel,
};

use stm32f7xx_hal::{
    ltdc::{DisplayConfig, DisplayController, Layer, PixelFormat, SupportedWord},
    pac::{DMA2D, LTDC},
    prelude::*,
    rcc::{HSEClock, HSEClockMode},
};

/// STM32F7-DISCO board display
pub const DISCO_SCREEN_CONFIG: DisplayConfig = DisplayConfig {
    active_width: 480,
    active_height: 272,
    h_back_porch: 13,
    h_front_porch: 30,
    h_sync: 41,
    v_back_porch: 2,
    v_front_porch: 2,
    v_sync: 10,
    frame_rate: 60,
    h_sync_pol: false,
    v_sync_pol: false,
    no_data_enable_pol: false,
    pixel_clock_pol: false,
};

pub struct Stm32F7DiscoDisplay<T: 'static + SupportedWord> {
    pub controller: DisplayController<T>,
}

impl<T: 'static + SupportedWord> Stm32F7DiscoDisplay<T> {
    pub fn new(ltdc: LTDC, dma2d: DMA2D) -> Stm32F7DiscoDisplay<T> {
        let controller = DisplayController::new(
            ltdc,
            dma2d,
            PixelFormat::RGB565,
            DISCO_SCREEN_CONFIG,
            Some(&HSEClock::new(25_000_000.Hz(), HSEClockMode::Bypass)),
        );

        Stm32F7DiscoDisplay { controller }
    }
}

impl Stm32F7DiscoDisplay<u16> {
    /// Draw a hardware accelerated (by DMA2D) rectangle
    #[allow(dead_code)]
    pub fn draw_rectangle(
        &mut self,
        rect: Rectangle,
        _c: Rgb565, // item: &Styled<primitives::rectangle::Rectangle, PrimitiveStyle<Rgb565>>,
    ) -> Result<(), <Stm32F7DiscoDisplay<u16> as DrawTarget>::Error> {
        if let Some(_bottom_right) = rect.bottom_right() {
            // if item.style.stroke_color.is_none() {
            // let color = match item.style.fill_color {
            //     Some(c) => {
            //         (c.b() as u32 & 0x1F)
            //             | ((c.g() as u32 & 0x3F) << 5)
            //             | ((c.r() as u32 & 0x1F) << 11)
            //     }
            //     None => 0u32,
            // };

            // Note(unsafe) because transfert might not be before an other write
            // to the buffer occurs. However, such Register -> Buffer transfert
            // is so fast that such issue does not occur
            // TODO : use safer DMA api when the embedde-hal DMA traits will be stabilised
            // unsafe {
            //     self.controller
            //         .draw_rectangle(Layer::L1,
            //             (item.primitive.top_left.x as usize, item.primitive.top_left.y as usize),
            //             (bottom_right.x as usize, bottom_right.y as usize),
            //             (c.b() as u32 & 0x1F)
            //                         | ((c.g() as u32 & 0x3F) << 5)
            //                         | ((c.r() as u32 & 0x1F) << 11));
            // }
            // }
        }
        Ok(())
    }
}

impl DrawTarget for Stm32F7DiscoDisplay<u16> {
    type Color = Rgb565;
    type Error = core::convert::Infallible;

    /// Draw a `Pixel` that has a color defined
    // fn draw_pixel(&mut self, pixel: Pixel<Rgb565>) -> Result<(), Self::Error> {
    //     let Pixel(coord, color) = pixel;
    //     let value: u16 = (color.b() as u16 & 0x1F)
    //         | ((color.g() as u16 & 0x3F) << 5)
    //         | ((color.r() as u16 & 0x1F) << 11);

    //     // TODO : draw pixel
    //     self.controller
    //         .draw_pixel(Layer::L1, coord.x as usize, coord.y as usize, value);
    //     Ok(())
    // }

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            let value: u16 = (color.b() as u16 & 0x1F)
                | ((color.g() as u16 & 0x3F) << 5)
                | ((color.r() as u16 & 0x1F) << 11);

            // TODO : draw pixel
            self.controller
                .draw_pixel(Layer::L1, coord.x as usize, coord.y as usize, value);
        }
        Ok(())
    }
}
impl OriginDimensions for Stm32F7DiscoDisplay<u16> {
    /// Return the size of the screen
    fn size(&self) -> Size {
        Size::new(480, 272)
    }
}
