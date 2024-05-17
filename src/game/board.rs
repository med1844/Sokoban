use crossterm::cursor::{MoveTo, MoveToNextLine};
use crossterm::queue;
use crossterm::terminal::Clear;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::io::stdout;
use std::sync::mpsc::Receiver;

use super::cell::Cell;
use super::entity::Entity;
use super::game_command::GameCommand;
use super::game_event::GameEvent;
use super::grid::Grid;
use crate::screen::screen::ScreenTransition;
use crate::utils::print_by_queue::PrintFullByQueue;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Board {
    pub cells: Vec<Vec<Cell>>,
    pub n: usize,
    pub m: usize,
    pub i: usize,
    pub j: usize,
    pub num_ok_box: usize, // number of boxes on targets
    pub num_box: usize,
}

pub type Solution = Vec<GameCommand>;

impl Board {
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
        let num_ok_box: usize = cells
            .iter()
            .map(|row| {
                row.iter()
                    .map(|c| match (c.entity, c.grid) {
                        (Some(Entity::Box), Grid::Target) => 1,
                        _ => 0,
                    })
                    .sum::<usize>()
            })
            .sum();
        let num_box: usize = cells
            .iter()
            .map(|row| {
                row.iter()
                    .map(|c| {
                        if let Some(Entity::Box) = c.entity {
                            1
                        } else {
                            0
                        }
                    })
                    .sum::<usize>()
            })
            .sum();
        match get_ij(&cells) {
            Ok((i, j)) => Self {
                cells,
                n,
                m,
                i,
                j,
                num_ok_box,
                num_box,
            },
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
                        if let Some(Entity::Box) = self.cells[i][j].entity {
                            self.num_ok_box = self
                                .num_ok_box
                                .overflowing_add(
                                    match (self.cells[i][j].grid, self.cells[ni][nj].grid) {
                                        (Grid::Ground, Grid::Target) => 1,
                                        (Grid::Target, Grid::Ground) => usize::MAX,
                                        _ => 0,
                                    },
                                )
                                .0;
                            if self.num_ok_box == self.num_box {
                                res.push(GameEvent::Win);
                            }
                        }
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

    pub fn is_finished(&self) -> bool {
        self.num_ok_box == self.num_box
    }

    pub fn execute(&mut self, command: GameCommand) -> (ScreenTransition, Vec<GameEvent>) {
        if self.is_finished() {
            return (ScreenTransition::Back, vec![]);
        }
        let events = match command {
            GameCommand::Up => self.push_entity((self.i, self.j), (usize::MAX, 0)),
            GameCommand::Down => self.push_entity((self.i, self.j), (1, 0)),
            GameCommand::Left => self.push_entity((self.i, self.j), (0, usize::MAX)),
            GameCommand::Right => self.push_entity((self.i, self.j), (0, 1)),
            _ => vec![],
        };
        (
            match command {
                GameCommand::Quit => ScreenTransition::Back,
                _ => ScreenTransition::Continue,
            },
            events,
        )
    }

    pub fn solve_interruptable(&self, r: Receiver<()>) -> Solution {
        // this has tons of improvement! so we start with BFS first.
        let mut que = VecDeque::new();
        let mut visited = HashSet::new();
        que.push_back((self.clone(), vec![]));
        while let Some((g, steps)) = que.pop_front() {
            if g.is_finished() {
                return steps;
            }
            if r.try_recv().is_ok() {
                return vec![];
            }
            if visited.contains(&g) {
                continue;
            }
            visited.insert(g.clone());
            for command in [
                GameCommand::Up,
                GameCommand::Down,
                GameCommand::Left,
                GameCommand::Right,
            ] {
                let mut new_g = g.clone();
                let _ = new_g.execute(command);
                if !visited.contains(&new_g) {
                    let mut new_steps = steps.clone();
                    new_steps.push(command);
                    que.push_back((new_g, new_steps));
                }
            }
        }
        vec![]
    }
}

impl From<&str> for Board {
    fn from(value: &str) -> Self {
        let rows = value.split('\n');
        Self::new(
            rows.into_iter()
                .map(|v| {
                    v.chars()
                        .into_iter()
                        .map(|c| match c {
                            '#' => Cell::new(Grid::Wall, None),
                            '@' => Cell::new(Grid::Ground, Some(Entity::Player)),
                            '$' => Cell::new(Grid::Ground, Some(Entity::Box)),
                            '.' => Cell::new(Grid::Target, None),
                            '+' => Cell::new(Grid::Target, Some(Entity::Player)),
                            '*' => Cell::new(Grid::Target, Some(Entity::Box)),
                            ' ' | _ => Cell::new(Grid::Ground, None),
                        })
                        .collect()
                })
                .collect(),
        )
    }
}

impl PrintFullByQueue for Board {
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