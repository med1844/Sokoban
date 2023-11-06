use crossterm::cursor::{MoveTo, MoveToNextLine};
use crossterm::queue;
use crossterm::terminal::Clear;
use std::io::stdout;

use super::cell::Cell;
use super::entity::Entity;
use super::game_command::GameCommand;
use super::game_event::GameEvent;
use super::grid::Grid;
use crate::screen::screen::ScreenTransition;
use crate::utils::print_by_queue::PrintFullByQueue;

pub struct Game {
    cells: Vec<Vec<Cell>>,
    n: usize,
    m: usize,
    i: usize,
    j: usize,
}

impl Game {
    pub fn new(cells: Vec<Vec<Cell>>) -> Self {
        let n = cells.len();
        let m = cells.first().unwrap_or(&vec![]).len();
        fn get_ij(cells: &Vec<Vec<Cell>>) -> Result<(usize, usize), &str> {
            for (i, row) in cells.iter().enumerate() {
                for (j, val) in row.iter().enumerate() {
                    if let Some(Entity::Player) = val.entity {
                        return Ok((i, j));
                    }
                }
            }
            Err("entities doesn't contain player")
        }
        match get_ij(&cells) {
            Ok((i, j)) => Self { cells, n, m, i, j },
            Err(e) => panic!("{}", e),
        }
    }

    pub fn push_entity(&mut self, src: (usize, usize), d: (usize, usize)) -> Vec<GameEvent> {
        let (i, j) = src;
        let (di, dj) = d;
        let ni = i.overflowing_add(di).0;
        let nj = j.overflowing_add(dj).0;
        let mut res = vec![];
        if ni < self.n && nj < self.m {
            match self.cells[ni][nj].grid {
                Grid::Ground | Grid::Target => {
                    if let Some(Entity::Box) = self.cells[ni][nj].entity {
                        res.append(&mut self.push_entity((ni, nj), d.clone()));
                    }
                    if self.cells[ni][nj].entity.is_none() {
                        self.cells[ni][nj].entity = std::mem::take(&mut self.cells[i][j].entity);
                        res.push(GameEvent::Put(i, j, self.cells[i][j].clone()));
                        res.push(GameEvent::Put(ni, nj, self.cells[ni][nj].clone()));
                        self.i = ni;
                        self.j = nj;
                    }
                }
                _ => {}
            }
        }
        res
    }

    pub fn execute(&mut self, command: GameCommand) -> (ScreenTransition, Vec<GameEvent>) {
        let events = match command {
            GameCommand::Null => vec![],
            GameCommand::Up => self.push_entity((self.i, self.j), (usize::MAX, 0)),
            GameCommand::Down => self.push_entity((self.i, self.j), (1, 0)),
            GameCommand::Left => self.push_entity((self.i, self.j), (0, usize::MAX)),
            GameCommand::Right => self.push_entity((self.i, self.j), (0, 1)),
            GameCommand::Quit => vec![],
        };
        (
            match command {
                GameCommand::Quit => ScreenTransition::Break,
                _ => ScreenTransition::Continue,
            },
            events,
        )
    }
}

impl From<&str> for Game {
    fn from(value: &str) -> Self {
        let rows = value.split('\n');
        Self::new(
            rows.into_iter()
                .map(|v| {
                    v.chars()
                        .into_iter()
                        .map(|c| match c {
                            ' ' => Cell::new(Grid::Ground, None),
                            '#' => Cell::new(Grid::Wall, None),
                            '@' => Cell::new(Grid::Ground, Some(Entity::Player)),
                            '$' => Cell::new(Grid::Ground, Some(Entity::Box)),
                            '.' => Cell::new(Grid::Target, None),
                            '+' => Cell::new(Grid::Target, Some(Entity::Player)),
                            '*' => Cell::new(Grid::Target, Some(Entity::Box)),
                            _ => panic!("Invalid char!"),
                        })
                        .collect()
                })
                .collect(),
        )
    }
}

impl PrintFullByQueue for Game {
    fn print_full(&self) -> Result<(), std::io::Error> {
        queue!(
            stdout(),
            Clear(crossterm::terminal::ClearType::All),
            MoveTo(0, 0)
        )?;
        for row in self.cells.iter() {
            for cell in row.iter() {
                cell.print_full()?;
            }
            queue!(stdout(), MoveToNextLine(1))?;
        }
        Ok(())
    }
}