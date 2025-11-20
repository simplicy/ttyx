//! This is the main (and only for now) application Error type.
//! It's using 'thiserror' as it reduces boilerplate error code while providing rich error typing.
//!
//! Notes:
//!     - The strategy is to start with one Error type for the whole application and then seggregate as needed.
//!     - Since everything is typed from the start, renaming and refactoring become relatively trivial.
//!     - By best practices, `anyhow` is not used in application code, but can be used in unit or integration test (will be in dev_dependencies when used)
//!

//Once ready to add Actix Error types that can be returned from the API, add the following to the top of the file
// use actix_web::HttpResponse;

use std::io::ErrorKind;

use color_eyre::eyre::Report;
use serde::Serialize;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    AppFail,

    Configuration(String),

    XValueNotOfType(&'static str),

    XPropertyNotFound(String),

    FailedToGetContext,

    FailedToGetToken,

    FailedtoGetCalendarList,

    FailedToAddCalendar,

    FailedToCheckToken,

    FailedToGetKey,

    StoreFailToCreate(String),

    StoreFailToRead(String),

    InvalidType,

    StoreFailedToDelete(String),

    StoreFailedToInit(String),

    StoreFailedToLogin(String),

    FailedToSetArgsentNS(String),

    JwtNotAuthorized,

    MissingConfig,

    UserNotFound,

    InvalidPassword,

    InvalidToken,

    ExpiredToken,

    UnknownDatabaseType,

    EmptyHeader,

    InvalidEmail,

    CreatingConfig,

    NotAuthorized,

    Unauthorized,

    TokenCouldNotBeRead,

    WrongUsernameOrPassword,

    JsonSerde(serde_json::Error),

    ModqlOperatorNotSupported(String),

    IO(std::io::Error),

    FailedToGetCalendar,
    FailedToCreateToken(String),
    FailedToFindToken(String),
    BadRequest(String),
    NotFound,
    Conflict,
    Exists,
    InvalidConfigFile,
    MissingValue,
    InvalidLogLevel,
    DeserializingConfig,
    Unknown(String),
    DatabaseConfig(String),
    SurrealDB(String),
    Cursor,
    InvalidKeyEvent(String),
    LoadingConfigFile,
    InvalidAppDataPath,
    FailedRequest,
    ActionSender(String),
}

#[derive(Serialize)]
pub struct ErrorMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_description: Option<String>,
    pub message: String,
}

impl From<Error> for std::io::Error {
    fn from(value: Error) -> Self {
        std::io::Error::new(ErrorKind::Other, value)
    }
}

impl From<std::io::Error> for ErrorMessage {
    fn from(value: std::io::Error) -> Self {
        ErrorMessage {
            error: Some(value.kind().to_string()),
            error_description: Some(value.to_string()),
            message: value.to_string(),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(val: serde_json::Error) -> Self {
        Error::JsonSerde(val)
    }
}
impl From<std::io::Error> for Error {
    fn from(val: std::io::Error) -> Self {
        Error::IO(val)
    }
}

impl From<Report> for Error {
    fn from(val: Report) -> Self {
        Error::Unknown(val.to_string())
    }
}
impl From<ratzilla::error::Error> for Error {
    fn from(val: ratzilla::error::Error) -> Self {
        Error::Unknown(val.to_string())
    }
}
// endregion: --- Froms
//
// region:    --- Error Boiler
impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> core::result::Result<(), std::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
// endregion: --- Error Boiler
