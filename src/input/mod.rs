use std::io;

pub use self::posix::PosixInputHandler;

mod posix;

const CMD_PROMPT: &'static str = ">> ";

#[derive(Debug)]
enum Key {
    Esc,
    Enter,
    Tab,

    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    Insert,
    PgUp,
    PgDown,

    Backspace,
    Delete,

    Char(char),
    F(u32),

    Unknown,
}

pub enum InputCmd {
    None,
    Quit,
    Equation(String),
}

pub trait InputHandler {
    fn start(&mut self) -> io::Result<()>;
    fn stop(&mut self) -> io::Result<()>;
    fn handle_input(&mut self) -> InputCmd;
    fn print_prompt(&self);
}