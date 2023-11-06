use super::screen::Screen;
use crate::utils::print_by_queue::PrintFullByQueue;
use crossterm::{cursor::MoveTo, queue, style::Print, terminal::Clear};
use std::io::stdout;

pub struct ExitScreen;

impl ExitScreen {
    pub fn new() -> Self {
        Self {}
    }
}

impl PrintFullByQueue for ExitScreen {
    fn print_full(&self) -> Result<(), std::io::Error> {
        queue!(
            stdout(),
            MoveTo(0, 0),
            Clear(crossterm::terminal::ClearType::All),
            Print("Press any key to exit...")
        )?;
        Ok(())
    }
}

impl Screen for ExitScreen {
    fn handle_input(&mut self, _event: crossterm::event::Event) -> super::screen::ScreenTransition {
        super::screen::ScreenTransition::Break
    }
}
