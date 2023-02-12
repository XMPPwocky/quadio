use std::collections::HashMap;

use slotmap::new_key_type;

use crate::node::{Node, NodeDescriptor};

new_key_type! {
    pub struct NodeKey;
}

#[derive(Default)]
pub struct NodeGraph<N: Node> {
    nodes: slotmap::SlotMap<NodeKey, N>,
    descriptors: slotmap::secondary::SecondaryMap<NodeKey, NodeDescriptor>,


    // each input socket has only one thing connected
    // so we can use that :)
    wires_by_destination: HashMap<(NodeKey, u64), (NodeKey, u64)>
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

            assert!(src_descriptor.output_sockets.contains_key(&src.1));
            assert!(dst_descriptor.input_sockets.contains_key(&dst.1));
        }
    }

    pub fn add_node(&mut self, node: N) -> NodeKey {
        // more convenient to do this now before we stick `node` into self.nodes
        let descriptor = node.get_descriptor();

        let key = self.nodes.insert(node);
        self.descriptors.insert(key, descriptor);
        
        key
    }

    pub fn remove_node(&mut self, node_key: NodeKey) -> Option<N> {
        // remove any wires to or from this node's sockets
        self.wires_by_destination.retain(|dst, src| {
            // i.e. keep only those which are unrelated to the removed node
            dst.0 != node_key && src.0 != node_key
        });

        self.nodes.remove(node_key)
    }
}