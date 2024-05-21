use super::board::Board;
use super::board_command::BoardCommand;
use super::entity::Entity;
use super::grid::Grid;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::sync::mpsc::Receiver;

#[derive(PartialEq, Eq)]
struct State {
    g: Board,
    steps: Vec<BoardCommand>, // not `Solution` for now!
    est_rest: usize, // A*, we use summed L1 distance to nearest goal to estimate the lowerbound
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (self.steps.len() + self.est_rest).partial_cmp(&(other.steps.len() + other.est_rest))
    }
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.steps.len() + self.est_rest).cmp(&(other.steps.len() + other.est_rest))
    }
}

#[derive(Clone)]
pub struct Solution {
    pub seq: Vec<BoardCommand>,
    pub visited_states: usize,
}

pub struct Solver<'a> {
    board: &'a Board,
    min_dist_to_goal: Vec<Vec<Option<usize>>>,
}

impl<'a> Solver<'a> {
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

    pub fn new(g: &'a Board) -> Self {
        Self {
            board: g,
            min_dist_to_goal: Self::calc_min_dist_to_goal(g),
        }
    }

    fn calc_est_rest(&self, board: &Board) -> Result<usize, ()> {
        let mut sum = 0;
        for (i, row) in board.cells.iter().enumerate() {
            for (j, cell) in row.iter().enumerate() {
                if let Some(Entity::Box) = cell.entity {
                    match self.min_dist_to_goal[i][j] {
                        Some(v) => sum += v,
                        None => return Err(()), // box unreachable
                    }
                }
            }
        }
        Ok(sum)
    }

    fn get_next_pushes(
        g: &Board,
        r: &Receiver<()>,
    ) -> Vec<(Board, Vec<BoardCommand>, BoardCommand)> {
        // figure out all possible one push next steps, i.e. closure of walk around
        // returns (State, command to push one box along one direction)
        let mut que = VecDeque::new();
        let mut visited = HashSet::new();
        let mut res = vec![];
        que.push_back((g.clone(), vec![]));
        while let Some((h, steps)) = que.pop_front() {
            if r.try_recv().is_ok() {
                return vec![];
            }
            if visited.contains(&h) {
                continue;
            }
            visited.insert(h.clone());
            for (command, (di, dj)) in [
                BoardCommand::Up,
                BoardCommand::Down,
                BoardCommand::Left,
                BoardCommand::Right,
            ]
            .into_iter()
            .zip([(usize::MAX, 0), (1, 0), (0, usize::MAX), (0, 1)])
            {
                let ni = h.i.overflowing_add(di).0;
                let nj = h.j.overflowing_add(dj).0;
                if let Some(Entity::Box) = h.cells[ni][nj].entity {
                    res.push((h.clone(), steps.clone(), command));
                } else {
                    let mut new_g = h.clone();
                    let _ = new_g.execute(command);
                    if !visited.contains(&new_g) {
                        let mut new_steps = steps.clone();
                        new_steps.push(command);
                        que.push_back((new_g, new_steps));
                    }
                }
            }
        }
        res
    }

    pub fn solve(&self, r: Receiver<()>) -> Result<Solution, String> {
        // basically A*
        let mut que = BinaryHeap::new();
        let mut visited = HashSet::new();
        let res_est_rest = self.calc_est_rest(self.board);
        if res_est_rest.is_err() {
            return Err("There exist a box such that it could never reach any goal".to_string());
        }
        que.push(Reverse(State {
            g: self.board.clone(),
            steps: vec![],
            est_rest: res_est_rest.unwrap(),
        }));
        while let Some(Reverse(State { g: h, steps, .. })) = que.pop() {
            if h.is_finished() {
                return Ok(Solution {
                    seq: steps,
                    visited_states: visited.len(),
                });
            }
            if r.try_recv().is_ok() {
                return Err("Interrupted".to_string());
            }
            if visited.contains(&h) {
                continue;
            }
            visited.insert(h.clone());

            for (mut new_h, mut new_steps, direction) in Self::get_next_pushes(&h, &r) {
                loop {
                    let res = new_h.execute(direction);
                    new_steps.push(direction);
                    if res.is_empty() {
                        // we pushed to the end
                        break;
                    }
                    if visited.contains(&new_h) {
                        continue;
                    }
                    que.push(Reverse(State {
                        g: new_h.clone(),
                        steps: {
                            let mut concated = steps.clone();
                            concated.append(&mut new_steps.clone());
                            concated
                        },
                        est_rest: self.calc_est_rest(&new_h).unwrap(),
                    }))
                }
            }
        }
        Err("The program reached some impossible branch".to_string())
    }
}
