mod attribute_type;
mod value;
mod attribute;
mod error;

use parser::CompOpDef;
pub use value::*;
pub use attribute_type::*;
pub use attribute::*;
pub use error::*;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompOp {
    Equal,
    NotEqual,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
}
impl From<CompOpDef> for CompOp {
    fn from(value: CompOpDef) -> Self {
        match value {
            CompOpDef::Equal => Self::Equal,
            CompOpDef::NotEqual => Self::NotEqual,
            CompOpDef::Less => Self::Less,
            CompOpDef::LessOrEqual => Self::LessOrEqual,
            CompOpDef::Greater => Self::Greater,
            CompOpDef::GreaterOrEqual => Self::GreaterOrEqual,
        }
    }
}