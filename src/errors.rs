#[derive(Debug)]
pub struct CalcrError {
	pub desc: String,
	pub span: (usize, usize)
}

pub type CalcrResult<T> = Result<T, CalcrError>;