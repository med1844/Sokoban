extern crate sokoban;
mod screens;
use crate::screens::{
    exit_screen::ExitScreen,
    menu_screen::MenuScreen,
    screen::{Screen, ScreenTransition},
};
use crossterm::cursor::{Hide, Show};
use crossterm::event::{poll, read, Event};
use crossterm::execute;
use crossterm::terminal::{enable_raw_mode, Clear};
use glob::glob;
use screens::level_selector_screen::{FileLevel, LevelSelectorScreen};
use std::cell::RefCell;
use std::io::{stdout, Write};
use std::rc::Rc;
use std::thread;
use std::time::Duration;

fn next_event() -> Option<Event> {
    // if poll is None, none, else read
    match poll(Duration::from_secs(0)) {
        Ok(v) => match v {
            true => read().ok(),
            false => None,
        },
        Err(_) => None,
    }
}

struct GameApp {
    screens: Vec<Rc<RefCell<dyn Screen>>>,
}

impl GameApp {
    fn new(screen: Rc<RefCell<dyn Screen>>) -> Self {
        let mut res = Self {
            screens: vec![screen],
        };
        res.refresh_screen();
        res
    }

    fn refresh_screen(&mut self) {
        let _ = execute!(stdout(), Clear(crossterm::terminal::ClearType::All));
        let _ = self.screens.last().unwrap().as_ref().borrow().print_full();
        let _ = stdout().flush();
    }

    fn run(&mut self) {
        'main: loop {
            let transition = self
                .screens
                .last()
                .unwrap()
                .as_ref()
                .borrow_mut()
                .update(next_event());
            let _ = stdout().flush();
            match transition {
                ScreenTransition::Continue => {}
                ScreenTransition::Break => break 'main,
                ScreenTransition::SwitchTo(next_screen) => {
                    self.screens.push(next_screen);
                    self.refresh_screen();
                }
                ScreenTransition::Back => {
                    if !self.screens.is_empty() {
                        self.screens.pop();
                        self.refresh_screen();
                    } else {
                        break 'main;
                    }
                }
            }
            thread::sleep(Duration::from_secs_f64(1.0 / 144.0));
        }
    }
}

fn main() {
    // important init section
    let _ = enable_raw_mode();
    let _ = execute!(stdout(), Hide);

    let level_selector_screen = Rc::new(RefCell::new(LevelSelectorScreen::from(
        glob("levels/**/*.txt")
            .expect("failed to read glob pattern")
            .filter(|v| v.is_ok())
            .map(|v| {
                let s = v.ok().unwrap().to_str().unwrap().to_string();
                FileLevel {
                    level_name: s.clone(),
                    filename: s,
                }
            })
            .collect::<Vec<FileLevel>>(),
    )));
    let menu_screen = Rc::new(RefCell::new(MenuScreen::new(vec![
        ("Start".into(), level_selector_screen.clone()),
        ("Exit".into(), Rc::new(RefCell::new(ExitScreen::new()))),
    ])));
    let mut app = GameApp::new(menu_screen);
    app.run();
    let _ = execute!(stdout(), Show);
}
