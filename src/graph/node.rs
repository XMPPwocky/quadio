use serde::{Deserialize, Serialize};


#[derive(Debug, Deserialize, Serialize)]
pub struct SocketDescriptor {
    pub label: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NodeDescriptor {
    pub input_sockets: Vec<SocketDescriptor>,
    pub output_sockets: Vec<SocketDescriptor>,
}

pub trait Node {
    fn get_descriptor(&self) -> NodeDescriptor;
}
