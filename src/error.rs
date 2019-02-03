use std::error::Error as StdError;
use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    pub context: String,
    pub cause: Option<Box<StdError + 'static>>,
}

impl Error {
    pub fn context<S: Into<String>>(text: S) -> Self {
        Error {
            context: text.into(),
            cause: None,
        }
    }

    pub fn iter(&self) -> ErrorIter {
        ErrorIter::new(self)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.context)
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.cause
            .as_ref()
            .map(|boxed| boxed.as_ref() as &(dyn StdError + 'static))
    }
}

#[derive(Debug)]
pub struct ErrorIter<'a>(Option<&'a dyn StdError>);

impl<'a> ErrorIter<'a> {
    pub fn new(err: &'a dyn StdError) -> Self {
        ErrorIter(Some(err))
    }
}

impl<'a> Iterator for ErrorIter<'a> {
    type Item = &'a dyn StdError;

    fn next(&mut self) -> Option<&'a dyn StdError> {
        match self.0.take() {
            Some(e) => {
                self.0 = e.cause();
                Some(e)
            }
            None => None,
        }
    }
}

pub trait ResultExt<T> {
    fn context<S: Into<String>>(self, message: S) -> Result<T>;
}

impl<T, E> ResultExt<T> for std::result::Result<T, E>
where
    E: StdError + 'static,
{
    fn context<S: Into<String>>(self, message: S) -> Result<T> {
        self.map_err(|e| Error {
            context: message.into(),
            cause: Some(e.into()),
        })
    }
}
