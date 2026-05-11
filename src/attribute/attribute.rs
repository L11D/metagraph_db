use super::*;
use bincode::{Decode, Encode};
use parser::AttributeDef;

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct Attribute {
    atype: AttributeType,
    value: Value,
}

impl Attribute {
    pub fn new(atype: AttributeType, value: Value) -> Self {
        Self { atype, value }
    }
    
    // pub fn from_string<T: AsRef<str>>(str_value: T, nullable: bool) -> Self {
    //     let value: Value = str_value.into();
    //     let prim_type = value.get_prim_type();
    //     let dtype = AttributeType::new(prim_type, nullable);
    //     Self { atype: dtype, value }
    // }
    
    pub fn has_atype(&self, atype: AttributeType) -> bool {
        self.atype == atype
    }

    pub fn compare_value(&self, other: &Value, op: CompOp) -> Result<bool, AttributeError> {
        self.value.compare(other, op)
    }

    pub fn atype(&self) -> &AttributeType {
        &self.atype
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn set_value(&mut self, value: Value) -> Result<(), AttributeError> {
        if !self.atype.support_value(&value) {
            return Err(AttributeError::InvalidType);
        }
        self.value = value;
        Ok(())
    }

}

// impl<T: AsRef<str>> From<T> for Attribute {
//     fn from(s: T) -> Self {
//         Self::from_string(s.as_ref(), false)
//     }
// }


// impl From<&str> for Attribute {
//     fn from(s: &str) -> Self {
//         Self::from_string(s, false)
//     }
// }

// impl From<String> for Attribute {
//     fn from(s: String) -> Self {
//         Self::from_string(&s, false)
//     }
// }

impl From<i32> for Attribute {
    fn from(value: i32) -> Self {
        let value = Value::Int32(value);
        let atype = AttributeType::new(PrimitiveType::Int32, false);
        Self { atype, value }
    }
}

impl From<&AttributeDef> for Attribute {
    fn from(value: &AttributeDef) -> Self {
        let atype = AttributeType {
            prim_type: value.primitive_type.into(),
            nullable: value.nullable
        };
        let v = value.value.clone().into();

        Self { atype: atype, value: v }
    }
}