use serde::{Deserialize, Serialize};

pub mod node;
pub mod graph;
pub mod graph_ui;
pub mod sample;

#[derive(Default, Deserialize, Serialize)]
pub struct QuadioApp {
    graph: graph::NodeGraph<Box<dyn node::QuadioNode>>
}

impl QuadioApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }
}


impl eframe::App for QuadioApp {


    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::right("side_panel").show(ctx, |ui| {
            ui.heading("quadio");
            egui::warn_if_debug_build(ui);
            ui.monospace("1ch 44100Hz, not oversampling, audio thread DOWN");
            ui.monospace("0 voices");
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            graph_ui::graph_ui(ui, "main_graph", &mut self.graph);
        });
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "quadio",
        native_options,
        Box::new(|cc| Box::new(QuadioApp::new(cc))),
    )
}