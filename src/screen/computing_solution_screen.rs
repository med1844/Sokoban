use super::screen::{Screen, ScreenTransition};
use super::solver_screen::SolverScreen;
use crate::game::game::Game;
use crate::utils::print_by_queue::PrintFullByQueue;
use crossterm::cursor::{MoveTo, MoveToNextLine};
use crossterm::event::{Event, KeyCode, KeyEvent};
use crossterm::queue;
use crossterm::style::{PrintStyledContent, Stylize};
use std::cell::RefCell;
use std::io::stdout;
use std::rc::Rc;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;

enum Status {
    Computing,
    Ok,
    Err,
}

pub struct ComputingSolutionScreen {
    pub sender: Sender<()>,
    pub handle: Option<thread::JoinHandle<Arc<SolverScreen>>>,
    status: Status,
}

impl ComputingSolutionScreen {
    pub fn new(sender: Sender<()>, handle: thread::JoinHandle<Arc<SolverScreen>>) -> Self {
        Self {
            sender,
            handle: Some(handle),
            status: Status::Computing,
        }
    }
}

impl PrintFullByQueue for ComputingSolutionScreen {
    fn print_full(&self) -> Result<(), std::io::Error> {
        queue!(
            stdout(),
            MoveTo(0, 0),
            PrintStyledContent(
                "Computing optimal solution, press <q> to cancel"
                    .grey()
                    .italic()
            )
        )
    }
}

impl Screen for ComputingSolutionScreen {
    fn update(&mut self, event: Option<Event>) -> ScreenTransition {
        match self.status {
            Status::Computing => {
                if let Some(handle) = &self.handle {
                    if handle.is_finished() {
                        if let Some(handle) = self.handle.take() {
                            let res = handle.join();
                            match res {
                                Ok(arc_screen) => {
                                    let screen = arc_screen.as_ref().clone();
                                    self.status = Status::Ok;
                                    return ScreenTransition::SwitchTo(Rc::new(RefCell::new(
                                        screen,
                                    )));
                                }
                                Err(_) => self.status = Status::Err,
                            }
                        }
                    }
                }
            }
            Status::Ok => return ScreenTransition::Back,
            Status::Err => {
                let _ = queue!(
                    stdout(),
                    PrintStyledContent(
                        "The solver subprocess failed, please press <q> to return to the game"
                            .red()
                            .bold()
                    )
                );
            }
        }
        match event {
            Some(Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            })) => {
                if let Status::Computing = self.status {
                    let _ = self.sender.send(());
                }
                ScreenTransition::Back
            }
            _ => ScreenTransition::Continue,
        }
    }
}
