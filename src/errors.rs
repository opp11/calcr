#[derive(Debug)]
pub struct CalcrError {
	pub desc: String,
	pub span: Option<(usize, usize)>,
}

impl CalcrError {
    pub fn print(&self, input: Option<&String>) {
        println!("{}", self.desc);
        if let (Some((begin, end)), Some(input)) = (self.span, input) {
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
}

pub type CalcrResult<T> = Result<T, CalcrError>;