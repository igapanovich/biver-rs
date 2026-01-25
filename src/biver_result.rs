use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

pub type BiverResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct BiverError {
    error_message: String,
}

impl BiverError {
    pub fn new(error_message: impl Into<String>) -> BiverError {
        BiverError {
            error_message: error_message.into(),
        }
        .into()
    }
}

impl Display for BiverError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error_message)
    }
}

impl Error for BiverError {}

pub fn error<T>(message: impl Into<String>) -> BiverResult<T> {
    Err(BiverError::new(message).into())
}
