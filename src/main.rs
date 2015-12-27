extern crate getopts;
extern crate termios;
extern crate libc;
extern crate unicode_width;

use std::env;
use std::io;
use getopts::Options;
use input::{InputHandler, PosixInputHandler};
use input::InputCmd;
use interpreter::Interpreter;

mod parser;
mod ast;
mod errors;
mod interpreter;
mod lexer;
mod token;
mod input;

const PROG_NAME: &'static str = "calcr";
const VERSION: &'static str = "v0.6.0";

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    opts.optflag("v", "version", "print the program version");
    opts.optflag("h", "help", "print this and then exit");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            println!("{}", e.to_string());
            return;
        }
    };

    if matches.opt_present("h") {
        println!("calcr - a small commandline calculator");
        print_usage(opts);
    } else if matches.opt_present("v") {
        print_version();
    } else if !matches.free.is_empty() {
        let mut interp = Interpreter::new();
        for eq in matches.free {
            match interp.eval_expression(&eq) {
                Ok(Some(num)) => println!("{}", num),
                Err(e) => {
                    println!("{}", e);
                    e.print_location_highlight(&eq, true);
                },
                _ => {}, // do nothing
            }
        }
    } else {
        run_enviroment(PosixInputHandler::new()).ok().unwrap(); // TODO: Deal with the error case
    }
}

fn run_enviroment<H: InputHandler>(mut ih: H) -> io::Result<()> {
    try!(ih.start());
    print_version();
    let mut interp = Interpreter::new();
    loop {
        ih.print_prompt();
        match ih.handle_input() {
            InputCmd::Quit => break,
            InputCmd::Equation(eq) => {
                match interp.eval_expression(&eq) {
                    Ok(Some(num)) => println!("{}", num.to_string()),
                    Err(e) => {
                        e.print_location_highlight(&eq, false);
                        println!("{}", e);
                    },
                    _ => {} // do nothing
                }
            },
            InputCmd::None => {} // do nothing
        }
    }
    println!(""); // an extra newline to make sure the terminal looks tidy
    Ok(())
}

fn print_usage(opts: Options) {
    let brief = format!("Usage:\n    {} [options...] [equation...]", PROG_NAME);
    println!("{}", opts.usage(&brief));
}

fn print_version() {
    println!("{} {}", PROG_NAME, VERSION);
}