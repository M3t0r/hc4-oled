use super::{Component, Drawer, Error};

use embedded_graphics::{
    image::{Image, ImageRaw},
    pixelcolor::BinaryColor,
    prelude::*,
};

#[derive(Debug)]
pub struct UpdateIndicator {}

impl std::fmt::Display for UpdateIndicator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UpdateIndicator")
    }
}

impl Component for UpdateIndicator {
    fn should_update(&self, _last_update: std::time::Duration) -> bool {
        return false;
    }

    fn update(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn draw(&self, drawable: &mut Drawer, _offset: Point, tick: u64) -> Result<(), Error> {
        #[rustfmt::skip]
        const FRAMES: &[&[u8]; 2] = &[
            &[
                0b010_00000,
                0b000_00000,
                0b010_00000,
            ],
            &[
                0b000_00000,
                0b101_00000,
                0b000_00000
            ],
        ];

        Image::new(
            &ImageRaw::<BinaryColor>::new(
                FRAMES[tick as usize % FRAMES.len()],
                3,
            ),
            Point::new(64 - 3, 128 - 3),
        )
        .draw(&mut drawable.display)?;

        Ok(())
    }
}
