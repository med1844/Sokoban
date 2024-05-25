use super::computing_solution_screen::ComputingSolutionScreen;
use super::screen::{Screen, ScreenTransition};
use super::solver_screen::SolverScreen;
use crossterm::cursor::{MoveTo, MoveToNextLine};
use crossterm::event::{Event, KeyCode, KeyEvent};
use crossterm::queue;
use crossterm::style::Print;
use crossterm::terminal::Clear;
use sokoban::game::board::Board;
use sokoban::game::board_event::BoardEvent;
use sokoban::game::solver::Solver;
use sokoban::utils::print_by_queue::PrintFullByQueue;
use std::cell::RefCell;
use std::io::stdout;
use std::rc::Rc;
use std::sync::{mpsc, Arc};
use std::thread;

#[derive(Clone)]
pub struct BoardScreen {
    pub g: Board,
}

impl BoardScreen {
    pub fn new(g: Board) -> Self {
        Self { g }
    }
}

impl PrintFullByQueue for BoardScreen {
    fn print_full(&self) -> Result<(), std::io::Error> {
        queue!(
            stdout(),
            Clear(crossterm::terminal::ClearType::All),
            MoveTo(0, 0)
        )?;
        for row in self.g.cells.iter() {
            for cell in row.iter() {
                cell.print_full()?;
            }
            queue!(stdout(), MoveToNextLine(1))?;
        }
        Ok(())
    }
}

impl Screen for BoardScreen {
    fn update(&mut self, event: Option<Event>) -> ScreenTransition {
        match event {
            Some(Event::Key(KeyEvent {
                code: KeyCode::Char('o'),
                ..
            })) => {
                let (sender, receiver) = mpsc::channel();
                let g = self.g.clone();
                let handle = thread::spawn(move || {
                    let solver = Solver::new(g.clone());
                    let solution = solver.solve(Some(receiver));
                    Arc::new(SolverScreen::new(g, solution))
                });
                ScreenTransition::SwitchTo(Rc::new(RefCell::new(ComputingSolutionScreen::new(
                    sender, handle,
                ))))
            }
            Some(Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            })) => ScreenTransition::Back,
            Some(event) => {
                // TODO refactor execute to not return screen transition
                let events = self.g.execute(event.into());
                // to reduce dependency & support increment printing, we use GameEvents to capture game
                // internal changes, and let Screens utilize these events.
                for event in events.iter() {
                    match event {
                        BoardEvent::Put(i, j, cell) => {
                            let _ = queue!(stdout(), MoveTo(*j as u16, *i as u16));
                            let _ = cell.print_full();
                        }
                        BoardEvent::Win => {
                            let _ = queue!(
                                stdout(),
                                Print("You win! Press any key to exit this level")
                            );
                        }
                    }
                }
                ScreenTransition::Continue
            }
            None => ScreenTransition::Continue,
        }
    }
}
