use std::collections::VecDeque;

use super::{Component, Drawer, Error};

use systemstat::{Platform, System};

use embedded_graphics::{prelude::*, primitives::Line};

pub struct Memory {
    sys: System,
    graph_values: VecDeque<f32>,
}

impl Memory {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            sys: System::new(),
            graph_values: VecDeque::with_capacity(Drawer::WIDTH.into()), // FIFO, newest in front
        })
    }
}

impl std::fmt::Display for Memory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Load")
    }
}

impl std::fmt::Debug for Memory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Load")
    }
}

impl Component for Memory {
    fn should_update(&self, last_update: std::time::Duration) -> bool {
        last_update > std::time::Duration::from_secs(60)
    }

    fn update(&mut self) -> Result<(), Error> {
        if self.graph_values.capacity() == self.graph_values.len() {
            // at capacity, remove oldest data point
            self.graph_values.pop_back();
        }
        let usage = self.sys.memory()?;
        self.graph_values
            .push_front(1.0 - (usage.free.as_u64() as f32 / usage.total.as_u64() as f32));
        Ok(())
    }

    fn draw(&self, drawable: &mut Drawer, offset: Point, _tick: u64) -> Result<(), Error> {
        for (i, datum) in self.graph_values.iter().enumerate() {
            Line::new(
                Point::new(
                    Drawer::WIDTH as i32 - i as i32,
                    Drawer::LINE_HEIGHT as i32 - (Drawer::LINE_HEIGHT as f32 * datum) as i32,
                ) + offset,
                Point::new(
                    Drawer::WIDTH as i32 - i as i32,
                    Drawer::LINE_HEIGHT as i32 - (Drawer::LINE_HEIGHT as f32 * datum) as i32,
                ) + offset,
            )
            .into_styled(drawable.base_primitive_style)
            .draw(&mut drawable.display)?;
        }

        for i in 0..Drawer::WIDTH {
            if i % 10 == 0 {
                Line::new(
                    Point::new(i.into(), Drawer::LINE_HEIGHT.into()) + offset,
                    Point::new(i.into(), Drawer::LINE_HEIGHT.into()) + offset,
                )
                .into_styled(drawable.base_primitive_style)
                .draw(&mut drawable.display)?;
            }
        }

        Line::new(Point::new(0, 0) + offset, Point::new(0, 0) + offset)
            .into_styled(drawable.base_primitive_style)
            .draw(&mut drawable.display)?;

        Line::new(
            Point::new(Drawer::WIDTH.into(), 0) + offset,
            Point::new(Drawer::WIDTH.into(), 0) + offset,
        )
        .into_styled(drawable.base_primitive_style)
        .draw(&mut drawable.display)?;

        Ok(())
    }
}
