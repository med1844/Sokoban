use crossterm::cursor::MoveTo;
use crossterm::event::{Event, KeyCode, KeyEvent};
use crossterm::queue;
use crossterm::style::Print;

use super::screen::{Screen, ScreenTransition};
use crate::game::game::Game;
use crate::game::game_event::GameEvent;
use crate::utils::print_by_queue::PrintFullByQueue;
use std::io::stdout;

pub struct GameScreen {
    g: Game,
}

impl GameScreen {
    pub fn new(g: Game) -> Self {
        Self { g }
    }
}

impl Screen for GameScreen {
    fn update(&mut self, event: Option<Event>) -> ScreenTransition {
        match event {
            Some(event) => {
                if let Event::Key(KeyEvent { code, .. }) = event {
                    if let KeyCode::Char('o') = code {}
                }
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
