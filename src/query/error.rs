use std::{backtrace::Backtrace};

use axum::{Json, http::StatusCode, response::{IntoResponse, Response}};
use parser::ParseError;
// use thiserror::Error;

use crate::{attribute::AttributeError, metagraph::ChildConstraintError, storage::StorageError};


#[derive(thiserror::Error, Debug)]
pub enum EngineError {
    #[error("Entity not found")]
    EntityNotFound,

    #[error("Unknown parent: {0}")]
    UnknownParent(String),

    #[error("Storage error: {source}")]
    StorageError {
        #[from] 
        source: StorageError,
        bt: Backtrace
    },

    #[error("{0}")]
    AttributeError(#[from] AttributeError),

    #[error("{0}")]
    ParseError(#[from] ParseError),

    #[error("Unexpected error: {0}")]
    Unexpected(String),

    #[error("Metavertex with name [{0}] already exists")]
    MetaAlreadyExists(String),

    #[error("Constraint error: {0}")]
    ConstraintError(#[from] ChildConstraintError),

    #[error("Undefined alias: [{0}]")]
    UndefinedAlias(String),

    #[error("Attribute: [{0}] not exist")]
    AttributeNotExist(String),

    #[error("Parent to not metavertex")]
    ParentToNotMeta,
}

impl From<std::io::Error> for EngineError {
    fn from(value: std::io::Error) -> Self {
        EngineError::Unexpected(value.to_string())
    }
}
// impl From<StorageError> for EngineError {
//     fn from(err: StorageError) -> Self {
//         Self::StorageError {
//             source: err,
//             bt: std::backtrace::Backtrace::capture(), // Захватываем стек здесь
//         }
//     }
// }

impl IntoResponse for EngineError {
    fn into_response(self) -> Response {
        log::error!("Root error: {}", self); 
                
        let (status, message) = match &self {
            EngineError::EntityNotFound => (StatusCode::BAD_REQUEST, Some(self.to_string())),
            EngineError::Unexpected(err) => {
                // log::error!("{:?}\n", err);
                println!("{}", Backtrace::capture());
                (StatusCode::INTERNAL_SERVER_ERROR, None)
            }
            EngineError::UnknownParent(_) => (StatusCode::BAD_REQUEST, Some(self.to_string())),
            EngineError::ParseError(_) => (StatusCode::BAD_REQUEST, Some(self.to_string())),
            EngineError::StorageError{source, bt} => {
                log::error!("bt: {}", bt);
                (StatusCode::BAD_REQUEST, Some(self.to_string()))
            }
            EngineError::AttributeError(_) => (StatusCode::BAD_REQUEST, Some(self.to_string())),
            EngineError::MetaAlreadyExists(_) => (StatusCode::BAD_REQUEST, Some(self.to_string())),
            EngineError::ConstraintError(_) => (StatusCode::BAD_REQUEST, Some(self.to_string())),
            EngineError::UndefinedAlias(_) => (StatusCode::BAD_REQUEST, Some(self.to_string())),
            EngineError::AttributeNotExist(_) => (StatusCode::BAD_REQUEST, Some(self.to_string())),
            EngineError::ParentToNotMeta => (StatusCode::BAD_REQUEST, Some(self.to_string())),
        };

        match message {
            None => status.into_response(),
            Some(m) => (status, Json(m)).into_response()
        }
    }
}