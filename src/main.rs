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

trait PrintFullByQueue {
    fn print_full(&self) -> Result<(), std::io::Error>;
}

impl PrintFullByQueue for Grid {
    fn print_full(&self) -> Result<(), std::io::Error> {
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

impl PrintFullByQueue for Entity {
    fn print_full(&self) -> Result<(), std::io::Error> {
        queue!(
            stdout(),
            match *self {
                Self::Box => PrintStyledContent("$".dark_yellow()),
                Self::Player => PrintStyledContent("@".blue()),
            }
        )
    }
}

impl PrintFullByQueue for Cell {
    fn print_full(&self) -> Result<(), std::io::Error> {
        match (self.entity, self.grid) {
            (None, g) => g.print_full(),
            (Some(Entity::Box), Grid::Target) => queue!(stdout(), PrintStyledContent("*".yellow())),
            (Some(Entity::Player), Grid::Target) => {
                queue!(stdout(), PrintStyledContent("+".green()))
            }
            (Some(e), Grid::Ground) => e.print_full(),
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

enum GameEvent {
    Put(usize, usize, Cell),
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

    fn push_entity(&mut self, src: (usize, usize), d: (usize, usize)) -> Vec<GameEvent> {
        let (i, j) = src;
        let (di, dj) = d;
        let ni = i.overflowing_add(di).0;
        let nj = j.overflowing_add(dj).0;
        let mut res = vec![];
        if ni < self.n && nj < self.m {
            match self.cells[ni][nj].grid {
                Grid::Ground | Grid::Target => {
                    if let Some(Entity::Box) = self.cells[ni][nj].entity {
                        res.append(&mut self.push_entity((ni, nj), d.clone()));
                    }
                    if self.cells[ni][nj].entity.is_none() {
                        self.cells[ni][nj].entity = std::mem::take(&mut self.cells[i][j].entity);
                        res.push(GameEvent::Put(i, j, self.cells[i][j].clone()));
                        res.push(GameEvent::Put(ni, nj, self.cells[ni][nj].clone()));
                        self.i = ni;
                        self.j = nj;
                    }
                }
                _ => {}
            }
        }
        res
    }

    fn execute(&mut self, command: GameCommand) -> (ScreenTransition, Vec<GameEvent>) {
        let events = match command {
            GameCommand::Null => vec![],
            GameCommand::Up => self.push_entity((self.i, self.j), (usize::MAX, 0)),
            GameCommand::Down => self.push_entity((self.i, self.j), (1, 0)),
            GameCommand::Left => self.push_entity((self.i, self.j), (0, usize::MAX)),
            GameCommand::Right => self.push_entity((self.i, self.j), (0, 1)),
            GameCommand::Quit => vec![],
        };
        (
            match command {
                GameCommand::Quit => ScreenTransition::Break,
                _ => ScreenTransition::Continue,
            },
            events,
        )
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

impl PrintFullByQueue for Game {
    fn print_full(&self) -> Result<(), std::io::Error> {
        queue!(
            stdout(),
            Clear(crossterm::terminal::ClearType::All),
            MoveTo(0, 0)
        )?;
        for row in self.cells.iter() {
            for cell in row.iter() {
                cell.print_full()?;
            }
            queue!(stdout(), MoveToNextLine(1))?;
        }
        Ok(())
    }
}

trait Screen {
    fn handle_input(&mut self, event: Event) -> ScreenTransition;
}

trait PrintableScreen: Screen + PrintFullByQueue {}

enum ScreenTransition {
    Continue,
    SwitchTo(Rc<RefCell<dyn PrintableScreen>>),
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

struct MenuScreen {
    options: Vec<(String, Rc<RefCell<dyn PrintableScreen>>)>,
    choice: usize,
}

impl MenuScreen {
    fn new(options: Vec<(String, Rc<RefCell<dyn PrintableScreen>>)>) -> Self {
        Self { options, choice: 0 }
    }
}

impl PrintableScreen for GameScreen {}
impl PrintableScreen for MenuScreen {}

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
}

impl PrintFullByQueue for MenuScreen {
    fn print_full(&self) -> Result<(), std::io::Error> {
        queue!(
            stdout(),
            Clear(crossterm::terminal::ClearType::All),
            MoveTo(0, 0)
        )?;
        for (desc, _) in self.options.iter() {
            queue!(stdout(), Print(desc))?;
        }
        Ok(())
    }
}

struct GameApp {
    cur_screen: Rc<RefCell<dyn PrintableScreen>>,
}

impl GameApp {
    fn new(screen: Rc<RefCell<dyn PrintableScreen>>) -> Self {
        let _ = screen.as_ref().borrow().print_full();
        let _ = stdout().flush();
        Self { cur_screen: screen }
    }
    fn run(&mut self) {
        loop {
            let transition = self
                .cur_screen
                .as_ref()
                .borrow_mut()
                .handle_input(read().unwrap());
            let _ = stdout().flush();
            match transition {
                ScreenTransition::Continue => {}
                ScreenTransition::Break => break,
                ScreenTransition::SwitchTo(next_screen) => {
                    self.cur_screen = next_screen;
                    let _ = self.cur_screen.as_ref().borrow().print_full();
                }
            }
        }
    }
}

fn main() {
    let _ = enable_raw_mode();
    //     let g = Game::from(
    //         "     ####
    // ######  #
    // #       #
    // #      .#
    // #@ #######
    // ##       #
    //  # # #   #
    //  #     $ #
    //  #   #####
    //  #####    ",
    //     );
    let g = Game::from(
        "#####  ####  #####
#   ####  ####   #
#                #
##        ###   ##
 ## $  # .. $ @##
##  ##   ####   ##
#                #
#   ##########   #
#####        #####",
    );
    let game_screen = Rc::new(RefCell::new(GameScreen::new(g)));
    let menu_screen = Rc::new(RefCell::new(MenuScreen::new(vec![(
        "Go Game".into(),
        game_screen.clone(),
    )])));
    let mut app = GameApp::new(menu_screen);
    app.run();
}
