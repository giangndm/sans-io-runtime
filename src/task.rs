use std::time::Instant;

use crate::TaskSwitcherChild;

pub mod group;
pub mod switcher;

#[derive(Debug)]
pub enum TaskError {
    Shutdowned,
}

/// Represents a task.
pub trait Task<In, Out>: TaskSwitcherChild<Out> {
    /// Called each time the task is ticked. Default is 1ms.
    fn on_tick(&mut self, now: Instant) -> Result<(), TaskError>;

    /// Called when an input event is received for the task.
    fn on_event(&mut self, now: Instant, input: In) -> Result<(), TaskError>;

    /// Gracefully shuts down the task.
    fn shutdown(&mut self, now: Instant) -> Result<(), TaskError>;
}
