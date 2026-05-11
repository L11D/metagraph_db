use std::collections::{HashMap, HashSet};

use crate::{
    attribute::{Attribute, AttributeType},
    node_id::NodeId,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Vertex {
    id: NodeId,
    attributes: HashMap<String, Attribute>,
    parents: HashSet<NodeId>,
    edges: HashSet<NodeId>,
}

impl Vertex {
    pub fn new(
        id: NodeId,
        attributes: HashMap<String, Attribute>,
        parents: HashSet<NodeId>,
        edges: HashSet<NodeId>,
    ) -> Self {
        Self {
            id,
            attributes,
            parents,
            edges
        }
    }
}

impl VertexLike for Vertex {
    fn get_id(&self) -> &NodeId {
        &self.id
    }

    fn get_attributes(&self) -> &HashMap<String, Attribute> {
        &self.attributes
    }

    fn get_attributes_mut(&mut self) -> &mut HashMap<String, Attribute> {
        &mut self.attributes
    }

    fn get_parents(&self) -> &HashSet<NodeId> {
        &self.parents
    }

    fn get_parents_mut(&mut self) -> &mut HashSet<NodeId> {
        &mut self.parents
    }
    
    fn get_edges(&self) -> &HashSet<NodeId> {
        &self.edges
    }
    
    fn get_edges_mut(&mut self) -> &mut HashSet<NodeId> {
        &mut self.edges
    }
}


pub trait VertexLike {
    fn get_id(&self) -> &NodeId;

    fn get_attributes(&self) -> &HashMap<String, Attribute>;
    fn get_attributes_mut(&mut self) -> &mut HashMap<String, Attribute>;

    fn get_parents(&self) -> &HashSet<NodeId>;
    fn get_parents_mut(&mut self) -> &mut HashSet<NodeId>;

    fn get_edges(&self) -> &HashSet<NodeId>;
    fn get_edges_mut(&mut self) -> &mut HashSet<NodeId>;

    fn has_attribute(&self, name: &str, atype: AttributeType) -> bool {
        match self.get_attributes().get(name) {
            Some(attr) => attr.has_atype(atype),
            None => false,
        }
    }

    fn add_parent(&mut self, parent_id: NodeId) {
        self.get_parents_mut().insert(parent_id);
    }

    fn remove_parent(&mut self, parent_id: &NodeId) {
        self.get_parents_mut().remove(parent_id);
    }
}
