use super::cell::Cell;

pub enum GameEvent {
    Put(usize, usize, Cell),
}
