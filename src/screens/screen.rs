use crossterm::event::Event;
use sokoban::utils::print_by_queue::PrintFullByQueue;
use std::cell::RefCell;
use std::rc::Rc;

pub trait Screen: PrintFullByQueue {
    fn update(&mut self, event: Option<Event>) -> ScreenTransition;
}

pub enum ScreenTransition {
    Back,
    Continue,
    SwitchTo(Rc<RefCell<dyn Screen>>),
    Break,
}
