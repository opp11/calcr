use std::fmt;
use std::fmt::Display;
use unicode_width::UnicodeWidthChar;
use std::error::Error;

pub type CalcrResult<T> = Result<T, CalcrError>;

#[derive(Debug, PartialEq)]
pub struct CalcrError {
    pub desc: String,
    pub span: Option<(usize, usize)>,
}

impl CalcrError {
    pub fn print_location_highlight(&self, input: &String, print_input: bool) {
        let (begin, end) = self.span.unwrap_or((0, input.chars().count()));
        if print_input {
            println!("  {}", input);
            print!("  ");
        } else {
            print!("   ");
        }
        for _ in 0..begin {
            print!(" ");
        }
        print!("^");
        // Since the span is in characters, and that number does not necessarily correspond with
        // how many bytes OR display columns we need, the only way to get the number of columns
        // is by looping over the characters and summing the widths.
        for _ in 1..input.chars()
                         .skip(begin)
                         .take(end-begin)
                         .fold(0, |len, ch| len + ch.width().unwrap_or(0)) {
            print!("~");
        }
        println!("");
    }
}

impl Error for CalcrError {
    fn description(&self) -> &str {
        self.desc.as_ref()
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}

impl Display for CalcrError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.desc)
    }
}
