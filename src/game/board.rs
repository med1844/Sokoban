use super::board_command::BoardCommand;
use super::board_event::BoardEvent;
use super::cell::Cell;
use super::entity::Entity;
use super::grid::Grid;

#[derive(Debug, Clone)]
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

    #[inline]
    pub fn get_next(src: (usize, usize), d: (usize, usize)) -> (usize, usize) {
        let (i, j) = src;
        let (di, dj) = d;
        let ni = i.overflowing_add(di).0;
        let nj = j.overflowing_add(dj).0;
        (ni, nj)
    }

    #[inline]
    pub fn pos_is_valid(&self, i: usize, j: usize) -> bool {
        i < self.n && j < self.m
    }

    pub fn push_entity(&mut self, src: (usize, usize), d: (usize, usize)) -> Vec<BoardEvent> {
        let (i, j) = src;
        let (ni, nj) = Self::get_next(src, d);
        let mut res = vec![];
        if self.pos_is_valid(ni, nj)
            && matches!(self.cells[ni][nj].grid, Grid::Ground | Grid::Target)
        {
            if let Some(Entity::Box) = self.cells[ni][nj].entity {
                let (nni, nnj) = Self::get_next((ni, nj), d);
                if self.pos_is_valid(nni, nnj)
                    && self.cells[nni][nnj].entity.is_none()
                    && matches!(self.cells[nni][nnj].grid, Grid::Ground | Grid::Target)
                {
                    self.cells[nni][nnj].entity = self.cells[ni][nj].entity.take();
                    self.num_ok_box = self
                        .num_ok_box
                        .overflowing_add(
                            match (self.cells[ni][nj].grid, self.cells[nni][nnj].grid) {
                                (Grid::Ground, Grid::Target) => 1,
                                (Grid::Target, Grid::Ground) => usize::MAX,
                                _ => 0,
                            },
                        )
                        .0;
                    if self.num_ok_box == self.num_box {
                        res.push(BoardEvent::Win);
                    }
                    res.push(BoardEvent::Put(nni, nnj, self.cells[nni][nnj]));
                }
            }
            if self.cells[ni][nj].entity.is_none() {
                self.cells[ni][nj].entity = self.cells[i][j].entity.take();
                res.push(BoardEvent::Put(i, j, self.cells[i][j]));
                res.push(BoardEvent::Put(ni, nj, self.cells[ni][nj]));
                self.i = ni;
                self.j = nj;
            }
        }
        res
    }

    pub fn is_finished(&self) -> bool {
        self.num_ok_box == self.num_box
    }

    pub fn execute(&mut self, command: BoardCommand) -> Vec<BoardEvent> {
        if self.is_finished() {
            return vec![];
        }
        match command {
            BoardCommand::Up => self.push_entity((self.i, self.j), (usize::MAX, 0)),
            BoardCommand::Down => self.push_entity((self.i, self.j), (1, 0)),
            BoardCommand::Left => self.push_entity((self.i, self.j), (0, usize::MAX)),
            BoardCommand::Right => self.push_entity((self.i, self.j), (0, 1)),
            _ => vec![],
        }
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
