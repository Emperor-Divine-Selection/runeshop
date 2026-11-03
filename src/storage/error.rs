use object_store::Error;
use std::fmt::Display;

#[derive(Debug)]
pub enum StorageError {
    Technical(Error),
}

impl From<Error> for StorageError {
    fn from(value: Error) -> Self {
        Self::Technical(value)
    }
}

impl Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Technical(e) => write!(f, "[Technical] {e}"),
        }
    }
}
