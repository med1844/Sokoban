use super::entity::Entity;
use super::grid::Grid;
use crate::utils::print_by_queue::PrintFullByQueue;
use crossterm::queue;
use crossterm::style::{PrintStyledContent, Stylize};
use std::io::stdout;

#[derive(Clone, Copy)]
pub struct Cell {
    pub grid: Grid,
    pub entity: Option<Entity>,
}

impl Cell {
    pub fn new(grid: Grid, entity: Option<Entity>) -> Self {
        Self { grid, entity }
    }
}

impl PrintFullByQueue for Cell {
    fn print_full(&self) -> Result<(), std::io::Error> {
        match (self.entity, self.grid) {
            (None, g) => g.print_full(),
            (Some(Entity::Box), Grid::Target) => queue!(stdout(), PrintStyledContent("*".yellow())),
            (Some(Entity::Player), Grid::Target) => {
                queue!(stdout(), PrintStyledContent("+".green()))
            }
            (Some(e), Grid::Ground) => e.print_full(),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Impossible state!",
            )),
        }
    }
}
