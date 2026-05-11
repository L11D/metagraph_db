use bincode::{Decode, Encode};
use parser::PrimitiveTypeDef;

use crate::attribute::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
pub enum PrimitiveType {
    Int32,
    Text,
    Bool,
}
impl PrimitiveType {
    pub fn to_string(&self) -> String {
        match self {
            PrimitiveType::Int32 => "int32".into(),
            PrimitiveType::Text => "text".into(),
            PrimitiveType::Bool => "bool".into(),
        }
    }
}
impl From<PrimitiveTypeDef> for PrimitiveType {
    fn from(value: PrimitiveTypeDef) -> Self {
        match value {
            PrimitiveTypeDef::Int32 => PrimitiveType::Int32,
            PrimitiveTypeDef::Text => PrimitiveType::Text,
            PrimitiveTypeDef::Bool => PrimitiveType::Bool,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
pub struct AttributeType {
    pub prim_type: PrimitiveType,
    pub nullable: bool,
}

impl AttributeType {
    pub fn new(prim_type: PrimitiveType, nullable: bool) -> Self {
        Self { prim_type, nullable }
    }

    pub fn support_value(&self, value: &Value) -> bool {
        if self.nullable && matches!(value, Value::Null) {
            return true;
        }
        if !self.nullable && matches!(value, Value::Null) {
            return false;
        }
        match (self.prim_type, value) {
            (PrimitiveType::Int32, Value::Int32(_)) => true,
            (PrimitiveType::Text, Value::Text(_)) => true,
            (PrimitiveType::Bool, Value::Bool(_)) => true,
            _ => false
        }
    }

    pub fn to_string(&self) -> String {
        format!("{}{}", self.prim_type.to_string(), if self.nullable { "?" } else { "" })
    }
}