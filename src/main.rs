use std::cell::RefCell;
use std::io::{stdout, Write};
use std::rc::Rc;

use crossterm::cursor::{Hide, Show};
use crossterm::event::read;
use crossterm::execute;
use crossterm::terminal::enable_raw_mode;

mod game;
mod screen;
mod utils;

use crate::game::game::Game;
use crate::screen::{
    game_screen::GameScreen,
    menu_screen::MenuScreen,
    screen::{Screen, ScreenTransition},
};

struct GameApp {
    cur_screen: Rc<RefCell<dyn Screen>>,
}

impl GameApp {
    fn new(screen: Rc<RefCell<dyn Screen>>) -> Self {
        let _ = screen.as_ref().borrow().print_full();
        let _ = stdout().flush();
        Self { cur_screen: screen }
    }

    fn run(&mut self) {
        loop {
            let transition = self
                .cur_screen
                .as_ref()
                .borrow_mut()
                .handle_input(read().unwrap());
            let _ = stdout().flush();
            match transition {
                ScreenTransition::Continue => {}
                ScreenTransition::Break => break,
                ScreenTransition::SwitchTo(next_screen) => {
                    self.cur_screen = next_screen;
                    let _ = self.cur_screen.as_ref().borrow().print_full();
                    let _ = stdout().flush();
                }
            }
        }
    }
}

fn main() {
    let _ = enable_raw_mode();
    let _ = execute!(stdout(), Hide);
    //     let g = Game::from(
    //         "     ####
    // ######  #
    // #       #
    // #      .#
    // #@ #######
    // ##       #
    //  # # #   #
    //  #     $ #
    //  #   #####
    //  #####    ",
    //     );
    let g = Game::from(
        "#####  ####  #####
#   ####  ####   #
#                #
##        ###   ##
 ## $  # .. $ @##
##  ##   ####   ##
#                #
#   ##########   #
#####        #####",
    );
    let game_screen = Rc::new(RefCell::new(GameScreen::new(g)));
    let menu_screen = Rc::new(RefCell::new(MenuScreen::new(vec![(
        "Go Game".into(),
        game_screen.clone(),
    )])));
    let mut app = GameApp::new(menu_screen);
    app.run();
    let _ = execute!(stdout(), Show);
}
