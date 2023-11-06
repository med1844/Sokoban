use crate::utils::print_by_queue::PrintFullByQueue;
use crossterm::event::Event;
use std::cell::RefCell;
use std::rc::Rc;

pub trait Screen: PrintFullByQueue {
    fn handle_input(&mut self, event: Event) -> ScreenTransition;
}

pub enum ScreenTransition {
    Back,
    Continue,
    SwitchTo(Rc<RefCell<dyn Screen>>),
    Break,
}
