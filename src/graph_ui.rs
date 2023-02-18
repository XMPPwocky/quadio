use crate::graph::NodeGraph;
use crate::graph::NodeKey;
use crate::graph::SocketDirection;
use crate::node::QuadioNode;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(PartialEq, Eq)]
enum Selection {
    Node(NodeKey),
    Socket(NodeKey, SocketDirection, usize),
}

enum ConnectionEvent {
    Connect((NodeKey, usize), (NodeKey, usize)),
    Disconnect(NodeKey, SocketDirection, usize),
}

#[derive(Default)]
struct GraphMemory {
    selection: Option<Selection>,
    socket_positions: HashMap<(NodeKey, SocketDirection, usize), egui::Pos2>,
}
impl GraphMemory {
    fn is_node_selected(&self, node: NodeKey) -> bool {
        match self.selection {
            None => false,
            Some(Selection::Node(sel_node)) if sel_node == node => true,
            Some(Selection::Socket(sel_node, _, _)) if sel_node == node => true,
            _ => false,
        }
    }
}

type NodeConstructors = [(&'static str, &'static dyn Fn() -> Box<dyn QuadioNode>)];

#[allow(clippy::box_default)]
fn node_constructors() -> &'static NodeConstructors {
    &[
        ("Phasor", &|| {
            Box::new(crate::node::PhasorNode::default()) as _
        }),
        ("Passthru", &|| {
            Box::new(crate::node::PassthruNode::default()) as _
        }),
        ("Output", &|| {
            Box::new(crate::node::OutputNode::default()) as _
        }),
    ]
}

pub fn graph_ui<I>(ui: &mut egui::Ui, id_source: I, graph: &mut NodeGraph) -> egui::Response
where
    I: std::hash::Hash,
{
    ui.push_id(id_source, |ui| {
        let memory = ui.memory_mut(|mem| {
            mem.data.get_temp_mut_or_default::<Arc<Mutex<GraphMemory>>>(ui.id()).clone()
        });
        let mut memory = memory.lock().unwrap();

        ui.menu_button("Add", |ui| {
            for (name, ctor) in node_constructors() {
                if ui.button(*name).clicked() {
                    graph.add_node(ctor());
                    ui.close_menu();
                }
            }
        });

        let bound_rect = egui::Rect::from_min_size(ui.next_widget_position(), ui.available_size());

        let mut pending_connections = vec![];
        for (node_key, node, descriptor) in graph.nodes_mut() {
            let node_is_selected = memory.is_node_selected(node_key);

            let area_response = egui::Area::new(
                ui.id().with(node_key)
            )
            .drag_bounds(bound_rect)
            .show(ui.ctx(), |ui| {
                let node_frame = if !node_is_selected {
                    egui::Frame {
                        shadow: egui::epaint::Shadow::NONE,
                        ..egui::Frame::window(ui.style())
                    }
                } else {
                    egui::Frame {
                        fill: ui.style().visuals.selection.bg_fill,
                        shadow: egui::epaint::Shadow::NONE,
                        ..egui::Frame::window(ui.style())
                    }
                };

                node_frame.show(ui, |ui| {
                    ui.set_min_width(96.0);
                    node.show_ui(ui);

                    ui.shrink_width_to_current();

                    ({
                        ui.separator();

                        egui::Grid::new("sockets").num_columns(2)
                            .show(ui, |ui| {
                                for i in 0..std::cmp::max(descriptor.input_sockets.len(), descriptor.output_sockets.len()) {
                                    if let Some(in_desc) = descriptor.input_sockets.get(i) {
                                        let in_label = &in_desc.label;
                                        let socket_pos = egui::Pos2::new(
                                            ui.min_rect().left() - node_frame.inner_margin.left,
                                            ui.next_widget_position().y
                                        );
                                        memory.socket_positions.insert(
                                            (node_key, SocketDirection::Input, i),
                                            socket_pos);
                                        let r = ui.button(format!("> {in_label}"));
                                        if r.clicked() {
                                            if let Some(Selection::Socket(other_node, SocketDirection::Output, other_node_sock_idx)) = memory.selection {
                                                pending_connections.push(ConnectionEvent::Connect((other_node, other_node_sock_idx), (node_key, i)));
                                                memory.selection = None;
                                            } else {
                                                memory.selection = Some(Selection::Socket(node_key, SocketDirection::Input, i));
                                            }
                                        } else if r.clicked_by(egui::PointerButton::Secondary) {
                                            pending_connections.push(ConnectionEvent::Disconnect(node_key, SocketDirection::Input, i));
                                        }
                                    } else {
                                        ui.label("");
                                    }

                                    let rtl_layout = egui::Layout::right_to_left(egui::Align::Center);
                                    ui.with_layout( rtl_layout, |ui| {
                                        if let Some(out_desc) = descriptor.output_sockets.get(i) {
                                            let socket_pos = egui::Pos2::new(
                                                ui.min_rect().right() + node_frame.inner_margin.left,
                                                ui.next_widget_position().y
                                            );
                                            memory.socket_positions.insert(
                                                (node_key, SocketDirection::Output, i),
                                                socket_pos); // should be offset to be on the frame...

                                            let out_label = &out_desc.label;
                                            let r = ui.button(format!("{out_label} >"));
                                            if r.clicked() {
                                                if let Some(Selection::Socket(other_node, SocketDirection::Input, other_node_sock_idx)) = memory.selection {
                                                    pending_connections.push(ConnectionEvent::Connect((node_key, i), (other_node, other_node_sock_idx)));
                                                    memory.selection = None;
                                                } else {
                                                    memory.selection = Some(Selection::Socket(node_key, SocketDirection::Output, i));
                                                }
                                            } else if r.clicked_by(egui::PointerButton::Secondary) {
                                                pending_connections.push(ConnectionEvent::Disconnect(node_key, SocketDirection::Output, i));
                                            }
                                        } else {
                                            ui.label("");
                                        }
                                    });

                                    ui.end_row();
                                }
                            });
                    });

                });
            }).response;

            if memory.selection == Some(Selection::Node(node_key)) && area_response.clicked_elsewhere() {
                memory.selection = None;
            } else if area_response.clicked() {
                memory.selection = Some(Selection::Node(node_key));
            }
        }

        for ev in pending_connections {
            match ev {
                ConnectionEvent::Connect(src, dst) => { graph.connect(src, dst); },
                ConnectionEvent::Disconnect(node, dir, idx) => { graph.disconnect(node, dir, idx); },
            }
        }

        for (src, dst) in graph.wires() {
            let Some(&src_pos) = memory.socket_positions.get(&(src.0, SocketDirection::Input, src.1)) else {
                continue;
            };
            let Some(&dst_pos) = memory.socket_positions.get(&(dst.0, SocketDirection::Output, dst.1)) else {
                continue;
            };
            let stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(0x00, 0xD3, 0xED));

            let horiz = src_pos.x < dst_pos.x;
            let points = if horiz {
                [
                    src_pos,
                    egui::Pos2::new(src_pos.x, dst_pos.y),
                    egui::Pos2::new(dst_pos.x, src_pos.y),
                    dst_pos
                ]
            } else {
                [
                    src_pos,
                    egui::Pos2::new(dst_pos.x, src_pos.y),
                    egui::Pos2::new(src_pos.x, dst_pos.y),
                    dst_pos
                ]
            };

            ui.painter().add(egui::Shape::CubicBezier(egui::epaint::CubicBezierShape {
                points,
                stroke,
                fill: egui::Color32::TRANSPARENT,
                closed: false
            }));
        }
    }).response
}
