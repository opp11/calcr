use std::io;
use std::io::Write;
use super::CMD_PROMPT;
use super::{InputHandler, InputCmd};
use super::Key;

pub struct DefaultInputHandler;

impl DefaultInputHandler {
    pub fn new() -> DefaultInputHandler {
        DefaultInputHandler
    }
}

impl InputHandler for DefaultInputHandler {
    fn start(&mut self) -> io::Result<()> {
        // Do nothing
        Ok(())
    }

    fn stop(&mut self) -> io::Result<()> {
        // Do nothing
        Ok(())
    }

    fn handle_input(&mut self) -> InputCmd {
        let mut cmd = String::new();
        if let Ok(_) = io::stdin().read_line(&mut cmd) {
            if cmd.trim() == "quit" || cmd.trim() == "exit" {
                InputCmd::Quit
            } else {
                println!(""); // go to new line to prepare for output
                InputCmd::Equation(cmd)
            }
        } else {
            // TODO: Actually handle errors
            InputCmd::None
        }
    }

    fn print_prompt(&self) {
        print!("{}", CMD_PROMPT);
        io::stdout().flush().ok().expect("Could not write prompt to terminal");
    }
}