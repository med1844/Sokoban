use super::cell::Cell;
use super::entity::Entity;
use super::game_command::GameCommand;
use super::game_event::GameEvent;
use super::grid::Grid;
use crate::screens::screen::ScreenTransition;
use crate::utils::print_by_queue::PrintFullByQueue;
use crossterm::cursor::{MoveTo, MoveToNextLine};
use crossterm::queue;
use crossterm::terminal::Clear;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::io::stdout;
use std::sync::mpsc::Receiver;

#[derive(Clone)]
pub struct Solution {
    pub seq: Vec<GameCommand>,
    pub visited_states: usize,
}

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

impl Board {
    pub fn new(cells: Vec<Vec<Cell>>) -> Self {
        let n = cells.len();
        let m = cells.first().unwrap_or(&vec![]).len();
        fn get_ij(cells: &[Vec<Cell>]) -> Result<(usize, usize), &str> {
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
                        res.append(&mut self.push_entity((ni, nj), d));
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
                        res.push(GameEvent::Put(i, j, self.cells[i][j]));
                        res.push(GameEvent::Put(ni, nj, self.cells[ni][nj]));
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

    pub fn solve_interruptable(&self, r: Receiver<()>) -> Result<Solution, String> {
        #[derive(PartialEq, Eq)]
        struct State {
            g: Board,
            steps: Vec<GameCommand>, // not `Solution` for now!
            est_rest: usize, // A*, we use summed L1 distance to nearest goal to estimate the lowerbound
        }

        impl PartialOrd for State {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                (self.steps.len() + self.est_rest)
                    .partial_cmp(&(other.steps.len() + other.est_rest))
            }
        }

        impl Ord for State {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                (self.steps.len() + self.est_rest).cmp(&(other.steps.len() + other.est_rest))
            }
        }

        fn calc_min_dist_to_goal(g: &Board) -> Vec<Vec<Option<usize>>> {
            // returns an array the same size as the game board, where at i, j, it stores the L1 distance to the
            // nearest goal
            let mut que = VecDeque::new();
            let mut res = vec![vec![None; g.m]; g.n];
            for (i, row) in g.cells.iter().enumerate() {
                for (j, cell) in row.iter().enumerate() {
                    if let Grid::Target = cell.grid {
                        que.push_back(((i, j), 0));
                    }
                }
            }
            while let Some(((i, j), d)) = que.pop_front() {
                if res[i][j].is_some() {
                    // updated by previous visits
                    continue;
                }
                res[i][j] = Some(d);
                for (di, dj) in [(1, 0), (0, 1), (usize::MAX, 0), (0, usize::MAX)] {
                    let ni = i.overflowing_add(di).0;
                    let nj = j.overflowing_add(dj).0;
                    if ni < g.n
                        && nj < g.m
                        && res[ni][nj].is_none()
                        && matches!(g.cells[ni][nj].grid, Grid::Ground | Grid::Target)
                    {
                        que.push_back(((ni, nj), d + 1));
                    }
                }
            }
            res
        }

        fn calc_est_rest(
            g: &Board,
            min_dist_to_goal: &Vec<Vec<Option<usize>>>,
        ) -> Result<usize, ()> {
            let mut sum = 0;
            for (i, row) in g.cells.iter().enumerate() {
                for (j, cell) in row.iter().enumerate() {
                    if let Some(Entity::Box) = cell.entity {
                        match min_dist_to_goal[i][j] {
                            Some(v) => sum += v,
                            None => return Err(()), // box unreachable
                        }
                    }
                }
            }
            Ok(sum)
        }

        let min_dist_to_goal = calc_min_dist_to_goal(self);

        let mut que = BinaryHeap::new();
        let mut visited = HashSet::new();
        let res_est_rest = calc_est_rest(self, &min_dist_to_goal);
        if res_est_rest.is_err() {
            return Err("There exist a box such that it could never reach any goal".to_string());
        }
        que.push(Reverse(State {
            g: self.clone(),
            steps: vec![],
            est_rest: res_est_rest.unwrap(),
        }));
        while let Some(Reverse(State { g, steps, .. })) = que.pop() {
            if g.is_finished() {
                return Ok(Solution {
                    seq: steps,
                    visited_states: visited.len(),
                });
            }
            if r.try_recv().is_ok() {
                return Err("Interrupted".to_string());
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
                    que.push(Reverse(State {
                        g: new_g,
                        steps: new_steps,
                        est_rest: calc_est_rest(self, &min_dist_to_goal).unwrap(),
                    }));
                }
            }
        }
        Err("The program reached some impossible branch".to_string())
    }
}

impl From<&str> for Board {
    fn from(value: &str) -> Self {
        let rows = value.split('\n');
        Self::new(
            rows.into_iter()
                .map(|v| {
                    v.chars()
                        .map(|c| match c {
                            '#' => Cell::new(Grid::Wall, None),
                            '@' => Cell::new(Grid::Ground, Some(Entity::Player)),
                            '$' => Cell::new(Grid::Ground, Some(Entity::Box)),
                            '.' => Cell::new(Grid::Target, None),
                            '+' => Cell::new(Grid::Target, Some(Entity::Player)),
                            '*' => Cell::new(Grid::Target, Some(Entity::Box)),
                            _ => Cell::new(Grid::Ground, None),
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
