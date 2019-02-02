use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            _ => write!(f, "test"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            _ => None,
        }
    }
}
