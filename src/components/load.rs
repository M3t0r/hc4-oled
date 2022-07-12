use std::collections::VecDeque;

use super::{Component, Drawer, Error};

use systemstat::{System, Platform, data::{DelayedMeasurement, CPULoad}};

use embedded_graphics::{
    prelude::*,
    primitives::Line,
};

pub struct Load {
    sys: System,
    measurement: Option<DelayedMeasurement<CPULoad>>,
    graph_values: VecDeque<f32>,
}

impl Load {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            sys: System::new(),
            measurement: None,
            graph_values: VecDeque::with_capacity(Drawer::WIDTH.into()), // FIFO, newest in front
        })
    }

    fn start_measurement(&mut self) -> Result<(), Error> {
        self.measurement = self.sys.cpu_load_aggregate().map_err(|e| println!("Could not start load measurement: {:?}", e)).ok();
        Ok(())
    }
    fn collect_measurement(&mut self) -> Result<(), Error> {
        match &self.measurement {
            None => {
                return Err("No measurement active to collect".into())
            }
            Some(measurement) => {
                let result = measurement.done()?;
                self.measurement = None;

                if self.graph_values.capacity() == self.graph_values.len() {
                    // at capacity, remove oldest data point
                    self.graph_values.pop_back();
                }

                self.graph_values.push_front(result.user);
                dbg!(&self.graph_values);

                Ok(())
            }
        }
    }
}

impl std::fmt::Display for Load {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Load")
    }
}

impl std::fmt::Debug for Load {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Load")
    }
}

impl Component for Load {
    fn should_update(&self, last_update: std::time::Duration) -> bool {
        return last_update > match self.measurement {
            // when a measurement is in progress, update after 1 second to collect values
            Some(_) => std::time::Duration::from_secs(1),
            None => std::time::Duration::from_secs(60),
        }
    }

    fn update(&mut self) -> Result<(), Error> {
        if self.measurement.is_none() {
            return self.start_measurement()
        }
        return self.collect_measurement()
    }

    fn draw(&self, drawable: &mut Drawer, offset: Point, _tick: u64) -> Result<(), Error> {
        for (i, datum) in self.graph_values.iter().enumerate() {
            Line::new(
                Point::new(Drawer::WIDTH as i32 - i as i32, Drawer::LINE_HEIGHT as i32 - (Drawer::LINE_HEIGHT as f32 * datum) as i32) + offset,
                Point::new(Drawer::WIDTH as i32 - i as i32, Drawer::LINE_HEIGHT as i32 - (Drawer::LINE_HEIGHT as f32 * datum) as i32) + offset
            ).into_styled(drawable.base_primitive_style)
            .draw(&mut drawable.display)?;
        }

        Ok(())
    }
}
