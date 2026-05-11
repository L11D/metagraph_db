use thiserror::Error;
use super::*;

#[derive(Debug, Error)]
pub enum AttributeError {
    #[error("Required value of type {0:?} cannot be None")]
    MissingRequired(AttributeType),

    #[error("Invalid value type")]
    InvalidType,

    #[error("Unsupported comparison operation: {0:?}")]
    UnsupportedCompOP(CompOp),

    #[error("Cannot compare attributes of different types")]
    ComparingDifferentTypes
}