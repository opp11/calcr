#![feature(core, convert, str_char)]
use parser::Parser;

mod parser;
mod ast;
mod errors;
mod evaluator;

fn main() {
	println!("Hello World");
}