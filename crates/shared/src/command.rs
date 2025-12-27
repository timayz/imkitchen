#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Validate(#[from] validator::ValidationErrors),

    #[error("forbidden")]
    Forbidden,

    #[error("not found {0}")]
    NotFound(String),

    #[error("{0}")]
    User(String),

    #[error("{0}")]
    Server(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        Self::Server(value.into())
    }
}

impl From<evento::WriteError> for Error {
    fn from(value: evento::WriteError) -> Self {
        Self::Server(value.into())
    }
}

impl From<argon2::password_hash::Error> for Error {
    fn from(value: argon2::password_hash::Error) -> Self {
        Self::Server(value.into())
    }
}

impl From<std::time::SystemTimeError> for Error {
    fn from(value: std::time::SystemTimeError) -> Self {
        Self::Server(value.into())
    }
}

impl From<std::num::TryFromIntError> for Error {
    fn from(value: std::num::TryFromIntError) -> Self {
        Self::Server(value.into())
    }
}

#[macro_export]
macro_rules! user {
    ($msg:literal $(,)?) => {
        return Err($crate::Error::User(format!($msg)))
    };
    ($err:expr $(,)?) => {
        return Err($crate::Error::User(format!($err)))
    };
    ($fmt:expr, $($arg:tt)*) => {
        return Err($crate::Error::User(format!($fmt, $($arg)*)))
    };
}

#[macro_export]
macro_rules! server {
    ($msg:literal $(,)?) => {
        return Err($crate::Error::Server(anyhow::anyhow!($msg)))
    };
    ($err:expr $(,)?) => {
        return Err($crate::Error::Server(anyhow::anyhow!($err)))
    };
    ($fmt:expr, $($arg:tt)*) => {
        return Err($crate::Error::Server(anyhow::anyhow!($fmt, $($arg)*)))
    };
}

#[macro_export]
macro_rules! not_found {
    ($msg:literal $(,)?) => {
        return Err($crate::Error::NotFound(format!($msg)))
    };
    ($err:expr $(,)?) => {
        return Err($crate::Error::NotFound(format!($err)))
    };
    ($fmt:expr, $($arg:tt)*) => {
        return Err($crate::Error::NotFound(format!($fmt, $($arg)*)))
    };
}
