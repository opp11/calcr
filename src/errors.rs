#[derive(Debug)]
pub struct CalcrError {
	pub desc: String,
	pub span: Option<(usize, usize)>,
}

pub type CalcrResult<T> = Result<T, CalcrError>;