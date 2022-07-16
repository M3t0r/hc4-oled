use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::PrimitiveStyle,
};
use ssd1306::{mode::BufferedGraphicsMode, prelude::*, I2CDisplayInterface, Ssd1306};

use i2c_linux::I2c;

use std::fs::File;
use std::io::Write;
use std::path::Path;

use std::time::{Instant, SystemTime};

use clap::Parser;

mod components;
use components::{Component, Disk, Hostname, Load, UpdateIndicator};

mod units;
pub use units::{Base, GlancableSizesWithOrdersOfMagnitude};

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

pub struct Drawer<'a> {
    display: Display,
    base_text_style: MonoTextStyle<'a, BinaryColor>,
    base_primitive_style: PrimitiveStyle<BinaryColor>,
}

impl Drawer<'_> {
    pub const BURNIN_OFFSET_MAX: u8 = 5;
    pub const WIDTH: u8 = 64 - Self::BURNIN_OFFSET_MAX;
    pub const HEIGHT: u8 = 128 - Self::BURNIN_OFFSET_MAX;
    pub const LINE_HEIGHT: u8 = 11;
    pub fn new_from_device_path(path: &Path) -> Result<Self, Error> {
        let mut display = Ssd1306::new(
            I2CDisplayInterface::new(EmbeddedHALWriter(I2c::<File>::from_path(path)?)),
            DisplaySize128x64,
            DisplayRotation::Rotate270,
        )
        .into_buffered_graphics_mode();
        display.init()?;
        display.set_display_on(true)?;

        Ok(Self {
            display,
            base_text_style: MonoTextStyleBuilder::new()
                .font(&FONT_6X10)
                .text_color(BinaryColor::On)
                .build(),
            base_primitive_style: PrimitiveStyle::with_stroke(BinaryColor::On, 1),
        })
    }

    pub fn draw(
        &mut self,
        tick: u64,
        components: &mut Vec<Box<dyn Component>>,
    ) -> Result<(), Error> {
        let burn_in_offset = Point::new(
            (tick / 17u64 % Self::BURNIN_OFFSET_MAX as u64) as i32,
            (tick / 11u64 % Self::BURNIN_OFFSET_MAX as u64) as i32,
        );

        self.display.clear();

        for (i, c) in components.iter().enumerate() {
            c.draw(
                self,
                burn_in_offset + Point::new(0, (Self::LINE_HEIGHT * i as u8).into()),
                tick,
            )?;
        }

        self.display.flush()?;
        Ok(())
    }
}

impl Drop for Drawer<'_> {
    fn drop(&mut self) {
        self.display.set_display_on(false).unwrap(); // turn off on shut down
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The I2C device to use to communicate with the display
    #[clap(long)]
    device: std::path::PathBuf,

    /// Which disks to monitor
    #[clap(long = "disk")]
    disks: Vec<String>,

    /// Enable CPU load graph
    #[clap(short, long)]
    load: bool,
}

fn main() {
    let args = Args::parse();

    dbg!(&args);

    let mut drawer = Drawer::new_from_device_path(&args.device).expect("Could not access display");

    let mut components: Vec<Box<dyn Component>> = Vec::with_capacity(8);
    components.push(Box::new(Hostname { hostname: None }));

    components.extend(
        args.disks
            .iter()
            .map(|name| Disk::new_from_name(name))
            .filter_map(|disk| -> Option<Box<dyn Component>> {
                match disk {
                    Ok(d) => Some(Box::new(d)),
                    Err(e) => {
                        println!("{}", e);
                        None
                    }
                }
            }),
    );

    if args.load {
        components.push(Box::new(Load::new().expect("Could not collect load stats")));
    }

    components.push(Box::new(UpdateIndicator {}));

    let mut last_updates: Vec<Instant> = vec![Instant::now(); components.len()];

    for c in &mut components {
        match c.update() {
            Ok(_) => (),
            Err(e) => println!("{}", e),
        };
    }

    println!("Started");

    loop {
        let tick = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Could not get time")
            .as_secs();

        for (i, c) in &mut components.iter_mut().enumerate() {
            if c.should_update(Instant::now() - last_updates[i]) {
                println!("Updating {}", &c);
                match c.update() {
                    Ok(_) => (),
                    Err(e) => println!("{}", e),
                };
                last_updates[i] = Instant::now();
            }
        }

        drawer
            .draw(tick, &mut components)
            .expect("Could not draw update");
    }
}
