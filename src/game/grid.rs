use crate::utils::print_by_queue::PrintFullByQueue;
use crossterm::queue;
use crossterm::style::{PrintStyledContent, Stylize};
use std::io::stdout;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum Grid {
    Wall,
    Ground,
    Target,
}

impl PrintFullByQueue for Grid {
    fn print_full(&self) -> Result<(), std::io::Error> {
        queue!(
            stdout(),
            match *self {
                Self::Wall => PrintStyledContent("#".grey()),
                Self::Ground => PrintStyledContent(" ".reset()),
                Self::Target => PrintStyledContent(".".cyan()),
            }
        )
    }
}
