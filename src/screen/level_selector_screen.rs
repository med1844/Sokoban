use std::{cell::RefCell, rc::Rc};

use super::game_screen::GameScreen;

pub struct LevelSelectorScreen {
    game_filenames: Vec<(String, fn() -> Rc<RefCell<GameScreen>>)>, // level name, game builder functions
}

// impl LevelSelectorScreen {
//     pub fn new()
// }
