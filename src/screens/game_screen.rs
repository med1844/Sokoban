use super::computing_solution_screen::ComputingSolutionScreen;
use super::screen::{Screen, ScreenTransition};
use super::solver_screen::SolverScreen;
use crate::game::board::Board;
use crate::game::game_event::GameEvent;
use crate::utils::print_by_queue::PrintFullByQueue;
use crossterm::cursor::MoveTo;
use crossterm::event::{Event, KeyCode, KeyEvent};
use crossterm::queue;
use crossterm::style::Print;
use std::cell::RefCell;
use std::io::stdout;
use std::rc::Rc;
use std::sync::{mpsc, Arc};
use std::thread;

#[derive(Clone)]
pub struct GameScreen {
    pub g: Board,
}

impl GameScreen {
    pub fn new(g: Board) -> Self {
        Self { g }
    }
}

impl Screen for GameScreen {
    fn update(&mut self, event: Option<Event>) -> ScreenTransition {
        match event {
            Some(event) => {
                if let Event::Key(KeyEvent {
                    code: KeyCode::Char('o'),
                    ..
                }) = event
                {
                    let (sender, receiver) = mpsc::channel();
                    let g = self.g.clone();
                    let handle = thread::spawn(move || {
                        let solution = g.solve_interruptable(receiver);
                        Arc::new(SolverScreen::new(g, solution))
                    });
                    return ScreenTransition::SwitchTo(Rc::new(RefCell::new(
                        ComputingSolutionScreen::new(sender, handle),
                    )));
                }
                // TODO refactor execute to not return screen transition
                let (transition, events) = self.g.execute(event.into());
                // to reduce dependency & support increment printing, we use GameEvents to capture game
                // internal changes, and let Screens utilize these events.
                for event in events.iter() {
                    match event {
                        GameEvent::Put(i, j, cell) => {
                            let _ = queue!(stdout(), MoveTo(*j as u16, *i as u16));
                            let _ = cell.print_full();
                        }
                        GameEvent::Win => {
                            let _ = queue!(
                                stdout(),
                                Print("You win! Press any key to exit this level")
                            );
                        }
                    }
                }
                transition
            }
            None => ScreenTransition::Continue,
        }
    }
}

impl PrintFullByQueue for GameScreen {
    fn print_full(&self) -> Result<(), std::io::Error> {
        self.g.print_full()?;
        Ok(())
    }
}
