use crate::utils::print_by_queue::PrintFullByQueue;
use crossterm::queue;
use crossterm::style::{PrintStyledContent, Stylize};
use std::io::stdout;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum Entity {
    Player,
    Box,
}

impl PrintFullByQueue for Entity {
    fn print_full(&self) -> Result<(), std::io::Error> {
        queue!(
            stdout(),
            match *self {
                Self::Box => PrintStyledContent("$".dark_yellow()),
                Self::Player => PrintStyledContent("@".blue()),
            }
        )
    }
}
