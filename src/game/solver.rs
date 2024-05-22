use super::board::Board;
use super::board_command::BoardCommand;
use super::entity::Entity;
use super::grid::Grid;
use std::cmp::Reverse;
use std::collections::hash_map::DefaultHasher;
use std::collections::BinaryHeap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::sync::mpsc::Receiver;

#[derive(Clone)]
struct DeltaBoard<'a> {
    g: &'a Board, // we only care about grids in it
    entity_vec: Vec<(usize, usize, Entity)>,
    n: usize,
    m: usize,
    i: usize,
    j: usize,
    num_ok_box: usize, // number of boxes on targets
    num_box: usize,
    entity_vec_hash: usize,
}

impl PartialEq for DeltaBoard<'_> {
    fn eq(&self, other: &Self) -> bool {
        // DeltaBoard would only be used as part of search state, thus we assume grids are always the same & skip
        // redundant checks
        self.entity_vec == other.entity_vec
    }
}

impl Eq for DeltaBoard<'_> {}

impl Debug for DeltaBoard<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ng = self.g.clone();
        for row in ng.cells.iter_mut() {
            for cell in row.iter_mut() {
                cell.entity = None;
            }
        }
        for (i, j, entity) in self.entity_vec.iter() {
            ng.cells[*i][*j].entity = Some(entity.to_owned());
        }
        writeln!(f, "Board [")?;
        for row in &ng.cells {
            for cell in row {
                write!(f, "{:?}", cell)?;
            }
            writeln!(f)?;
        }
        write!(f, "]")
    }
}

impl<'a> From<&'a Board> for DeltaBoard<'a> {
    fn from(value: &'a Board) -> Self {
        let entity_vec = value
            .cells
            .iter()
            .enumerate()
            .flat_map(|(i, row)| {
                row.iter()
                    .enumerate()
                    .map(move |(j, cell)| (i, j, cell.entity))
            })
            .filter_map(|(i, j, entity)| entity.map(|v| (i, j, v)))
            .collect::<Vec<_>>();
        let entity_vec_hash = {
            let mut s = DefaultHasher::new();
            entity_vec.hash(&mut s);
            s.finish() as usize
        };
        Self {
            g: value,
            entity_vec,
            n: value.n,
            m: value.m,
            i: value.i,
            j: value.j,
            num_ok_box: value.num_ok_box,
            num_box: value.num_box,
            entity_vec_hash,
        }
    }
}

impl Hash for DeltaBoard<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.entity_vec_hash.hash(state);
    }
}

impl DeltaBoard<'_> {
    fn is_finished(&self) -> bool {
        self.num_ok_box == self.num_box
    }

    fn get_grid_at(&self, i: usize, j: usize) -> Grid {
        self.g.cells[i][j].grid
    }

    fn get_entity_at(&self, i: usize, j: usize) -> Option<Entity> {
        self.entity_vec.iter().find_map(|(x, y, entity)| {
            if x == &i && y == &j {
                Some(entity.to_owned())
            } else {
                None
            }
        })
    }

    fn push_entity(&mut self, src: (usize, usize), d: (usize, usize)) -> bool {
        // returns true if any block has been pushed
        let (i, j) = src;
        let (di, dj) = d;
        let ni = i.overflowing_add(di).0;
        let nj = j.overflowing_add(dj).0;
        if ni < self.n && nj < self.m {
            match self.get_grid_at(ni, nj) {
                Grid::Ground | Grid::Target => {
                    if let Some(Entity::Box) = self.get_entity_at(ni, nj) {
                        let _ = self.push_entity((ni, nj), d);
                    }
                    if self.get_entity_at(ni, nj).is_none() {
                        if let Some(Entity::Box) = self.get_entity_at(i, j) {
                            self.num_ok_box = self
                                .num_ok_box
                                .overflowing_add(
                                    match (self.get_grid_at(i, j), self.get_grid_at(ni, nj)) {
                                        (Grid::Ground, Grid::Target) => 1,
                                        (Grid::Target, Grid::Ground) => usize::MAX,
                                        _ => 0,
                                    },
                                )
                                .0;
                        }
                        for (x, y, _entity) in self.entity_vec.iter_mut() {
                            if *x == i && *y == j {
                                *x = ni;
                                *y = nj;
                                break;
                            }
                        }
                        self.i = ni;
                        self.j = nj;
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            }
        } else {
            false
        }
    }

    fn execute(&mut self, command: BoardCommand) -> bool {
        let res = match command {
            BoardCommand::Up => self.push_entity((self.i, self.j), (usize::MAX, 0)),
            BoardCommand::Down => self.push_entity((self.i, self.j), (1, 0)),
            BoardCommand::Left => self.push_entity((self.i, self.j), (0, usize::MAX)),
            BoardCommand::Right => self.push_entity((self.i, self.j), (0, 1)),
            _ => false,
        };
        if res {
            let mut s = DefaultHasher::new();
            self.entity_vec.hash(&mut s);
            self.entity_vec_hash = s.finish() as usize;
        }
        res
    }
}

#[derive(PartialEq, Eq)]
struct State<'a> {
    g: DeltaBoard<'a>,
    steps: Vec<BoardCommand>, // not `Solution` for now!
    est_rest: usize, // A*, we use summed L1 distance to nearest goal to estimate the lowerbound
}

impl PartialOrd for State<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for State<'_> {
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
    insolvable: Vec<Vec<bool>>,
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

    fn calc_insolvable(g: &Board) -> Vec<Vec<bool>> {
        let mut visited = vec![vec![false; g.m]; g.n];
        // start pulling a virtual box at all target positions
        for (si, sj) in g
            .cells
            .iter()
            .enumerate()
            .flat_map(|(i, row)| row.iter().enumerate().map(move |(j, cell)| (i, j, cell)))
            .filter_map(|(i, j, cell)| {
                if let Grid::Target = cell.grid {
                    Some((i, j))
                } else {
                    None
                }
            })
        {
            let mut que = VecDeque::new();
            que.push_back((si, sj));
            while let Some((i, j)) = que.pop_front() {
                if visited[i][j] {
                    continue;
                }
                visited[i][j] = true;
                for (di, dj) in [(usize::MAX, 0), (1, 0), (0, usize::MAX), (0, 1)] {
                    // next position of box
                    let ni = i.overflowing_add(di).0;
                    let nj = j.overflowing_add(dj).0;
                    if ni < g.n
                        && nj < g.m
                        && matches!(g.cells[ni][nj].grid, Grid::Ground | Grid::Target)
                        && !visited[ni][nj]
                    {
                        // next position of puller
                        let nni = ni.overflowing_add(di).0;
                        let nnj = nj.overflowing_add(dj).0;
                        if nni < g.n
                            && nnj < g.m
                            && matches!(g.cells[nni][nnj].grid, Grid::Ground | Grid::Target)
                        {
                            que.push_back((ni, nj))
                        }
                    }
                }
            }
        }

        visited
            .into_iter()
            .map(|v| v.into_iter().map(|b| !b).collect())
            .collect()
    }

    pub fn new(g: &'a Board) -> Self {
        Self {
            board: g,
            min_dist_to_goal: Self::calc_min_dist_to_goal(g),
            insolvable: Self::calc_insolvable(g),
        }
    }

    fn calc_est_rest(&self, board: &DeltaBoard<'_>) -> Result<usize, ()> {
        let mut sum = 0;
        for (i, j, _entity) in board.entity_vec.iter() {
            match self.min_dist_to_goal[*i][*j] {
                Some(v) => sum += v,
                None => return Err(()), // box unreachable
            }
        }
        Ok(sum)
    }

    fn get_next_pushes(
        g: &DeltaBoard<'a>,
        r: &Option<Receiver<()>>,
    ) -> Vec<(DeltaBoard<'a>, Vec<BoardCommand>, BoardCommand)> {
        // figure out all possible one push next steps, i.e. closure of walk around
        // returns (State, command to push one box along one direction)
        let mut que = VecDeque::new();
        let mut visited = HashSet::new();
        let mut res = vec![];
        que.push_back((g.clone(), vec![]));
        // stop search when the length of `res` has the same number of empty grids around all boxes
        let target_len = g
            .entity_vec
            .iter()
            .filter(|(_i, _j, entity)| matches!(entity, Entity::Box))
            .map(|(i, j, _entity)| {
                [(usize::MAX, 0), (1, 0), (0, usize::MAX), (0, 1)]
                    .map(|(di, dj)| {
                        let ni = i.overflowing_add(di).0;
                        let nj = j.overflowing_add(dj).0;
                        if ni < g.n
                            && nj < g.m
                            && matches!(g.get_grid_at(ni, nj), Grid::Ground | Grid::Target)
                            && matches!(g.get_entity_at(ni, nj), Some(Entity::Player) | None)
                        {
                            1
                        } else {
                            0
                        }
                    })
                    .into_iter()
                    .sum::<usize>()
            })
            .sum::<usize>();

        while let Some((h, steps)) = que.pop_front() {
            if let Some(r) = r {
                if r.try_recv().is_ok() {
                    return vec![];
                }
            }
            if res.len() == target_len {
                break;
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
                if let Some(Entity::Box) = h.get_entity_at(ni, nj) {
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

    pub fn solve(&self, r: Option<Receiver<()>>) -> Result<Solution, String> {
        // basically A*
        let mut que = BinaryHeap::new();
        let mut visited = HashSet::new();
        let init_delta_board = self.board.into();
        let res_est_rest = self.calc_est_rest(&init_delta_board);
        if res_est_rest.is_err() {
            return Err("There exist a box such that it could never reach any goal".to_string());
        }
        que.push(Reverse(State {
            g: init_delta_board,
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
            if let Some(r) = &r {
                if r.try_recv().is_ok() {
                    return Err("Interrupted".to_string());
                }
            }
            if visited.contains(&h) {
                continue;
            }
            visited.insert(h.clone());

            for (mut new_h, mut new_steps, direction) in Self::get_next_pushes(&h, &r) {
                loop {
                    let player_moved = new_h.execute(direction);
                    new_steps.push(direction);
                    if !player_moved {
                        // we can't push anymore
                        break;
                    }
                    if new_h
                        .entity_vec
                        .iter()
                        .any(|(i, j, _entity)| self.insolvable[*i][*j])
                    {
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

#[cfg(test)]
mod tests {
    use super::Board;
    use super::BoardCommand;
    use super::DeltaBoard;
    use super::Solver;
    use std::collections::HashSet;

    #[test]
    fn test_execute_0() {
        let g = Board::from(
            "#######\n\
             #.$@$.#\n\
             #######",
        );
        let mut h = DeltaBoard::from(&g);
        h.execute(BoardCommand::Left);
        assert_eq!(
            h,
            (&Board::from(
                "#######\n\
                 #*@ $.#\n\
                 #######",
            ))
                .into()
        )
    }

    #[test]
    fn test_next_pushes_0() {
        let g = Board::from(
            "#######\n\
             #.$@$.#\n\
             #######",
        );
        let set = Solver::get_next_pushes(&(&g).into(), &None)
            .into_iter()
            .map(|(_b, _steps, dir)| dir)
            .collect::<HashSet<_>>();
        assert_eq!(
            set,
            HashSet::from([BoardCommand::Left, BoardCommand::Right])
        );
    }
    #[test]
    fn test_next_pushes_1() {
        let g = Board::from(
            "#######\n\
             #  .  #\n\
             #  $  #\n\
             #.$@$.#\n\
             #  $  #\n\
             #  .  #\n\
             #######",
        );
        let set = Solver::get_next_pushes(&(&g).into(), &None)
            .into_iter()
            .map(|(_b, _steps, dir)| dir)
            .collect::<HashSet<_>>();
        assert_eq!(
            set,
            HashSet::from([
                BoardCommand::Left,
                BoardCommand::Right,
                BoardCommand::Up,
                BoardCommand::Down
            ])
        );
    }
    #[test]
    fn test_next_pushes_2() {
        let g = Board::from(
            "#######\n\
             #  .$ #\n\
             #  @ .#\n\
             #    $#\n\
             #     #\n\
             #######",
        );
        let set = Solver::get_next_pushes(&(&g).into(), &None)
            .into_iter()
            .map(|(_b, _steps, dir)| dir)
            .collect::<HashSet<_>>();
        assert_eq!(
            set,
            HashSet::from([
                BoardCommand::Left,
                BoardCommand::Right,
                BoardCommand::Up,
                BoardCommand::Down
            ])
        );
    }

    #[test]
    fn test_solve_0() {
        let g = Board::from(
            "#######\n\
             #  .$ #\n\
             #  @ .#\n\
             #    $#\n\
             #     #\n\
             #######",
        );
        let solver = Solver::new(&g);
        assert_eq!(
            solver.solve(None).unwrap().seq,
            vec![
                BoardCommand::Right,
                BoardCommand::Right,
                BoardCommand::Up,
                BoardCommand::Left,
                BoardCommand::Down,
                BoardCommand::Down,
                BoardCommand::Down,
                BoardCommand::Right,
                BoardCommand::Up,
            ]
        );
    }

    #[test]
    fn test_insolvable_0() {
        let g = Board::from(
            "#######\n\
             #  .$ #\n\
             #  @ .#\n\
             #    $#\n\
             #     #\n\
             #######",
        );
        let insolvable = vec![
            vec![true; 7],
            vec![true, true, false, false, false, true, true],
            vec![true, true, false, false, false, false, true],
            vec![true, true, false, false, false, false, true],
            vec![true; 7],
            vec![true; 7],
        ];
        let solver = Solver::new(&g);
        assert_eq!(solver.insolvable, insolvable)
    }
}
