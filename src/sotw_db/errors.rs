use std::fmt;
use uuid::Uuid;

#[derive(Debug, PartialEq)]
pub enum DataError {
    ActiveCompetitionExists(Uuid),
    UserDoesNotOwnEntity(Uuid),
    DieselError(diesel::result::Error),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
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

impl From<diesel::result::Error> for DataError {
    fn from(error: diesel::result::Error) -> Self {
        DataError::DieselError(error)
    }
}
