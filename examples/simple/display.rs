use drogue_bme680::Data;
use drogue_bsec::{Accuracy, Outputs};
use embedded_graphics::drawable::Drawable;
use embedded_graphics::fonts::{Font, Font24x32, Font6x8, Text};
use embedded_graphics::geometry::Point;
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use embedded_graphics::primitives::{Primitive, Rectangle};
use embedded_graphics::style::{PrimitiveStyle, TextStyle};
use embedded_graphics::DrawTarget;
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::v2::OutputPin;
use heapless::consts;
use heapless::String;
use ssd1351::builder::Builder;
use ssd1351::interface::SpiInterface;
use ssd1351::mode::GraphicsMode;

static mut BUFFER: [u8; 128 * 128 * 2] = [0u8; 128 * 128 * 2];

pub struct Display<SPI, DC, RST>
where
    SPI: embedded_hal::blocking::spi::Transfer<u8> + embedded_hal::blocking::spi::Write<u8>,
    DC: OutputPin,
    RST: OutputPin + Sized,
{
    inner: GraphicsMode<SpiInterface<SPI, DC>>,
    rst: RST,
}

impl<SPI, DC, RST> Display<SPI, DC, RST>
where
    SPI: embedded_hal::blocking::spi::Transfer<u8> + embedded_hal::blocking::spi::Write<u8>,
    DC: OutputPin,
    RST: OutputPin + Sized,
{
    pub fn new(spi: SPI, dc: DC, rst: RST) -> Self {
        let display: GraphicsMode<_> =
            unsafe { Builder::new().connect_spi(spi, dc, &mut BUFFER).into() };
        Display {
            inner: display,
            rst,
        }
    }

    pub fn init<D>(&mut self, delay: &mut D) -> Result<(), ()>
    where
        D: DelayMs<u8>,
    {
        self.inner.reset(&mut self.rst, delay).map_err(|_| ())?;
        self.inner.init()?;
        Ok(())
    }

    fn accuracy_as_text<F, P1, P2>(
        &mut self,
        value: &Option<Accuracy<f32>>,
        value_provider: P1,
        text_provider: P2,
        font: F,
        vpadding: i32,
        top: i32,
        left: i32,
        width: i32,
    ) -> Result<(), ()>
    where
        F: Font + Copy,
        P1: FnOnce(&mut dyn core::fmt::Write, f32) -> core::fmt::Result,
        P2: FnOnce(&mut dyn core::fmt::Write, &str) -> core::fmt::Result,
    {
        let mut b1: String<consts::U16> = String::new();
        let mut b2: String<consts::U16> = String::new();

        let s = match value {
            Some(Accuracy::Low(v)) | Some(Accuracy::Medium(v)) | Some(Accuracy::High(v)) => {
                value_provider(&mut b1, *v).map_err(|_| ())?;
                &b1
            }
            _ => "<??>",
        };

        text_provider(&mut b2, s).map_err(|_| ())?;

        let color = match value {
            None => Rgb565::CYAN,
            Some(Accuracy::Unreliable) => Rgb565::MAGENTA,
            Some(Accuracy::Low(_)) => Rgb565::RED,
            Some(Accuracy::Medium(_)) => Rgb565::YELLOW,
            Some(Accuracy::High(_)) => Rgb565::GREEN,
        };

        self.draw_text(&b2, font, color, vpadding, top, left, width)
    }

    fn draw_text<F>(
        &mut self,
        value: &str,
        font: F,
        color: Rgb565,
        vpadding: i32,
        top: i32,
        left: i32,
        width: i32,
    ) -> Result<(), ()>
    where
        F: Font + Copy,
    {
        let height = F::CHARACTER_SIZE.height as i32 + vpadding * 2;

        self.clear_rect(Rectangle::new(
            Point::new(left, top),
            Point::new(left + width - 1, top + height),
        ))?;

        Text::new(value, Point::new(left, top + vpadding))
            .into_styled(TextStyle::new(font, color))
            .draw(&mut self.inner)?;

        Ok(())
    }

    fn value_as_text<F, P>(
        &mut self,
        value: P,
        font: F,
        color: Rgb565,
        vpadding: i32,
        top: i32,
        left: i32,
        width: i32,
    ) -> Result<(), ()>
    where
        F: Font + Copy,
        P: FnOnce(&mut dyn core::fmt::Write) -> core::fmt::Result,
    {
        let mut buffer: String<consts::U16> = String::new();

        value(&mut buffer).map_err(|_| ())?;

        self.draw_text(&buffer, font, color, vpadding, top, left, width)
    }

    fn optional_as_text<F, P1, P2, T>(
        &mut self,
        value: Option<&T>,
        value_provider: P1,
        text_provider: P2,
        font: F,
        color: Rgb565,
        vpadding: i32,
        top: i32,
        left: i32,
        width: i32,
    ) -> Result<(), ()>
    where
        F: Font + Copy,
        P1: FnOnce(&mut dyn core::fmt::Write, &T) -> core::fmt::Result,
        P2: FnOnce(&mut dyn core::fmt::Write, &str) -> core::fmt::Result,
    {
        let mut b1: String<consts::U16> = String::new();
        let mut b2: String<consts::U16> = String::new();

        let s: &str = match value {
            Some(v) => {
                value_provider(&mut b1, v).map_err(|_| ())?;
                &b1
            }
            None => "??",
        };

        text_provider(&mut b2, s).map_err(|_| ())?;

        self.draw_text(&b2, font, color, vpadding, top, left, width)
    }

    pub fn set_state(&mut self, data: &Data, outputs: &Outputs) -> Result<(), ()> {
        let size = self.inner.size();

        let mut top = 0;

        self.value_as_text(
            |w| write!(w, "{:.1}°", data.temperature),
            Font24x32,
            Rgb565::WHITE,
            4,
            top,
            0,
            size.width as i32,
        )?;

        top += 40;

        // first real row

        let height = 10;
        let half = size.width as i32 / 2;

        self.accuracy_as_text(
            &outputs.iaq,
            |w, v| write!(w, "{:.0}", v),
            |w, s| write!(w, "IAQ: {}", s),
            Font6x8,
            1,
            top,
            0,
            half,
        )?;
        self.accuracy_as_text(
            &outputs.static_iaq,
            |w, v| write!(w, "{:.1}", v),
            |w, s| write!(w, "SQ:  {}", s),
            Font6x8,
            1,
            top,
            half,
            half,
        )?;

        // next row

        top += height;

        self.accuracy_as_text(
            &outputs.co2_equivalent,
            |w, v| write!(w, "{:.0}", v),
            |w, s| write!(w, "CO2: {}", s),
            Font6x8,
            1,
            top,
            0,
            half,
        )?;
        self.accuracy_as_text(
            &outputs.breath_voc_equivalent,
            |w, v| write!(w, "{:.2}", v),
            |w, s| write!(w, "VOC: {}", s),
            Font6x8,
            1,
            top,
            half,
            half,
        )?;

        // next row

        top += height;

        self.accuracy_as_text(
            &outputs.compensated_gas,
            |w, v| write!(w, "{:.2}", v),
            |w, s| write!(w, "CG:  {}", s),
            Font6x8,
            1,
            top,
            0,
            half,
        )?;
        self.accuracy_as_text(
            &outputs.gas_percentage,
            |w, v| write!(w, "{:.2}", v),
            |w, s| write!(w, "G%:  {}", s),
            Font6x8,
            1,
            top,
            half,
            half,
        )?;

        // next row

        top += height;

        self.optional_as_text(
            outputs.sensor_heat_compensated_temperature.as_ref(),
            |w, v| write!(w, "{:.2}", v),
            |w, s| write!(w, "HCT: {}", s),
            Font6x8,
            Rgb565::WHITE,
            1,
            top,
            0,
            half,
        )?;
        self.optional_as_text(
            outputs.sensor_heat_compensated_humidity.as_ref(),
            |w, v| write!(w, "{:.2}", v),
            |w, s| write!(w, "HCH: {}", s),
            Font6x8,
            Rgb565::WHITE,
            1,
            top,
            half,
            half,
        )?;

        // next row

        top += height;

        self.optional_as_text(
            outputs.stabilization_status.as_ref(),
            |w, v| write!(w, "{:.2}", v),
            |w, s| write!(w, "STA: {}", s),
            Font6x8,
            Rgb565::WHITE,
            1,
            top,
            0,
            half,
        )?;
        self.optional_as_text(
            outputs.run_in_status.as_ref(),
            |w, v| write!(w, "{:.2}", v),
            |w, s| write!(w, "RIS: {}", s),
            Font6x8,
            Rgb565::WHITE,
            1,
            top,
            half,
            half,
        )?;

        // next row

        top += height;

        self.optional_as_text(
            outputs.raw_temperature.as_ref(),
            |w, v| write!(w, "{:.2}", v),
            |w, s| write!(w, "RT:  {}", s),
            Font6x8,
            Rgb565::WHITE,
            1,
            top,
            0,
            half,
        )?;
        self.optional_as_text(
            outputs.raw_humidity.as_ref(),
            |w, v| write!(w, "{:.2}", v),
            |w, s| write!(w, "RH:  {}", s),
            Font6x8,
            Rgb565::WHITE,
            1,
            top,
            half,
            half,
        )?;

        // flush

        self.inner.flush();

        Ok(())
    }

    fn clear_rect(&mut self, rect: Rectangle) -> Result<(), ()> {
        rect.into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
            .draw(&mut self.inner)
    }
}
