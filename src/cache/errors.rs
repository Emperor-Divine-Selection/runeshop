use fred::error::Error;
use std::fmt::{self};

use crate::cache::errors::CacheError::Technical;

#[derive(Debug)]
pub enum CacheError {
    Technical(Error),
}

impl From<Error> for CacheError {
    fn from(value: Error) -> Self {
        Technical(value)
    }
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Technical(msg) => write!(f, "[Technical] {msg}"),
        }
    }
}
