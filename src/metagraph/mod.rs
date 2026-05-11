mod node;
mod child_constraint;
mod metavertex;
mod vertex;
mod edge;

pub use child_constraint::*;
pub use node::*;
pub use metavertex::*;
pub use vertex::*;
pub use edge::*;

#[derive(Debug, Clone, PartialEq)]
pub enum EntityType {
    Vertex,
    Metavertex,
    Edge
}