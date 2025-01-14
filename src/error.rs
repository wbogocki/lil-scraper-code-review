use std::fmt::{Display, Formatter, Result as FmtResult};

// NOTE(Wojciech): You probably know this but this is not useful for any kind of debugging. If an error happens you
// won't know why and probably won't be able to fix it. There's two options here: include the underlying error on the
// error structure itself or use a library e.g. anyhow and/or thiserror to get nice errors.
#[derive(Debug, PartialEq)]
pub enum ScrapeError {
    InvalidResponse,
    InvalidURI,
    NoMatch,
    RequestFailed,
    RequestTimeout,
    SendError,
}

impl ScrapeError {
    pub fn message(&self) -> &str {
        match self {
            Self::InvalidResponse => "INVALID RESPONSE",
            Self::InvalidURI => "INVALID URI",
            Self::NoMatch => "NO MATCH",
            Self::RequestFailed => "REQUEST FAILED",
            Self::RequestTimeout => "REQUEST TIMEOUT",
            Self::SendError => "SEND ERROR",
        }
    }
}

impl Display for ScrapeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message())
    }
}
