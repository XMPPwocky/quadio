use serde::{Deserialize, Serialize};

pub mod audio;
pub mod graph;
pub mod graph_ui;
pub mod node;
pub mod sample;

use std::sync::{Arc, Mutex};

pub struct QuadioApp {
    graph: Arc<Mutex<graph::NodeGraph<Box<dyn node::QuadioNode>>>>,
    ui_disabled: bool,
}

impl QuadioApp {
    /// Called once before the first frame.
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        graph: Arc<Mutex<graph::NodeGraph<Box<dyn node::QuadioNode>>>>,
    ) -> Self {
        QuadioApp {
            graph,
            ui_disabled: false,
        }
    }
}

impl eframe::App for QuadioApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::right("side_panel").show(ctx, |ui| {
            ui.heading("quadio");
            egui::warn_if_debug_build(ui);
            ui.monospace("1ch 44100Hz, not oversampling");
            ui.monospace("1 voices");
            ui.checkbox(&mut self.ui_disabled, "Disable graph UI");
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.ui_disabled {
                return;
            }
            graph_ui::graph_ui(ui, "main_graph", &mut self.graph.lock().unwrap());
        });
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let graph: Arc<Mutex<graph::NodeGraph<Box<dyn node::QuadioNode>>>> = Default::default();
    let _audio_stream = audio::audio_main(graph.clone()).unwrap();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "quadio",
        native_options,
        Box::new(|cc| Box::new(QuadioApp::new(cc, graph))),
    )
}
