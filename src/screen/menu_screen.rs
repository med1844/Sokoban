use std::cell::RefCell;
use std::io::stdout;
use std::rc::Rc;

use super::screen::{Screen, ScreenTransition};
use crate::utils::print_by_queue::PrintFullByQueue;
use crossterm::cursor::{MoveTo, MoveToNextLine};
use crossterm::event::{Event, KeyCode};
use crossterm::queue;
use crossterm::style::{Print, PrintStyledContent, Stylize};
use crossterm::terminal::Clear;

pub struct MenuScreen {
    pub options: Vec<(String, Rc<RefCell<dyn Screen>>)>,
    pub choice: usize,
}

impl MenuScreen {
    pub fn new(options: Vec<(String, Rc<RefCell<dyn Screen>>)>) -> Self {
        Self { options, choice: 0 }
    }
}

impl Screen for MenuScreen {
    fn handle_input(&mut self, event: Event) -> ScreenTransition {
        let old_choice = self.choice;
        match event {
            Event::Key(event) => match event.code {
                KeyCode::Up => self.choice = if self.choice == 0 { 0 } else { self.choice - 1 },
                KeyCode::Down => self.choice = (self.choice + 1).min(self.options.len() - 1),
                _ => {}
            },
            _ => {}
        };
        if self.choice != old_choice {
            let _ = queue!(
                stdout(),
                MoveTo(0, old_choice as u16),
                Print(self.options[old_choice].0.clone()),
            );
            let _ = queue!(
                stdout(),
                MoveTo(0, self.choice as u16),
                PrintStyledContent(self.options[self.choice].0.as_str().green()),
            );
        }
        match event {
            Event::Key(event) => match event.code {
                KeyCode::Char('q') => ScreenTransition::Break,
                KeyCode::Enter => ScreenTransition::SwitchTo(self.options[self.choice].1.clone()),
                _ => ScreenTransition::Continue,
            },
            _ => ScreenTransition::Continue,
        }
    }
}

impl PrintFullByQueue for MenuScreen {
    fn print_full(&self) -> Result<(), std::io::Error> {
        queue!(
            stdout(),
            Clear(crossterm::terminal::ClearType::All),
            MoveTo(0, 0)
        )?;
        for (desc, _) in self.options.iter() {
            queue!(stdout(), Print(desc), MoveToNextLine(1))?;
        }
        queue!(
            stdout(),
            MoveTo(0, self.choice as u16),
            PrintStyledContent(self.options[self.choice].0.as_str().green())
        )?;
        Ok(())
    }
}
