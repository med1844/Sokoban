use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum GameCommand {
    Null,
    Up,
    Down,
    Left,
    Right,
    Quit,
}

impl From<KeyEvent> for GameCommand {
    fn from(value: KeyEvent) -> Self {
        match value.code {
            KeyCode::Left => Self::Left,
            KeyCode::Right => Self::Right,
            KeyCode::Up => Self::Up,
            KeyCode::Down => Self::Down,
            KeyCode::Char('q') => Self::Quit,
            _ => Self::Null,
        }
    }
}

impl From<Event> for GameCommand {
    fn from(value: Event) -> Self {
        match value {
            Event::Key(value) => value.into(),
            _ => Self::Null,
        }
    }
}
