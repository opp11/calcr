#[derive(Debug)]
pub struct CError {
	pub desc: String,
	pub span: (usize, usize)
}

pub type CResult<T> = Result<T, CError>;