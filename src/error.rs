use std::borrow::Cow;
use std::error::Error as StdError;
use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    pub context: Cow<'static, str>,
    pub cause: Option<Box<StdError + 'static>>,
}

impl Error {
    pub fn context<S>(text: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
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
    fn context<F, S>(self, to_message: F) -> Result<T>
    where
        F: FnOnce() -> S,
        S: Into<Cow<'static, str>>;
}

impl<T, E> ResultExt<T> for std::result::Result<T, E>
where
    E: StdError + 'static,
{
    fn context<F, S>(self, to_message: F) -> Result<T>
    where
        F: FnOnce() -> S,
        S: Into<Cow<'static, str>>,
    {
        self.map_err(|e| Error {
            context: to_message().into(),
            cause: Some(e.into()),
        })
    }
}
