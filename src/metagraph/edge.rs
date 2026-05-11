use std::collections::HashMap;

use crate::{attribute::Attribute, node_id::NodeId};

#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    id: NodeId,
    attributes: HashMap<String, Attribute>,
    from: NodeId,
    to: NodeId,
}
impl Edge {
    pub fn new(
        id: NodeId,
        attributes: HashMap<String, Attribute>,
        from: NodeId,
        to: NodeId,
    ) -> Self {
        Self { id, attributes, from, to }
    }

    pub fn get_id(&self) -> &NodeId {
        &self.id
    }

    pub fn get_attributes(&self) -> &HashMap<String, Attribute> {
        &self.attributes
    }

    pub fn get_from(&self) -> &NodeId {
        &self.from
    }

    pub fn get_to(&self) -> &NodeId {
        &self.to
    }
}
