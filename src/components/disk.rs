use super::{Component, Drawer, Error};

use std::path::Path;

use embedded_graphics::{
    prelude::*,
    text::{Baseline, Text},
};

#[derive(Debug)]
pub struct Disk {
    name: String,
    size: u64,
    available: u64,
}

impl Disk {
    pub fn new_from_name(name: &str) -> Result<Self, Error> {
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

impl Component for Disk {
    fn should_update(&self, _last_update: std::time::Duration) -> bool {
        return false;
    }

    fn update(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn draw(&self, drawable: &mut Drawer, offset: Point, _tick: u64) -> Result<(), Error> {
        Text::with_baseline(
            &self.name,
            offset,
            drawable.base_text_style.clone(),
            Baseline::Top,
        )
        .draw(&mut drawable.display)?;
        Ok(())
    }
}
