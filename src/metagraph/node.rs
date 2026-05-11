use std::collections::{HashMap, HashSet};

use crate::{
    attribute::Attribute,
    metagraph::{Metavertex, Vertex, VertexLike},
    node_id::NodeId,
};

#[derive(Debug, Clone)]
pub enum Node {
    Metavertex(Metavertex),
    Vertex(Vertex),
}

impl VertexLike for Node {
    fn get_id(&self) -> &NodeId {
        match self {
            Node::Metavertex(mv) => mv.get_id(),
            Node::Vertex(v) => v.get_id(),
        }
    }

    fn get_attributes(&self) -> &HashMap<String, Attribute> {
        match self {
            Node::Metavertex(mv) => mv.get_attributes(),
            Node::Vertex(v) => v.get_attributes(),
        }
    }

    fn get_attributes_mut(&mut self) -> &mut HashMap<String, Attribute> {
        match self {
            Node::Metavertex(mv) => mv.get_attributes_mut(),
            Node::Vertex(v) => v.get_attributes_mut(),
        }
    }

    fn get_parents(&self) -> &HashSet<NodeId> {
        match self {
            Node::Metavertex(mv) => mv.get_parents(),
            Node::Vertex(v) => v.get_parents(),
        }
    }

    fn get_parents_mut(&mut self) -> &mut HashSet<NodeId> {
        match self {
            Node::Metavertex(mv) => mv.get_parents_mut(),
            Node::Vertex(v) => v.get_parents_mut(),
        }
    }
    
    fn get_edges(&self) -> &HashSet<NodeId> {
        match self {
            Node::Metavertex(mv) => mv.get_edges(),
            Node::Vertex(v) => v.get_edges(),
        }
    }
    
    fn get_edges_mut(&mut self) -> &mut HashSet<NodeId> {
        match self {
            Node::Metavertex(mv) => mv.get_edges_mut(),
            Node::Vertex(v) => v.get_edges_mut(),
        }
    }
}
