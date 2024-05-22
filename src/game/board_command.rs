use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum BoardCommand {
    Null,
    Up,
    Down,
    Left,
    Right,
}

impl From<KeyEvent> for BoardCommand {
    fn from(value: KeyEvent) -> Self {
        match value.code {
            KeyCode::Left => Self::Left,
            KeyCode::Right => Self::Right,
            KeyCode::Up => Self::Up,
            KeyCode::Down => Self::Down,
            _ => Self::Null,
        }
    }
}

impl From<Event> for BoardCommand {
    fn from(value: Event) -> Self {
        match value {
            Event::Key(value) => value.into(),
            _ => Self::Null,
        }
    }
}
