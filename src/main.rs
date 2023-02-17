pub mod audio;
pub mod graph;
pub mod graph_ui;
pub mod math;
pub mod node;
pub mod param;
pub mod sample;

use std::sync::{Arc, Mutex};

pub struct QuadioApp {
    graph: Arc<Mutex<graph::NodeGraph<Box<dyn node::QuadioNode>>>>,
    ui_disabled: bool,
    peeper: egui_extras::RetainedImage
}

impl QuadioApp {
    /// Called once before the first frame.
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        graph: Arc<Mutex<graph::NodeGraph<Box<dyn node::QuadioNode>>>>,
    ) -> Self {
        let peeper = egui_extras::RetainedImage::from_image_bytes(
            "peeper", include_bytes!("peeper.png"))
            .unwrap();
        
        QuadioApp {
            graph,
            ui_disabled: false,
            peeper
        }
    }
}

impl eframe::App for QuadioApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        egui::SidePanel::right("side_panel").show(ctx, |ui| {
            ui.heading("quadio");
            egui::warn_if_debug_build(ui);
            ui.monospace("1ch 44100Hz, not oversampling");
            ui.monospace("1 voices");
            ui.checkbox(&mut self.ui_disabled, "Disable graph UI");
        });

        let mut frame = egui::Frame {
            ..egui::Frame::central_panel(&ctx.style())
        };
        frame.inner_margin.bottom = 0.0;

        egui::CentralPanel::default().frame(frame)
            .show(ctx, |ui| {
            if self.ui_disabled {
                return;
            }

            {
                let mut ui = ui.child_ui(ui.max_rect(), egui::Layout::bottom_up(egui::Align::Min));
                self.peeper.show_scaled(&mut ui, 0.33);
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
