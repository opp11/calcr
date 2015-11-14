extern crate getopts;
extern crate termios;
extern crate libc;

use std::env;
use std::io;
use getopts::Options;
use errors::CalcrResult;
use input::{InputHandler, PosixInputHandler};
use input::InputCmd;

mod parser;
mod ast;
mod errors;
mod evaluator;
mod lexer;
mod token;
mod input;

const PROG_NAME: &'static str = "calcr";
const VERSION: &'static str = "v0.4.1";

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
        for eq in matches.free {
            match process_eq(&eq) {
                Ok(num) => println!("{}", num),
                Err(e) => e.print(Some(&eq)),
            }
        }
    } else {
        run_enviroment(PosixInputHandler::new()).ok().unwrap(); // TODO: Deal with the error case
    }
}

fn process_eq(eq: &String) -> CalcrResult<f64> {
    let tokens = try!(lexer::lex_equation(&eq));
    let ast = try!(parser::parse_equation(tokens));
    evaluator::eval_eq(&ast)
}

fn run_enviroment<H: InputHandler>(mut ih: H) -> io::Result<()> {
    try!(ih.start());
    print_version();
    loop {
        ih.print_prompt();
        match ih.handle_input() {
            InputCmd::Quit => break,
            InputCmd::Equation(eq) => {
                match process_eq(&eq) {
                    Ok(num) => println!("{}", num.to_string()),
                    Err(e) => e.print(Some(&eq)),
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