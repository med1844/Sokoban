use super::game_screen::GameScreen;
use super::screen::{Screen, ScreenTransition};
use crate::{
    game::{
        board::{Board, Solution},
        game_command::GameCommand,
    },
    utils::print_by_queue::PrintFullByQueue,
};
use crossterm::cursor::{MoveTo, MoveToNextLine};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use crossterm::queue;
use crossterm::style::{PrintStyledContent, Stylize};
use crossterm::terminal::{Clear, ClearType};
use std::io::stdout;

#[derive(Clone)]
pub struct SolverScreen {
    pub origin_game: Board,
    pub game_screen: GameScreen,
    pub sol: Vec<GameCommand>,
    pub cur: usize,
    pub play: bool,
    print_per_n_updates: u8,
    cur_update: u8,
}

impl SolverScreen {
    pub fn new(game: Board, sol: Solution) -> Self {
        Self {
            origin_game: game.clone(),
            game_screen: GameScreen::new(game),
            sol,
            cur: 0,
            play: false,
            print_per_n_updates: 16,
            cur_update: 0,
        }
    }
}

impl PrintFullByQueue for SolverScreen {
    fn print_full(&self) -> Result<(), std::io::Error> {
        self.game_screen.print_full()?;
        queue!(
            stdout(),
            MoveToNextLine(1),
            if self.play {
                PrintStyledContent("Playing".green())
            } else {
                PrintStyledContent("Paused".grey())
            },
            MoveToNextLine(1),
            PrintStyledContent("Press <space> to start/pause playback".dark_grey().italic()),
            MoveToNextLine(1),
            PrintStyledContent("Press <q> to return to game play".dark_grey().italic()),
            MoveToNextLine(1),
            PrintStyledContent("Press <r> to restart".dark_grey().italic()),
        )?;
        Ok(())
    }
}

impl Screen for SolverScreen {
    fn update(
        &mut self,
        event: Option<crossterm::event::Event>,
    ) -> super::screen::ScreenTransition {
        let default_key_event = KeyEvent {
            code: KeyCode::Null,
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
                KeyCode::Char('r') => {
                    self.cur = 0;
                    self.cur_update = 0;
                    self.game_screen.g = self.origin_game.clone();
                    let _ = queue!(stdout(), Clear(ClearType::All));
                    let _ = self.print_full();
                    ScreenTransition::Continue
                }
                _ => ScreenTransition::Continue,
            },
            _ => {
                if self.play && self.cur < self.sol.len() {
                    if self.cur_update == 0 {
                        self.game_screen.update(match self.sol[self.cur] {
                            // `game_screen` only takes in `crossterm::event::Event`, thus we have to reconstruct it...
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
                                code: KeyCode::Null,
                                ..default_key_event
                            })),
                        });
                        self.cur += 1;
                    }
                    self.cur_update += 1;
                    if self.cur_update >= self.print_per_n_updates {
                        self.cur_update -= self.print_per_n_updates;
                    }
                }
                // update play status
                let h = self.game_screen.g.n;
                let _ = queue!(stdout(), MoveTo(0, h as u16), MoveToNextLine(1));
                let _ = queue!(
                    stdout(),
                    if self.play && self.cur < self.sol.len() {
                        PrintStyledContent("Playing".green())
                    } else {
                        PrintStyledContent("Paused ".grey())
                    }
                );
                ScreenTransition::Continue
            }
        }
    }
}