pub trait PrintFullByQueue {
    fn print_full(&self) -> Result<(), std::io::Error>;
}
