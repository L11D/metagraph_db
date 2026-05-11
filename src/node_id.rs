use bincode::{Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Encode, Decode)]
pub struct NodeId(String);

impl NodeId {
    pub fn new(id: String) -> Self {
        NodeId(id)
    }

    pub fn from_str(id: &str) -> Self {
        NodeId(id.to_string())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl<T: AsRef<str>> From<T> for NodeId {
    fn from(s: T) -> Self {
        Self::from_str(s.as_ref())
    }
}