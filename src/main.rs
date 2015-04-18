extern crate getopts;

use std::env;
use std::io::Write;
use getopts::Options;
use errors::CalcrResult;

mod parser;
mod ast;
mod errors;
mod evaluator;

const PROG_NAME: &'static str = "calcr";
const VERSION: &'static str = "v0.2.2";

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
        run_enviroment();
    }
}

fn process_eq(eq: &String) -> CalcrResult<f64> {
	let ast = try!(parser::parse_equation(&eq));
	evaluator::eval_eq(&ast)
}

fn run_enviroment() {
    print_version();
    let mut buf = String::new();
    let mut in_stream = std::io::stdin();
    let mut out_stream = std::io::stdout();
    loop {
        print!(">> ");
        // we explicitly call flush on stdout, or else the '>>' prompt won't be printed untill
        // after we have read the user's input
        out_stream.flush().ok().expect("Fatal error while writing prompt");
        in_stream.read_line(&mut buf).ok().expect("Fatal error while reading input");
        let eq = buf.trim().to_string();
        if eq == "quit" {
            break;
        }
        match process_eq(&eq) {
            Ok(num) => println!("{}", num),
            Err(e) => e.print(Some(&eq)),
        }
        buf.clear();
    }
}

fn print_usage(opts: Options) {
	let brief = format!("Usage:\n    {} [options...] [equation...]", PROG_NAME);
	print!("{}", opts.usage(&brief));
}

fn print_version() {
    println!("{} {}", PROG_NAME, VERSION);
}