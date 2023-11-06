use crossterm::cursor::MoveTo;
use crossterm::event::Event;
use crossterm::queue;

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
    fn handle_input(&mut self, event: Event) -> ScreenTransition {
        let (transition, events) = self.g.execute(event.into());
        // to reduce dependency & support increment printing, we use GameEvents to capture game
        // internal changes, and let Screens utilize these events.
        for event in events.iter() {
            match event {
                GameEvent::Put(i, j, cell) => {
                    let _ = queue!(stdout(), MoveTo(*j as u16, *i as u16));
                    let _ = cell.print_full();
                }
            }
        }
        transition
    }
}

impl PrintFullByQueue for GameScreen {
    fn print_full(&self) -> Result<(), std::io::Error> {
        self.g.print_full()?;
        Ok(())
    }
}
