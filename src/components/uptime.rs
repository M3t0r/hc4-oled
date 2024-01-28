use super::{Component, Drawer, Error};

use embedded_graphics::{
    prelude::*,
    text::{Baseline, Text},
};

use systemstat::{System, Platform};

pub struct Uptime {
    sys: System,
    display_string: String,
}

impl Uptime {
    pub fn new() -> Self {
        Self {
            sys: System::new(),
            display_string: "".to_string(),
        }
    }
}

impl std::fmt::Display for Uptime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Uptime")
    }
}

impl std::fmt::Debug for Uptime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Uptime")
    }
}

impl Component for Uptime {
    fn should_update(&self, last_update: std::time::Duration) -> bool {
        last_update > std::time::Duration::from_secs(15)
    }

    fn update(&mut self) -> Result<(), Error> {
        let uptime = self.sys.uptime()?.as_secs();
        self.display_string = format!(
            "{:3}d{:02}h{:02}m",
            uptime / (60*60*24),
            uptime / (60*60) % 24,
            uptime / 60 % 60,
        );
        Ok(())
    }

    fn draw(&self, drawable: &mut Drawer, offset: Point, _tick: u64) -> Result<(), Error> {
        Text::with_baseline(
            &self.display_string,
            offset,
            drawable.base_text_style,
            Baseline::Top,
        )
        .draw(&mut drawable.display)?;

        Ok(())
    }
}
