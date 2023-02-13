use serde::{Deserialize, Serialize};

use crate::graph::{self, NodeDescriptor, SocketDescriptor};

use crate::sample::QuadioSample;

pub trait QuadioNode: graph::Node {
    fn show_ui(&mut self, ui: &mut egui::Ui);

    fn process(&mut self, inputs: &[&[QuadioSample]], outputs: &mut [&mut [QuadioSample]]);
}
impl graph::Node for Box<dyn QuadioNode> {
    fn get_descriptor(&self) -> graph::NodeDescriptor {
        (**self).get_descriptor()
    }
}

#[derive(Default)]
pub struct PassthruNode;
impl graph::Node for PassthruNode {
    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            input_sockets: vec![
                SocketDescriptor {
                    label: "In".to_owned()
                }
            ], output_sockets: vec![
                SocketDescriptor {
                    label: "Out".to_owned()
                }
            ]
        }
    }
}
impl QuadioNode for PassthruNode {
    fn show_ui(&mut self, ui: &mut egui::Ui) {
        ui.label("PASSTHRU");
    }

    fn process(&mut self, inputs: &[&[QuadioSample]], outputs: &mut [&mut [QuadioSample]]) {
        for (inp, out) in inputs[0].iter().zip(outputs[0].iter_mut()) {
            *out = *inp;
        }
    }
}

impl<'de> Deserialize<'de> for Box<dyn QuadioNode> {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        unimplemented!()
    }
}
impl Serialize for Box<dyn QuadioNode> {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        unimplemented!()
    }
}


#[derive(Default)]
pub struct SumNode;
impl graph::Node for SumNode {
    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            input_sockets: vec![
                SocketDescriptor {
                    label: "A".to_owned()
                },
                SocketDescriptor {
                    label: "B".to_owned()
                }
            ], output_sockets: vec![
                SocketDescriptor {
                    label: "Out".to_owned()
                }
            ]
        }
    }
}
impl QuadioNode for SumNode {
    fn show_ui(&mut self, ui: &mut egui::Ui) {
        ui.label("SUM");
    }

    fn process(&mut self, inputs: &[&[QuadioSample]], outputs: &mut [&mut [QuadioSample]]) {
        for ((a, b), out) in inputs[0].iter().zip(inputs[1].iter()).zip(outputs[0].iter_mut()) {
            *out = *a + *b;
        }
    }
}


#[derive(Default)]
pub struct OutputNode;
impl graph::Node for OutputNode {
    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            input_sockets: vec![
                SocketDescriptor {
                    label: "Out".to_owned()
                }
            ],
            output_sockets: vec![]
        }
    }
}

impl QuadioNode for OutputNode {
    fn show_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Out");
    }

    fn process(&mut self, inputs: &[&[QuadioSample]], outputs: &mut [&mut [QuadioSample]]) {
        unimplemented!()
    }
}

pub struct PhasorNode {
    f_mul: u32,
    f_div: u32,
}
impl Default for PhasorNode {
    fn default() -> Self {
        PhasorNode { f_mul: 1, f_div: 1 }
    }
}
impl graph::Node for PhasorNode {
    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            input_sockets: vec![
                SocketDescriptor {
                    label: "Mod".to_owned()
                }
            ], output_sockets: vec![
                SocketDescriptor {
                    label: "Out".to_owned()
                }
            ]
        }
    }
}
impl QuadioNode for PhasorNode {
    fn show_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Phasor");
        ui.horizontal(|ui| {
            ui.label("FREQ MUL/DIV");
            ui.add(egui::DragValue::new(&mut self.f_mul));
            ui.add(egui::DragValue::new(&mut self.f_div).clamp_range(1..=usize::MAX));
        });
    }

    fn process(&mut self, inputs: &[&[QuadioSample]], outputs: &mut [&mut [QuadioSample]]) {
        for (_mod_in, out) in inputs[0].iter().zip(outputs[0].iter_mut()) {
            *out = QuadioSample::from(0.0)
        }
    }
}