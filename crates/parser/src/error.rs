use thiserror::Error;

use crate::{CompOpDef, lexer::LexerError};

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Expected: {0:?}, actual: {1:?}")]
    UnexpectedNode(String, String),

    #[error("Unsupported comparison operation: {0:?} with: {1:?}")]
    UnsupportedCompOP(CompOpDef, String),

    #[error("Can not find edge")]
    CantFindEdge,

    #[error("Invalid create")]
    InvalidCreate,

    #[error("Unknown type: {0:?}")]
    UnknownType(String),

    #[error("Miss find or create statement")]
    MissFindOrCreate,
    
    #[error("Unexpected EOF")]
    UnexpectedEOF,

    #[error("LexerError: {0:?}")]
    LexerError(#[from] LexerError),

    #[error("Statement: {0} already defined")]
    AlreadyDefined(String),

    #[error("Unknown error")]
    Unknown,
}