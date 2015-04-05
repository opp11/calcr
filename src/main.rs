extern crate getopts;

use std::env;
use getopts::Options;
use errors::CalcrResult;

mod parser;
mod ast;
mod errors;
mod evaluator;

const PROG_NAME: &'static str = "calcr";
const VERSION: &'static str = "v0.0.1";

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
		println!("{} {}", PROG_NAME, VERSION);
	} else if !matches.free.is_empty() {
		for eq in matches.free {
			match process_eq(&eq) {
				Ok(num) => println!("{}", num),
				Err(e) => println!("{:?}", e),
			}
		}
	}
}

fn process_eq(eq: &String) -> CalcrResult<f64> {
	let ast = try!(parser::parse_equation(&eq));
	evaluator::eval_eq(&ast)
}

fn print_usage(opts: Options) {
	let brief = format!("usage:\n    {} [options...] [equation...]", PROG_NAME);
	print!("{}", opts.usage(&brief));
}