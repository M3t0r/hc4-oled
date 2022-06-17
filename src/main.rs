use embedded_graphics::{
    image::{Image, ImageRaw},
    mono_font::{ascii::FONT_6X10, MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use ssd1306::{mode::BufferedGraphicsMode, prelude::*, I2CDisplayInterface, Ssd1306};

use i2c_linux::I2c;

use std::fs::File;
use std::io::Write;
use std::path::Path;

use std::time::SystemTime;

type Display = Ssd1306<
    I2CInterface<EmbeddedHALWriter<File>>,
    DisplaySize128x64,
    BufferedGraphicsMode<DisplaySize128x64>,
>;

// i2c_linux is written to use Linux system devices, while embedded_graphics is
// targeting embedded device where the I2C bus is behind a bunch of registers,
// not a path on the filesystem. But in the end they are both simple devices
// expecting byte arrays to be written to them. So we just bridge the gap.
struct EmbeddedHALWriter<I>(i2c_linux::I2c<I>);

impl embedded_hal::blocking::i2c::Write for EmbeddedHALWriter<File> {
    type Error = std::io::Error;
    fn write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), std::io::Error> {
        self.0.smbus_set_slave_address(addr.into(), false)?;
        self.0.write_all(bytes)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum Error {
    DisplayError(display_interface::DisplayError),
    IOError(std::io::Error),
    Errno(nix::errno::Errno),
    String(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::DisplayError(inner) => format!("{:?}", inner),
                Self::IOError(inner) => inner.to_string(),
                Self::Errno(inner) => inner.to_string(),
                Self::String(inner) => inner.to_string(),
            }
        )
    }
}

impl From<std::io::Error> for Error {
    fn from(v: std::io::Error) -> Self {
        Self::IOError(v)
    }
}

impl From<display_interface::DisplayError> for Error {
    fn from(v: display_interface::DisplayError) -> Self {
        Self::DisplayError(v)
    }
}

impl From<nix::errno::Errno> for Error {
    fn from(v: nix::errno::Errno) -> Self {
        Self::Errno(v)
    }
}

impl From<&str> for Error {
    fn from(v: &str) -> Self {
        Self::String(v.to_string())
    }
}

impl From<String> for Error {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}

struct Drawer<'a> {
    display: Display,
    base_text_style: MonoTextStyle<'a, BinaryColor>,
}

impl Drawer<'_> {
    pub fn new_from_device_path(path: &Path) -> Result<Self, Error> {
        let mut display = Ssd1306::new(
            I2CDisplayInterface::new(EmbeddedHALWriter(I2c::<File>::from_path(path)?)),
            DisplaySize128x64,
            DisplayRotation::Rotate90,
        )
        .into_buffered_graphics_mode();
        display.init()?;
        display.set_display_on(true)?;

        Ok(Self {
            display,
            base_text_style: Self::get_base_text_style(),
        })
    }

    fn get_base_text_style() -> MonoTextStyle<'static, BinaryColor> {
        MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(BinaryColor::On)
            .build()
    }

    pub fn draw(&mut self, data: &DrawableData, tick: u64) -> Result<(), Error> {
        let burn_in_offset = Point::new((tick / 17 % 5) as i32, (tick / 11 % 5) as i32);

        self.display.clear();

        self.draw_hostname(data, burn_in_offset, tick);
        self.draw_update_indicator(data, burn_in_offset, tick);

        self.display.flush()?;
        Ok(())
    }

    fn draw_hostname(
        &mut self,
        data: &DrawableData,
        burn_in_offset: Point,
        _tick: u64,
    ) -> Result<(), Error> {
        Text::with_baseline(
            data.hostname.as_ref().ok_or("hostname not available")?,
            Point::zero() + burn_in_offset,
            self.base_text_style.clone(),
            Baseline::Top,
        )
        .draw(&mut self.display)?;

        Ok(())
    }

    fn draw_update_indicator(
        &mut self,
        _data: &DrawableData,
        _burn_in_offset: Point,
        tick: u64,
    ) -> Result<(), Error> {
        #[rustfmt::skip]
        const V: &[u8] = &[
            0b010_00000,
            0b000_00000,
            0b010_00000,
        ];
        #[rustfmt::skip]
        const H: &[u8] = &[
            0b000_00000,
            0b101_00000,
            0b000_00000
        ];

        Image::new(
            &ImageRaw::<BinaryColor>::new(
                match tick % 2 {
                    0 => H,
                    1..=u64::MAX => V,
                },
                3,
            ),
            Point::new(64 - 3, 128 - 3),
        )
        .draw(&mut self.display)?;

        Ok(())
    }
}

impl Drop for Drawer<'_> {
    fn drop(&mut self) {
        self.display.set_display_on(false).unwrap(); // turn off on shut down
    }
}

struct DrawableData {
    hostname: Option<String>,
    disks: Vec<DiskData>,
}

#[derive(Debug)]
struct DiskData {
    name: String,
    size: u64,
    available: u64,
}

impl DiskData {
    fn new_from_name(name: &str) -> Result<Self, Error> {
        // todo: check if path is mount point
        let stats = nix::sys::statfs::statfs(&Path::new("/data/chunks/").join(name))
            .map_err(|e| format!("Could not collect stats for disk '{}': {}", name, e))?;

        Ok(Self {
            name: name.to_string(),
            size: stats.blocks() * stats.block_size() as u64,
            available: stats.blocks_available() * stats.block_size() as u64, // available to non-root
                                                                             // free: stats.blocks_free() * stats.block_size() as u64, // available to root
        })
    }
}

fn main() {
    let mut drawer = Drawer::new_from_device_path(std::path::Path::new("/dev/i2c-0"))
        .expect("Could not access display");

    let hostname: String = hostname::get().unwrap().to_string_lossy().into();

    loop {
        let tick = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Could not get time")
            .as_secs();

        let data = DrawableData {
            hostname: Some(hostname.clone()),
            disks: [
                "my-disk-1",
                "my-disk-2",
            ]
            .iter()
            .map(|name| DiskData::new_from_name(name))
            .filter_map(|disk| match disk {
                Ok(d) => Some(d),
                Err(e) => {
                    println!("{}", e);
                    None
                }
            })
            .collect::<Vec<_>>(),
        };

        drawer.draw(&data, tick).expect("Could not draw update");
    }
}
