use std::collections::HashMap;

use bincode::{Decode, Encode};
use thiserror::Error;
use crate::{attribute::{Attribute, AttributeType}};

#[derive(Debug, Error)]
pub enum ChildConstraintError {
    #[error("Node has not attribute {0} with type {1:?}")]
    HasAttribute(String, AttributeType)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Encode, Decode)]
pub enum ChildConstraint {
    // PlacedIn { parent_id: String },
    HasAttribute {
        name: String,
        atype: AttributeType,
    }
}

impl ChildConstraint {
    pub fn check(&self, attributes: &HashMap<String, Attribute>) -> Result<(), ChildConstraintError> {
        match self {
            ChildConstraint::HasAttribute { name, atype } => {
                match attributes.get(name) {
                    Some(attr) => {
                        if !attr.has_atype(*atype) {
                            return Err(ChildConstraintError::HasAttribute(name.into(), *atype));
                        }
                    },
                    None => return Err(ChildConstraintError::HasAttribute(name.into(), *atype)),
                }
            },
        }

        Ok(())
    }
}
