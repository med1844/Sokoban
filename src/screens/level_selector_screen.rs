use super::{
    game_screen::BoardScreen,
    screen::{Screen, ScreenTransition},
};
use crossterm::cursor::{MoveTo, MoveToNextLine};
use crossterm::event::{Event, KeyCode};
use crossterm::queue;
use crossterm::style::{Print, PrintStyledContent, Stylize};
use sokoban::utils::print_by_queue::PrintFullByQueue;
use std::{
    cell::RefCell,
    fs::File,
    io::{self, stdout, Read},
    rc::Rc,
};

type LevelLoader = Box<dyn Fn() -> Result<Rc<RefCell<BoardScreen>>, String>>;

pub struct LevelSelectorScreen {
    levels: Vec<(String, LevelLoader)>,
    cur: usize,
}

pub struct FileLevel {
    pub level_name: String,
    pub filename: String,
}

fn load_file(filename: String) -> io::Result<String> {
    let mut f = File::open(filename)?;
    let mut res = String::new();
    f.read_to_string(&mut res)?;
    Ok(res)
}

impl From<Vec<FileLevel>> for LevelSelectorScreen {
    fn from(value: Vec<FileLevel>) -> Self {
        let levels = value
            .into_iter()
            .map(|file_level| {
                let filename = file_level.filename;
                let loader: LevelLoader = Box::new(move || {
                    load_file(filename.clone())
                        .map(|val| Rc::new(RefCell::new(BoardScreen::new(val.as_str().into()))))
                        .map_err(|err| err.to_string())
                });
                (file_level.level_name, loader)
            })
            .collect();
        Self { levels, cur: 0 }
    }
}

impl PrintFullByQueue for LevelSelectorScreen {
    fn print_full(&self) -> Result<(), std::io::Error> {
        queue!(stdout(), MoveTo(0, 0))?;
        for (name, _) in self.levels.iter() {
            queue!(stdout(), Print(name), MoveToNextLine(1))?;
        }
        queue!(
            stdout(),
            MoveTo(0, self.cur as u16),
            PrintStyledContent(self.levels[self.cur].0.clone().green())
        )?;
        Ok(())
    }
}

impl Screen for LevelSelectorScreen {
    fn update(
        &mut self,
        event: Option<crossterm::event::Event>,
    ) -> super::screen::ScreenTransition {
        let original_cur = self.cur;
        if let Some(Event::Key(event)) = event {
            match event.code {
                KeyCode::Up => self.cur = if self.cur == 0 { 0 } else { self.cur - 1 },
                KeyCode::Down => self.cur = (self.cur + 1).min(self.levels.len() - 1),
                _ => {}
            }
        }
        if self.cur != original_cur {
            let _ = queue!(
                stdout(),
                MoveTo(0, original_cur as u16),
                Print(self.levels[original_cur].0.clone()),
            );
            let _ = queue!(
                stdout(),
                MoveTo(0, self.cur as u16),
                PrintStyledContent(self.levels[self.cur].0.as_str().green()),
            );
        }
        match event {
            Some(Event::Key(event)) => match event.code {
                KeyCode::Char('q') => ScreenTransition::Back,
                KeyCode::Enter => {
                    if let Ok(screen) = self.levels[self.cur].1() {
                        ScreenTransition::SwitchTo(screen)
                    } else {
                        ScreenTransition::Continue
                    }
                }
                _ => ScreenTransition::Continue,
            },
            _ => ScreenTransition::Continue,
        }
    }
}
