use std::collections::{HashMap, HashSet};

use crate::{
    attribute::Attribute,
    metagraph::{ChildConstraint, ChildConstraintError, Node, Vertex, VertexLike},
    node_id::NodeId,
};

#[derive(Debug, PartialEq, Clone)]
pub struct Metavertex {
    vertex: Vertex,
    v_children: HashSet<NodeId>,
    mv_children: HashSet<NodeId>,
    constrains: HashSet<ChildConstraint>,
}

impl Metavertex {
    pub fn new(
        id: NodeId,
        attributes: HashMap<String, Attribute>,
        parents: HashSet<NodeId>,
        edges: HashSet<NodeId>,
        v_children: HashSet<NodeId>,
        mv_children: HashSet<NodeId>,
        constrains: HashSet<ChildConstraint>,
    ) -> Self {
        Self {
            vertex: Vertex::new(id, attributes, parents, edges),
            v_children,
            mv_children,
            constrains,
        }
    }

    pub fn get_vertex(&self) -> &Vertex {
        &self.vertex
    }

    pub fn get_child_vertices(&self) -> &HashSet<NodeId> {
        &self.v_children
    }

    pub fn get_child_metavertexes(&self) -> &HashSet<NodeId> {
        &self.mv_children
    }

    pub fn get_constrains(&self) -> &HashSet<ChildConstraint> {
        &self.constrains
    }

    pub fn get_child_nodes(&self) -> HashSet<NodeId> {
        &self.v_children | &self.mv_children
    }

    // pub fn check_child_constrains(&self, child: &Node) -> Result<(), ChildConstraintError> {
    //     for constrain in &self.constrains {
    //         constrain.check(child)?;
    //     }
    //     Ok(())
    // }

    pub fn add_child(&mut self, child: &Node) {
        match child {
            Node::Metavertex(_) => self.mv_children.insert(child.get_id().clone()),
            Node::Vertex(_) => self.v_children.insert(child.get_id().clone()),
        };
    }

    pub fn chick_child_exists(&self, child_id: &NodeId) -> bool {
        self.v_children.contains(child_id) || self.mv_children.contains(child_id)
    }
}

impl VertexLike for Metavertex {
    fn get_id(&self) -> &NodeId {
        self.vertex.get_id()
    }

    fn get_attributes(&self) -> &HashMap<String, Attribute> {
        self.vertex.get_attributes()
    }

    fn get_attributes_mut(&mut self) -> &mut HashMap<String, Attribute> {
        self.vertex.get_attributes_mut()
    }

    fn get_parents(&self) -> &HashSet<NodeId> {
        self.vertex.get_parents()
    }

    fn get_parents_mut(&mut self) -> &mut HashSet<NodeId> {
        self.vertex.get_parents_mut()
    }
    
    fn get_edges(&self) -> &HashSet<NodeId> {
        self.vertex.get_edges()
    }
    
    fn get_edges_mut(&mut self) -> &mut HashSet<NodeId> {
        self.vertex.get_edges_mut()
    }
}


