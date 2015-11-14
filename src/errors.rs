use std::fmt;
use std::fmt::Display;
use std::error::Error;

pub type CalcrResult<T> = Result<T, CalcrError>;

#[derive(Debug, PartialEq)]
pub struct CalcrError {
    pub desc: String,
    pub span: Option<(usize, usize)>,
}

impl CalcrError {
    pub fn print_location_highlight(&self, input: &String) {
        let (begin, end) = self.span.unwrap_or((0, input.chars().count()));
        println!("  {}", input);
        print!("  ");
        for _ in 0..begin {
            print!(" ");
        }
        print!("^");
        for _ in begin + 1..end {
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
