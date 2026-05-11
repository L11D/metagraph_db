use std::{fmt::Debug, str::FromStr};

use super::*;
use bincode::{Decode, Encode};
use parser::ValueDef;

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub enum Value {
    Int32(i32),
    Text(String),
    Bool(bool),
    Null,
}

impl Value {
    pub fn compare(&self, other: &Value, op: CompOp) -> Result<bool, AttributeError> {
        match (self, other) {
            (Value::Int32(a), Value::Int32(b)) => Ok(Self::compare_values(a, b, op)),
            (Value::Text(a), Value::Text(b)) => Self::simple_compare_values(a, b, op),
            (Value::Bool(a), Value::Bool(b)) => Self::simple_compare_values(a, b, op),
            (Value::Null, Value::Null) => {
                match op {
                    CompOp::Equal => Ok(true),
                    CompOp::NotEqual => Ok(false),
                    _ => Err(AttributeError::UnsupportedCompOP(op)),
                }
            },
            (Value::Null, _) => {
                match op {
                    CompOp::Equal => Ok(false),
                    CompOp::NotEqual => Ok(false),
                    _ => Err(AttributeError::UnsupportedCompOP(op)),
                }
            },
            (_, Value::Null)=> {
                match op {
                    CompOp::Equal => Ok(false),
                    CompOp::NotEqual => Ok(false),
                    _ => Err(AttributeError::UnsupportedCompOP(op)),
                }
            },
            _ => Err(AttributeError::ComparingDifferentTypes),
        }
    }

    fn compare_values<T: PartialOrd + PartialEq>(a: &T, b: &T, op: CompOp) -> bool {
        match op {
            CompOp::Equal => a == b,
            CompOp::NotEqual => a != b,
            CompOp::Less => a < b,
            CompOp::LessOrEqual => a <= b,
            CompOp::Greater => a > b,
            CompOp::GreaterOrEqual => a >= b,
        }
    }

    fn simple_compare_values<T: PartialEq>(a: &T, b: &T, op: CompOp) -> Result<bool, AttributeError> {
        match op {
            CompOp::Equal => Ok(a == b),
            CompOp::NotEqual => Ok(a != b),
            _ => Err(AttributeError::UnsupportedCompOP(op)),
        }
    }
    
    fn parse<T: FromStr>(value: &str) -> T {
        value
            .parse::<T>()
            .unwrap_or_else(|_| panic!("Failed to parse value '{}'", value))
    }

    pub fn to_string(&self) -> String {
        match self {
            Value::Int32(i) => i.to_string(),
            Value::Text(t) => t.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "None".to_string(),
        }
    }
}

// impl<T: AsRef<str>> From<T> for Value {
//     fn from(value: T) -> Self {
//         let s = value.as_ref();

//         if let Ok(int_value) = s.parse::<i32>() {
//             return Value::Int32(int_value);
//         }

//         if let Ok(bool_value) = s.parse::<bool>() {
//             return Value::Bool(bool_value);
//         }

//         Value::Text(s.to_string())
//     }
// }


impl From<ValueDef> for Value {
    fn from(value: ValueDef) -> Self {
        match value {
            ValueDef::Int32(i) => Self::Int32(i),
            ValueDef::Text(t) => Self::Text(t),
            ValueDef::Bool(b) => Self::Bool(b),
            ValueDef::Null => Self::Null,
        }
    }
}