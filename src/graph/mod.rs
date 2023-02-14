use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use slotmap::new_key_type;

mod node;
pub use node::{Node, NodeDescriptor, SocketDescriptor};

new_key_type! {
    pub struct NodeKey;
}

#[derive(Serialize, Deserialize)]
pub struct NodeGraph<N: Node> {
    nodes: slotmap::SlotMap<NodeKey, N>,
    descriptors: slotmap::secondary::SecondaryMap<NodeKey, NodeDescriptor>,

    // each input socket has only one thing connected
    // so we can use that :)
    wires_by_destination: HashMap<(NodeKey, usize), (NodeKey, usize)>,
}
impl<N: Node> Default for NodeGraph<N> {
    fn default() -> Self {
        NodeGraph {
            nodes: Default::default(),
            descriptors: Default::default(),
            wires_by_destination: Default::default(),
        }
    }
}
impl<N: Node> NodeGraph<N> {
    #[doc(hidden)]
    pub fn validate_wires(&self) {
        for (dst, src) in &self.wires_by_destination {
            // FIXME: we must also check the input/output name
            assert!(self.nodes.contains_key(src.0));
            assert!(self.nodes.contains_key(dst.0));

            let src_descriptor = &self.descriptors[src.0];
            let dst_descriptor = &self.descriptors[dst.0];

            assert!(src_descriptor.output_sockets.get(src.1).is_some());
            assert!(dst_descriptor.input_sockets.get(dst.1).is_some());
        }
    }

    pub fn add_node(&mut self, node: N) -> NodeKey {
        // more convenient to do this now before we stick `node` into self.nodes
        let descriptor = node.get_descriptor();

        let key = self.nodes.insert(node);
        self.descriptors.insert(key, descriptor);

        self.validate_wires();

        key
    }

    pub fn remove_node(&mut self, node_key: NodeKey) -> Option<N> {
        // remove any wires to or from this node's sockets
        self.wires_by_destination.retain(|dst, src| {
            // i.e. keep only those which are unrelated to the removed node
            dst.0 != node_key && src.0 != node_key
        });

        let rv = self.nodes.remove(node_key);

        self.validate_wires();

        rv
    }

    pub fn connect(
        &mut self,
        src: (NodeKey, usize),
        dst: (NodeKey, usize),
    ) -> Option<(NodeKey, usize)> {
        // FIXME: validate args here to avoid confusing blowups later!
        let rv = self.wires_by_destination.insert(dst, src);

        self.validate_wires();

        rv
    }
    pub fn disconnect(&mut self, node: NodeKey, dir: SocketDirection, idx: usize) {
        match dir {
            SocketDirection::Input => {
                // easy
                self.wires_by_destination.remove(&(node, idx));
            }
            SocketDirection::Output => {
                // ugh... have to do a linear scan... this sucks
                self.wires_by_destination.retain(|_, v| *v != (node, idx));
            }
        }
    }
    pub fn node_descriptor(&self, node: NodeKey) -> &NodeDescriptor {
        &self.descriptors[node]
    }
    pub fn nodes(&self) -> impl Iterator<Item = (NodeKey, &N)> {
        self.nodes.iter()
    }
    pub fn nodes_mut(&mut self) -> impl Iterator<Item = (NodeKey, &mut N, &NodeDescriptor)> {
        self.nodes
            .iter_mut()
            .map(|(k, v)| (k, v, &self.descriptors[k]))
    }

    /// returns iterator over (src, dest). dest is globally unique
    pub fn wires(&self) -> impl Iterator<Item = ((NodeKey, usize), (NodeKey, usize))> + '_ {
        self.wires_by_destination.iter().map(|(&k, &v)| (k, v))
    }

    pub fn src_for_dest(&self, node: NodeKey, idx: usize) -> Option<(NodeKey, usize)> {
        self.wires_by_destination.get(&(node, idx)).copied()
    }

    pub fn get_node_mut(&mut self, node: NodeKey) -> &mut N {
        &mut self.nodes[node]
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum SocketDirection {
    Input,
    Output,
}
