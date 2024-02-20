use crate::{
    game::{game::Game, game_command::GameCommand},
    utils::print_by_queue::PrintFullByQueue,
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

use super::game_screen::GameScreen;
use super::screen::{Screen, ScreenTransition};

pub struct SolverScreen {
    game_screen: GameScreen,
    sol: Vec<GameCommand>,
    cur: usize,
    play: bool,
}

impl From<Game> for SolverScreen {
    fn from(value: Game) -> Self {
        Self {
            game_screen: GameScreen::new(value),
            sol: vec![
                GameCommand::Up,
                GameCommand::Right,
                GameCommand::Down,
                GameCommand::Left,
            ],
            cur: 0,
            play: false,
        }
    }
}

impl PrintFullByQueue for SolverScreen {
    fn print_full(&self) -> Result<(), std::io::Error> {
        self.game_screen.print_full()
    }
}

impl Screen for SolverScreen {
    fn update(
        &mut self,
        event: Option<crossterm::event::Event>,
    ) -> super::screen::ScreenTransition {
        let default_key_event = KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        match event {
            Some(Event::Key(KeyEvent { code, .. })) => match code {
                KeyCode::Char(' ') => {
                    self.play ^= true;
                    ScreenTransition::Continue
                }
                KeyCode::Char('q') => ScreenTransition::Back,
                _ => ScreenTransition::Continue,
            },
            _ => {
                if self.play {
                    self.game_screen.update(match self.sol[self.cur] {
                        GameCommand::Up => Some(Event::Key(KeyEvent {
                            code: KeyCode::Up,
                            ..default_key_event
                        })),
                        GameCommand::Down => Some(Event::Key(KeyEvent {
                            code: KeyCode::Down,
                            ..default_key_event
                        })),
                        GameCommand::Left => Some(Event::Key(KeyEvent {
                            code: KeyCode::Left,
                            ..default_key_event
                        })),
                        GameCommand::Right => Some(Event::Key(KeyEvent {
                            code: KeyCode::Right,
                            ..default_key_event
                        })),
                        _ => Some(Event::Key(KeyEvent {
                            code: KeyCode::Char(' '),
                            ..default_key_event
                        })),
                    });
                }
                ScreenTransition::Continue
            }
        }
    }
}
