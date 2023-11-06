use crossterm::event::Event;
use crossterm::event::KeyCode;

pub enum GameCommand {
    Null,
    Up,
    Down,
    Left,
    Right,
    Quit,
}

impl From<Event> for GameCommand {
    fn from(value: Event) -> Self {
        match value {
            Event::Key(value) => match value.code {
                KeyCode::Left => Self::Left,
                KeyCode::Right => Self::Right,
                KeyCode::Up => Self::Up,
                KeyCode::Down => Self::Down,
                KeyCode::Char('q') => Self::Quit,
                _ => Self::Null,
            },
            _ => Self::Null,
        }
    }
}
