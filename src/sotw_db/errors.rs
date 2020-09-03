use std;
use std::fmt::{self};

use actix_web::http::{header, StatusCode};
use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use uuid::Uuid;

#[derive(Eq, Debug, PartialEq, Serialize)]
pub enum DataError {
    NotImplementedError,
    CmdParsingError(String),
    NoActiveCompetition,
    ActiveCompetitionExists(Uuid),
    UserDoesNotOwnEntity(Uuid),
    DieselError(String),
}

#[derive(Debug, PartialEq, Serialize)]
pub struct BotError {
    pub data_error: DataError,
    pub message: String,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataError::NotImplementedError => write!(f, "This thing is not yet implemented"),
            DataError::CmdParsingError(ref msg) => {
                write!(f, "Error trying to parse cmd err={:?}", msg)
            }
            DataError::NoActiveCompetition => write!(f, "No active competition available"),
            DataError::ActiveCompetitionExists(ref id) => {
                write!(f, "An active competition already exists id={:?}", id)
            }
            DataError::UserDoesNotOwnEntity(ref id) => {
                write!(f, "Competition id={:?} is owned by another user", id)
            }
            DataError::DieselError(error) => write!(f, "{:?}", error.to_string()),
        }
    }
}

impl fmt::Display for BotError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "data_error={}, message={}",
            self.data_error, self.message
        )
    }
}

impl From<diesel::result::Error> for BotError {
    fn from(error: diesel::result::Error) -> Self {
        BotError {
            data_error: DataError::CmdParsingError(error.to_string()),
            message: error.to_string(),
        }
    }
}

impl ResponseError for BotError {
    fn status_code(&self) -> StatusCode {
        match self.data_error {
            DataError::NotImplementedError => StatusCode::NOT_IMPLEMENTED,
            DataError::NoActiveCompetition => StatusCode::NOT_FOUND,
            DataError::DieselError(_) => StatusCode::METHOD_NOT_ALLOWED,
            DataError::ActiveCompetitionExists(_) => StatusCode::CONFLICT,
            _ => StatusCode::IM_A_TEAPOT,
        }
    }
    fn error_response(&self) -> HttpResponse {
        actix_http::ResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "application/json; charset=utf-8")
            .json(self)
    }
}
