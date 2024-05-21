use super::cell::Cell;

pub enum BoardEvent {
    Put(usize, usize, Cell),
    Win,
}
