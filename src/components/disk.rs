use super::{Component, Drawer, Error};
use crate::{Base, GlancableSizesWithOrdersOfMagnitude};

use std::path::{Path, PathBuf};

use embedded_graphics::{
    prelude::*,
    primitives::Line,
    text::{Baseline, Text},
};

#[derive(Debug)]
pub struct Disk {
    name: String,
    mount_point: PathBuf,
    mounted: bool,
    size: u64,
    available: u64,
}

impl Disk {
    pub fn new_from_path(path: &Path) -> Result<Self, Error> {
        Ok(Self {
            name: path
                .file_name()
                .ok_or("Could not get name from mountpoint")?
                .to_string_lossy()
                .to_string(),
            mount_point: path.to_path_buf(),
            mounted: false,
            size: 0,
            available: 0,
        })
    }

    fn is_mounted(&self) -> Result<bool, Error> {
        // check if the filesystem ID between the mount point and the parent dir
        // differ. if they don't, then they belong to the same filesystem and
        // the mount point is not actually a mount point but just a folder on a disk.
        let mount_parent = self
            .mount_point
            .parent()
            .ok_or("Could not get parent of mountpoint")?;
        let mount_parent_fs_id = nix::sys::statvfs::statvfs(mount_parent)?.filesystem_id();
        let mount_point_fs_id = nix::sys::statvfs::statvfs(&self.mount_point)?.filesystem_id();

        Ok(mount_point_fs_id != mount_parent_fs_id)
    }

    fn draw_usage_bar(&self, drawable: &mut Drawer, offset: Point) -> Result<(), Error> {
        let bar_width =
            (Drawer::WIDTH as f32 * (1f32 - (self.available as f32 / self.size as f32))) as i32;

        Line::new(Point::new(0, 5) + offset, Point::new(bar_width, 5) + offset)
            .into_styled(drawable.base_primitive_style)
            .draw(&mut drawable.display)?;

        Line::new(Point::new(0, 2) + offset, Point::new(0, 8) + offset)
            .into_styled(drawable.base_primitive_style)
            .draw(&mut drawable.display)?;

        Line::new(
            Point::new(Drawer::WIDTH.into(), 2) + offset,
            Point::new(Drawer::WIDTH.into(), 8) + offset,
        )
        .into_styled(drawable.base_primitive_style)
        .draw(&mut drawable.display)?;

        let size_str = format!(
            "{}",
            GlancableSizesWithOrdersOfMagnitude::new(self.size, Base::Ten)
        );
        let size_text = Text::with_baseline(
            size_str.as_str(),
            offset + Point::new(0, 0),
            drawable.base_text_style,
            Baseline::Top,
        );
        let size_text_width = size_text.bounding_box().size.width;
        size_text
            .translate(Point::new(
                ((Drawer::WIDTH as u32 - size_text_width) / 2) as i32,
                0,
            ))
            .draw(&mut drawable.display)?;

        Ok(())
    }

    fn draw_not_mounted(&self, drawable: &mut Drawer, offset: Point) -> Result<(), Error> {
        let text = Text::with_baseline(
            "-/-",
            offset + Point::new(0, 0),
            drawable.base_text_style,
            Baseline::Top,
        );
        let text_width = text.bounding_box().size.width;
        text.translate(Point::new(
            ((Drawer::WIDTH as u32 - text_width) / 2) as i32,
            0,
        ))
        .draw(&mut drawable.display)?;

        Ok(())
    }
}

impl std::fmt::Display for Disk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Component for Disk {
    fn should_update(&self, last_update: std::time::Duration) -> bool {
        last_update
            > std::time::Duration::from_secs(match self.mounted {
                true => 60 * 5,
                false => 5, // redetect disks quickly
            })
    }

    fn update(&mut self) -> Result<(), Error> {
        self.mounted = self.is_mounted()?;

        if !self.mounted {
            // we can't collect any info if not mounted, but that's ok, we'll just display an error
            return Ok(());
        }

        let stats = nix::sys::statfs::statfs(&self.mount_point)
            .map_err(|e| format!("Could not collect stats for disk '{}': {}", self.name, e))?;

        self.size = stats.blocks() * stats.block_size() as u64;
        self.available = stats.blocks_available() * stats.block_size() as u64; // available to non-root
                                                                               // self.free = stats.blocks_free() * stats.block_size() as u64; // available to root

        Ok(())
    }

    fn draw(&self, drawable: &mut Drawer, offset: Point, _tick: u64) -> Result<(), Error> {
        match self.mounted {
            true => self.draw_usage_bar(drawable, offset),
            false => self.draw_not_mounted(drawable, offset),
        }
    }
}
