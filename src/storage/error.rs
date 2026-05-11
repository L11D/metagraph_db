use std::backtrace::Backtrace;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Unexpected error: {0:?}")]
    Unexpected(String),

    #[error("Nothing to evict, cash is full")]
    NothingToEvict,

    #[error("Slot {0} is out of page range")]
    SlotOutOfRange(u16, Backtrace),

    #[error("Page is full, cannot insert record")]
    PageFull,

    #[error("Record size is too large to fit in a single page")]
    OverflowRecord,
    
    #[error("Using empty id")]
    UsingEmptyId,

    #[error("Invalid id: {0}")]
    InvalidId(String)
}