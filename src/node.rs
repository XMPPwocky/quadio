use std::collections::HashMap;

pub struct SocketDescriptor {
    pub label: String,
}

pub struct NodeDescriptor {
    pub input_sockets: HashMap<u64, SocketDescriptor>,
    pub output_sockets: HashMap<u64, SocketDescriptor>
}

pub trait Node {
    fn show_ui(&mut self, ui: &mut egui::Ui);

    fn get_descriptor(&self) -> NodeDescriptor;
}