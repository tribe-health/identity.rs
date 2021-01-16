use anyhow::Result as AnyhowResult;
use core::fmt::Display;
use thiserror::Error as DeriveError;

#[derive(Debug, DeriveError)]
pub enum Error {
    #[error("Diff Error: {0}")]
    DiffError(String),
    #[error("Merge Error: {0}")]
    MergeError(String),
    #[error("Conversion Error: {0}")]
    ConversionError(String),
}

impl Error {
    pub fn diff<T>(message: T) -> Self
    where
        T: Display,
    {
        Self::DiffError(format!("{}", message))
    }

    pub fn merge<T>(message: T) -> Self
    where
        T: Display,
    {
        Self::MergeError(format!("{}", message))
    }

    pub fn convert<T>(message: T) -> Self
    where
        T: Display,
    {
        Self::ConversionError(format!("{}", message))
    }
}

pub type Result<T, E = Error> = AnyhowResult<T, E>;
