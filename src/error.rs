use std::error::Error as StdError;
use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    context: String,
    cause: Option<Box<StdError + 'static>>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.context)?;
        if let Some(cause) = self.cause() {
            write!(f, "\nCaused by: {}", cause)?;
        }

        Ok(())
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.cause
            .as_ref()
            .map(|boxed| boxed.as_ref() as &(dyn StdError + 'static))
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
