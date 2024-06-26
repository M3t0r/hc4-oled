pub use crate::{Drawer, Error};

// pub trait Component<D>: std::fmt::Debug
// where
//     D: embedded_graphics::prelude::DrawTarget<Color = embedded_graphics::pixelcolor::BinaryColor, Error = display_interface::DisplayError>,
// {

pub trait Component: std::fmt::Debug + std::fmt::Display
{
    fn should_update(&self, last_update: std::time::Duration) -> bool;

    fn update(&mut self) -> Result<(), Error>;

    fn draw(
        &self,
        drawable: &mut Drawer,
        offset: embedded_graphics::prelude::Point,
        tick: u64,
    ) -> Result<(), Error>;
}

mod disk;
mod load;
mod memory;
mod hostname;
mod update_indicator;
mod uptime;

pub use self::disk::Disk;
pub use self::load::Load;
pub use self::memory::Memory;
pub use self::hostname::Hostname;
pub use self::update_indicator::UpdateIndicator;
pub use self::uptime::Uptime;
