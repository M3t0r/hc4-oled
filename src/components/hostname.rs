use super::{Component, Drawer, Error};

use embedded_graphics::{
    prelude::*,
    text::{Baseline, Text},
};

#[derive(Debug)]
pub struct Hostname {
    pub hostname: Option<String>,
}

impl std::fmt::Display for Hostname {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Hostname")
    }
}

impl Component for Hostname
{
    fn should_update(&self, _last_update: std::time::Duration) -> bool {
        return self.hostname.is_none(); // update when hostname not present
    }

    fn update(&mut self) -> Result<(), Error> {
        self.hostname = Some(hostname::get()?.to_string_lossy().into());
        Ok(())
    }

    fn draw(&self, drawable: &mut Drawer, offset: Point, _tick: u64) -> Result<(), Error> {
        Text::with_baseline(
            self.hostname.as_ref().ok_or("hostname not available")?,
            offset,
            drawable.base_text_style,
            Baseline::Top,
        )
        .draw(&mut drawable.display)?;

        Ok(())
    }
}
