use super::screen::Screen;
use crate::utils::print_by_queue::PrintFullByQueue;

pub struct ExitScreen;

impl ExitScreen {
    pub fn new() -> Self {
        Self {}
    }
}

impl PrintFullByQueue for ExitScreen {
    fn print_full(&self) -> Result<(), std::io::Error> {
        Ok(())
    }
}

impl Screen for ExitScreen {
    fn update(
        &mut self,
        _event: Option<crossterm::event::Event>,
    ) -> super::screen::ScreenTransition {
        super::screen::ScreenTransition::Break
    }
}
