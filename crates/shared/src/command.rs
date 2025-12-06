#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Validate(#[from] validator::ValidationErrors),

    #[error("forbidden")]
    Forbidden,

    #[error("{0}")]
    Server(String),

    #[error("{0}")]
    Unknown(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<evento::ReadError> for Error {
    fn from(value: evento::ReadError) -> Self {
        Self::Unknown(value.into())
    }
}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        Self::Unknown(value.into())
    }
}

impl From<evento::WriteError> for Error {
    fn from(value: evento::WriteError) -> Self {
        Self::Unknown(value.into())
    }
}

impl From<bincode::error::EncodeError> for Error {
    fn from(value: bincode::error::EncodeError) -> Self {
        Self::Unknown(value.into())
    }
}

impl From<argon2::password_hash::Error> for Error {
    fn from(value: argon2::password_hash::Error) -> Self {
        Self::Unknown(value.into())
    }
}

impl From<std::time::SystemTimeError> for Error {
    fn from(value: std::time::SystemTimeError) -> Self {
        Self::Unknown(value.into())
    }
}

#[macro_export]
macro_rules! bail {
    ($msg:literal $(,)?) => {
        return Err($crate::Error::Server(format!($msg)))
    };
    ($err:expr $(,)?) => {
        return Err($crate::Error::Server(format!($err)))
    };
    ($fmt:expr, $($arg:tt)*) => {
        return Err($crate::Error::Server(format!($fmt, $($arg)*)))
        // return $crate::__private::Err($crate::__anyhow!($fmt, $($arg)*))
    };
}
