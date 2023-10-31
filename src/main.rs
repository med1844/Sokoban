use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::io::{stdout, Write};
use std::rc::Rc;

use crossterm::cursor::{MoveTo, MoveToNextLine};
use crossterm::event::{read, Event, KeyCode};
use crossterm::queue;
use crossterm::style::{Print, PrintStyledContent, Stylize};
use crossterm::terminal::{enable_raw_mode, Clear};

enum GameCommand {
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

#[derive(Clone, Copy)]
enum Grid {
    Wall,
    Ground,
    Target,
}

#[derive(Clone, Copy)]
enum Entity {
    Player,
    Box,
}

#[derive(Clone, Copy)]
struct Cell {
    grid: Grid,
    entity: Option<Entity>,
}

impl Cell {
    fn new(grid: Grid, entity: Option<Entity>) -> Self {
        Self { grid, entity }
    }
}

trait PrintableByQueue {
    fn print(&self) -> Result<(), std::io::Error>;
}

impl PrintableByQueue for Grid {
    fn print(&self) -> Result<(), std::io::Error> {
        queue!(
            stdout(),
            match *self {
                Self::Wall => PrintStyledContent("#".grey()),
                Self::Ground => PrintStyledContent(" ".reset()),
                Self::Target => PrintStyledContent(".".cyan()),
            }
        )
    }
}

impl PrintableByQueue for Entity {
    fn print(&self) -> Result<(), std::io::Error> {
        queue!(
            stdout(),
            match *self {
                Self::Box => PrintStyledContent("$".dark_yellow()),
                Self::Player => PrintStyledContent("@".blue()),
            }
        )
    }
}

impl PrintableByQueue for Cell {
    fn print(&self) -> Result<(), std::io::Error> {
        match (self.entity, self.grid) {
            (None, g) => g.print(),
            (Some(Entity::Box), Grid::Target) => queue!(stdout(), PrintStyledContent("*".yellow())),
            (Some(Entity::Player), Grid::Target) => {
                queue!(stdout(), PrintStyledContent("+".green()))
            }
            (Some(e), Grid::Ground) => e.print(),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Impossible state!",
            )),
        }
    }
}

struct Game {
    cells: Vec<Vec<Cell>>,
    n: usize,
    m: usize,
    i: usize,
    j: usize,
}

impl Game {
    fn new(cells: Vec<Vec<Cell>>) -> Self {
        let n = cells.len();
        let m = cells.first().unwrap_or(&vec![]).len();
        fn get_ij(cells: &Vec<Vec<Cell>>) -> Result<(usize, usize), &str> {
            for (i, row) in cells.iter().enumerate() {
                for (j, val) in row.iter().enumerate() {
                    if let Some(Entity::Player) = val.entity {
                        return Ok((i, j));
                    }
                }
            }
            Err("entities doesn't contain player")
        }
        match get_ij(&cells) {
            Ok((i, j)) => Self { cells, n, m, i, j },
            Err(e) => panic!("{}", e),
        }
    }

    fn push_entity(&mut self, src: (usize, usize), d: (usize, usize)) {
        let (i, j) = src;
        let (di, dj) = d;
        let ni = i.overflowing_add(di).0;
        let nj = j.overflowing_add(dj).0;
        if ni < self.n && nj < self.m {
            match self.cells[ni][nj].grid {
                Grid::Ground | Grid::Target => {
                    if let Some(Entity::Box) = self.cells[ni][nj].entity {
                        self.push_entity((ni, nj), d.clone());
                    }
                    if self.cells[ni][nj].entity.is_none() {
                        self.cells[ni][nj].entity = std::mem::take(&mut self.cells[i][j].entity);
                        self.i = ni;
                        self.j = nj;
                    }
                }
                _ => {}
            }
        }
    }

    fn execute(&mut self, command: GameCommand) -> ScreenTransition {
        match command {
            GameCommand::Null => {}
            GameCommand::Up => self.push_entity((self.i, self.j), (usize::MAX, 0)),
            GameCommand::Down => self.push_entity((self.i, self.j), (1, 0)),
            GameCommand::Left => self.push_entity((self.i, self.j), (0, usize::MAX)),
            GameCommand::Right => self.push_entity((self.i, self.j), (0, 1)),
            GameCommand::Quit => {}
        };
        match command {
            GameCommand::Quit => ScreenTransition::Break,
            _ => ScreenTransition::Continue,
        }
    }
}

impl From<&str> for Game {
    fn from(value: &str) -> Self {
        let rows = value.split('\n');
        Self::new(
            rows.into_iter()
                .map(|v| {
                    v.chars()
                        .into_iter()
                        .map(|c| match c {
                            ' ' => Cell::new(Grid::Ground, None),
                            '#' => Cell::new(Grid::Wall, None),
                            '@' => Cell::new(Grid::Ground, Some(Entity::Player)),
                            '$' => Cell::new(Grid::Ground, Some(Entity::Box)),
                            '.' => Cell::new(Grid::Target, None),
                            '+' => Cell::new(Grid::Target, Some(Entity::Player)),
                            '*' => Cell::new(Grid::Target, Some(Entity::Box)),
                            _ => panic!("Invalid char!"),
                        })
                        .collect()
                })
                .collect(),
        )
    }
}

impl PrintableByQueue for Game {
    fn print(&self) -> Result<(), std::io::Error> {
        queue!(
            stdout(),
            Clear(crossterm::terminal::ClearType::All),
            MoveTo(0, 0)
        )?;
        for row in self.cells.iter() {
            for cell in row.iter() {
                cell.print()?;
            }
            queue!(stdout(), MoveToNextLine(1))?;
        }
        Ok(())
    }
}

trait Screen {
    fn handle_input(&mut self, event: Event) -> ScreenTransition;
    fn print(&self) -> Result<(), std::io::Error>;
}

enum ScreenTransition {
    Continue,
    SwitchTo(Rc<RefCell<dyn Screen>>),
    Break,
}

struct GameScreen {
    g: Game,
}

impl GameScreen {
    fn new(g: Game) -> Self {
        Self { g }
    }
}

impl Screen for GameScreen {
    fn handle_input(&mut self, event: Event) -> ScreenTransition {
        self.g.execute(event.into())
    }

    fn print(&self) -> Result<(), std::io::Error> {
        self.g.print()?;
        stdout().flush()?;
        Ok(())
    }
}

struct MenuScreen {
    options: Vec<(String, Rc<RefCell<dyn Screen>>)>,
    choice: usize,
}

impl MenuScreen {
    fn new(options: Vec<(String, Rc<RefCell<dyn Screen>>)>) -> Self {
        Self { options, choice: 0 }
    }
}

impl Screen for MenuScreen {
    fn handle_input(&mut self, event: Event) -> ScreenTransition {
        match event {
            Event::Key(event) => match event.code {
                KeyCode::Up => self.choice = if self.choice == 0 { 0 } else { self.choice - 1 },
                KeyCode::Down => self.choice = (self.choice + 1).max(self.options.len()),
                _ => {}
            },
            _ => {}
        };
        match event {
            Event::Key(event) => match event.code {
                KeyCode::Char('q') => ScreenTransition::Break,
                KeyCode::Enter => ScreenTransition::SwitchTo(self.options[self.choice].1.clone()),
                _ => ScreenTransition::Continue,
            },
            _ => ScreenTransition::Continue,
        }
    }

    fn print(&self) -> Result<(), std::io::Error> {
        queue!(
            stdout(),
            Clear(crossterm::terminal::ClearType::All),
            MoveTo(0, 0)
        )?;
        for (desc, _) in self.options.iter() {
            queue!(stdout(), Print(desc))?;
        }
        stdout().flush()?;
        Ok(())
    }
}

struct GameApp {
    cur_screen: Rc<RefCell<dyn Screen>>,
}

impl GameApp {
    fn new(screen: Rc<RefCell<dyn Screen>>) -> Self {
        Self { cur_screen: screen }
    }
    fn run(&mut self) {
        loop {
            let transition = self
                .cur_screen
                .as_ref()
                .borrow_mut()
                .handle_input(read().unwrap());
            match transition {
                ScreenTransition::Continue => {}
                ScreenTransition::Break => break,
                ScreenTransition::SwitchTo(next_screen) => self.cur_screen = next_screen,
            }
            match self.cur_screen.as_ref().borrow().print() {
                Ok(_) => {}
                Err(msg) => println!("{:?}", msg),
            }
        }
    }
}

fn main() {
    let _ = enable_raw_mode();
    let g = Game::from(
        "     #### 
######  # 
#       # 
#      .# 
#@ #######
##       #
 # # #   #
 #     $ #
 #   #####
 #####    ",
    );
    let game_screen = Rc::new(RefCell::new(GameScreen::new(g)));
    let menu_screen = Rc::new(RefCell::new(MenuScreen::new(vec![(
        "Go Game".into(),
        game_screen.clone(),
    )])));
    let mut app = GameApp::new(menu_screen);
    app.run();
}
