use super::{Component, Drawer, Error};
use crate::{GlancableSizesWithOrdersOfMagnitude, Base};

use std::path::Path;

use embedded_graphics::{
    prelude::*,
    text::{Baseline, Text},
    primitives::Line,
};

#[derive(Debug)]
pub struct Disk {
    name: String,
    size: u64,
    available: u64,
}

impl Disk {
    pub fn new_from_name(name: &str) -> Result<Self, Error> {
        Ok(Self {
            name: name.to_string(),
            size: 0,
            available: 0,
        })
    }
}

impl std::fmt::Display for Disk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Component for Disk {
    fn should_update(&self, last_update: std::time::Duration) -> bool {
        return last_update > std::time::Duration::from_secs(60 * 5);
    }

    fn update(&mut self) -> Result<(), Error> {
        // todo: check if path is mount point
        let stats = nix::sys::statfs::statfs(&Path::new("/data/chunks/").join(&self.name))
            .map_err(|e| format!("Could not collect stats for disk '{}': {}", self.name, e))?;

        self.size = stats.blocks() * stats.block_size() as u64;
        self.available = stats.blocks_available() * stats.block_size() as u64; // available to non-roo
        // self.free = stats.blocks_free() * stats.block_size() as u64; // available to root

        Ok(())
    }

    fn draw(&self, drawable: &mut Drawer, offset: Point, _tick: u64) -> Result<(), Error> {
        let bar_width = (Drawer::WIDTH as f32 * (1f32 - (self.available as f32 / self.size as f32))) as i32;

        Line::new(
            Point::new(0, 5) + offset,
            Point::new(bar_width, 5) + offset
        ).into_styled(drawable.base_primitive_style)
        .draw(&mut drawable.display)?;

        Line::new(
            Point::new(0, 2) + offset,
            Point::new(0, 8) + offset
        ).into_styled(drawable.base_primitive_style)
        .draw(&mut drawable.display)?;

        Line::new(
            Point::new(Drawer::WIDTH.into(), 2) + offset,
            Point::new(Drawer::WIDTH.into(), 8) + offset
        ).into_styled(drawable.base_primitive_style)
        .draw(&mut drawable.display)?;

        let size_str = format!("{}", GlancableSizesWithOrdersOfMagnitude::new(self.size, Base::Ten));
        let size_text = Text::with_baseline(
            size_str.as_str(),
            offset + Point::new(0,0),
            drawable.base_text_style,
            Baseline::Top,
        );
        let size_text_width = size_text.bounding_box().size.width;
        size_text.translate(Point::new(
            ((Drawer::WIDTH as u32 - size_text_width) / 2) as i32,
            0
        )).draw(&mut drawable.display)?;

        Ok(())
    }
}
