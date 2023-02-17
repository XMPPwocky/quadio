use core::f32::consts::TAU;
use core::ops::RangeInclusive;
use num_complex::Complex32;
use serde::{Deserialize, Serialize};

use crate::audio::AudioContext;
use crate::graph::{self, NodeDescriptor, SocketDescriptor};

use crate::sample::QuadioSample;

pub trait QuadioNode: graph::Node + Send + Sync + std::any::Any {
    fn show_ui(&mut self, ui: &mut egui::Ui);

    fn process(&mut self, ctx: &AudioContext, inputs: &[&[QuadioSample]], outputs: &mut [&mut [QuadioSample]]);
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
            input_sockets: vec![SocketDescriptor {
                label: "In".to_owned(),
            }],
            output_sockets: vec![SocketDescriptor {
                label: "Out".to_owned(),
            }],
        }
    }
}
impl QuadioNode for PassthruNode {
    fn show_ui(&mut self, ui: &mut egui::Ui) {
        ui.label("PASSTHRU");
    }

    fn process(&mut self, _ctx: &AudioContext, inputs: &[&[QuadioSample]], outputs: &mut [&mut [QuadioSample]]) {
        for (inp, out) in inputs[0].iter().zip(outputs[0].iter_mut()) {
            *out = *inp;
        }
    }
}

/*
#[derive(Default)]
pub struct SumNode;
impl graph::Node for SumNode {
    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            input_sockets: vec![
                SocketDescriptor {
                    label: "A".to_owned(),
                },
                SocketDescriptor {
                    label: "B".to_owned(),
                },
            ],
            output_sockets: vec![SocketDescriptor {
                label: "Out".to_owned(),
            }],
        }
    }
}
impl QuadioNode for SumNode {
    fn show_ui(&mut self, ui: &mut egui::Ui) {
        ui.label("SUM");
    }

    fn process(&mut self, _ctx: &AudioContext, inputs: &[&[QuadioSample]], outputs: &mut [&mut [QuadioSample]]) {
        for ((a, b), out) in inputs[0]
            .iter()
            .zip(inputs[1].iter())
            .zip(outputs[0].iter_mut())
        {
            *out = *a + *b;
        }
    }
}

#[derive(Default)]
pub struct ProductNode;
impl graph::Node for ProductNode {
    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            input_sockets: vec![
                SocketDescriptor {
                    label: "A".to_owned(),
                },
                SocketDescriptor {
                    label: "B".to_owned(),
                },
            ],
            output_sockets: vec![SocketDescriptor {
                label: "Out".to_owned(),
            }],
        }
    }
}
impl QuadioNode for ProductNode {
    fn show_ui(&mut self, ui: &mut egui::Ui) {
        ui.label("PRODUCT");
    }

    fn process(&mut self, _ctx: &AudioContext, inputs: &[&[QuadioSample]], outputs: &mut [&mut [QuadioSample]]) {
        for ((a, b), out) in inputs[0]
            .iter()
            .zip(inputs[1].iter())
            .zip(outputs[0].iter_mut())
        {
            *out = *a * *b;
        }
    }
}

fn edit_complex(
    ui: &mut egui::Ui,
    label: impl Into<egui::RichText>,
    c: &mut Complex32,
    r_range: RangeInclusive<f32>,
    logarithmic: bool,
) {
    let (mut r, mut theta) = c.to_polar();
    ui.horizontal(|ui| {
        ui.monospace(label);
        ui.add(egui::Slider::new(&mut r, r_range).logarithmic(logarithmic));
        ui.drag_angle(&mut theta);
    });
    *c = Complex32::from_polar(r, theta);
}

#[derive(Default)]
pub struct LinearNode {
    m: Complex32,
    b: Complex32,
}
impl graph::Node for LinearNode {
    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            input_sockets: vec![SocketDescriptor {
                label: "In".to_owned(),
            }],
            output_sockets: vec![SocketDescriptor {
                label: "Out".to_owned(),
            }],
        }
    }
}
impl QuadioNode for LinearNode {
    fn show_ui(&mut self, ui: &mut egui::Ui) {
        ui.label("LINEAR");

        edit_complex(ui, "m", &mut self.m, 0.0..=1024., true);
        edit_complex(ui, "b", &mut self.b, 0.0..=1024., true);
    }

    fn process(&mut self, _ctx: &AudioContext, inputs: &[&[QuadioSample]], outputs: &mut [&mut [QuadioSample]]) {
        for (x, out) in inputs[0].iter().zip(outputs[0].iter_mut()) {
            *out = (self.m * *x) + self.b
        }
    }
}

#[derive(Default)]
pub struct PhaseScaleNode {
    scale: f32,
}
impl graph::Node for PhaseScaleNode {
    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            input_sockets: vec![SocketDescriptor {
                label: "In".to_owned(),
            }],
            output_sockets: vec![SocketDescriptor {
                label: "Out".to_owned(),
            }],
        }
    }
}
impl QuadioNode for PhaseScaleNode {
    fn show_ui(&mut self, ui: &mut egui::Ui) {
        ui.label("PHASE-SCALE");

        ui.add(
            egui::Slider::new(&mut self.scale, 0.0..=32.0)
                .logarithmic(true)
                .text("Scale"),
        );
    }

    fn process(&mut self, _ctx: &AudioContext, inputs: &[&[QuadioSample]], outputs: &mut [&mut [QuadioSample]]) {
        for (inp, out) in inputs[0].iter().zip(outputs[0].iter_mut()) {
            let (r, theta) = inp.to_polar();
            *out = Complex32::from_polar(r, theta * self.scale);
        }
    }
}

#[derive(Default)]
pub struct MagAngSwitchNode;
impl graph::Node for MagAngSwitchNode {
    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            input_sockets: vec![SocketDescriptor {
                label: "In".to_owned(),
            }],
            output_sockets: vec![SocketDescriptor {
                label: "Out".to_owned(),
            }],
        }
    }
}
impl QuadioNode for MagAngSwitchNode {
    fn show_ui(&mut self, ui: &mut egui::Ui) {
        ui.label("MAG-ANG SWITCH");
    }

    fn process(&mut self, _ctx: &AudioContext, inputs: &[&[QuadioSample]], outputs: &mut [&mut [QuadioSample]]) {
        for (inp, out) in inputs[0].iter().zip(outputs[0].iter_mut()) {
            let (r, theta) = inp.to_polar();
            *out = Complex32::from_polar(theta / std::f32::consts::PI, r * std::f32::consts::PI);
        }
    }
}


#[derive(Default)]
pub struct ReImSplitNode;
impl graph::Node for ReImSplitNode {
    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            input_sockets: vec![SocketDescriptor {
                label: "In".to_owned(),
            }],
            output_sockets: vec![SocketDescriptor {
                label: "Re".to_owned(),
            },
            SocketDescriptor {
                label: "Im".to_owned(),
            }],
        }
    }
}
impl QuadioNode for ReImSplitNode {
    fn show_ui(&mut self, ui: &mut egui::Ui) {
        ui.monospace("RE-IM SPLIT");
    }

    fn process(&mut self, _ctx: &AudioContext, inputs: &[&[QuadioSample]], outputs: &mut [&mut [QuadioSample]]) {
        let (a, b) = outputs.split_at_mut(1);
        for (inp, (out_re, out_im)) in inputs[0].iter().zip(a[0].iter_mut().zip(b[0].iter_mut())) {
            *out_re = Complex32::new(inp.re, 0.0);
            *out_im = Complex32::new(0.0, inp.im);
        }
    }
}

pub struct QuadrantNode {
    scales: [Complex32; 4],
}
impl Default for QuadrantNode {
    fn default() -> Self {
     QuadrantNode { scales: [Complex32::new(1.0, 0.0); 4] }
    }
}
impl graph::Node for QuadrantNode {
    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            input_sockets: vec![SocketDescriptor {
                label: "In".to_owned(),
            }],
            output_sockets: vec![SocketDescriptor {
                label: "Out".to_owned(),
            }],
        }
    }
}
impl QuadioNode for QuadrantNode {
    fn show_ui(&mut self, ui: &mut egui::Ui) {
        ui.monospace("QUADRANT");

        edit_complex(ui, "I", &mut self.scales[0], 0.0..=32.0, false);
        edit_complex(ui, "II", &mut self.scales[1], 0.0..=32.0, false);
        edit_complex(ui, "II", &mut self.scales[2], 0.0..=32.0, false);
        edit_complex(ui, "IV", &mut self.scales[3], 0.0..=32.0, false);
    }

    fn process(&mut self, _ctx: &AudioContext, inputs: &[&[QuadioSample]], outputs: &mut [&mut [QuadioSample]]) {
        for (inp, out) in inputs[0].iter().zip(outputs[0].iter_mut()) {
            let scale = match (inp.re.is_sign_positive(), inp.im.is_sign_positive()) {
                (true, true) => self.scales[0],
                (false, true) => self.scales[1],
                (false, false) => self.scales[2],
                (true, false) => self.scales[3],
            };
            *out = inp * scale;
        }
    }
}

pub struct QuantizeNode {
    amp_factor: f32,
    phase_factor: f32
}
impl Default for QuantizeNode {
    fn default() -> Self {
     QuantizeNode { amp_factor: 2.0f32.powf(16.0), phase_factor: 2.0f32.powf(16.0) }
    }
}
impl graph::Node for QuantizeNode {
    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            input_sockets: vec![SocketDescriptor {
                label: "In".to_owned(),
            }],
            output_sockets: vec![SocketDescriptor {
                label: "Out".to_owned(),
            }],
        }
    }
}
impl QuadioNode for QuantizeNode {
    fn show_ui(&mut self, ui: &mut egui::Ui) {
        ui.monospace("QUANTIZE");

        let mut amp_bits = self.amp_factor.log2();
        ui.add(egui::DragValue::new(&mut amp_bits).clamp_range(1.0..=16.0));
        self.amp_factor = 2.0f32.powf(amp_bits);
        
        let mut phase_bits = self.phase_factor.log2();
        ui.add(egui::DragValue::new(&mut phase_bits).clamp_range(1.0..=16.0));
        self.phase_factor = 2.0f32.powf(phase_bits);
    }

    fn process(&mut self, _ctx: &AudioContext, inputs: &[&[QuadioSample]], outputs: &mut [&mut [QuadioSample]]) {
        for (inp, out) in inputs[0].iter().zip(outputs[0].iter_mut()) {
            let (amp, phase) = inp.to_polar();
            let amp = (amp * self.amp_factor).round() / self.amp_factor;
            let phase = (phase * self.phase_factor / TAU).round() / self.phase_factor;
            *out = Complex32::from_polar(amp, phase * TAU);
        }
    }
}

pub struct SlomoNode {
    alpha: f32,
    
    last_phase: f32,
}
impl Default for SlomoNode {
    fn default() -> Self {
     SlomoNode { alpha: 0.9, last_phase: 0.0 }
    }
}
impl graph::Node for SlomoNode {
    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            input_sockets: vec![SocketDescriptor {
                label: "In".to_owned(),
            }],
            output_sockets: vec![SocketDescriptor {
                label: "Out".to_owned(),
            }],
        }
    }
}
impl QuadioNode for SlomoNode {
    fn show_ui(&mut self, ui: &mut egui::Ui) {
        ui.monospace("SLO-MO");

        ui.add(egui::Slider::new(&mut self.alpha, 0.0..=1.0).logarithmic(true));
    }

    fn process(&mut self, _ctx: &AudioContext, inputs: &[&[QuadioSample]], outputs: &mut [&mut [QuadioSample]]) {
        for (inp, out) in inputs[0].iter().zip(outputs[0].iter_mut()) {
            let (amp, phase) = inp.to_polar();
            let phase = crate::math::lerp(self.last_phase, phase, 1.0 - self.alpha);
            self.last_phase = phase;
            *out = Complex32::from_polar(amp, phase);
        }
    }
}

pub struct ScopeNode {
    length: usize,
    depth: bool,

    triggered: bool,
    free_run: bool,
    last_sample: QuadioSample,

    full_waveform: Vec<QuadioSample>,
    capture_buf: Vec<QuadioSample>
}
impl Default for ScopeNode {
    fn default() -> Self {
        ScopeNode { length: 4096,
            triggered: false,
            free_run: false,
            depth: false,
            full_waveform: Vec::new(),
            last_sample: 0.0.into(),
            capture_buf: Vec::new()
        }
    }
}
impl graph::Node for ScopeNode {
    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            input_sockets: vec![SocketDescriptor {
                label: "In".to_owned(),
            }],
            output_sockets: vec![SocketDescriptor {
                label: "Out".to_owned(),
            }],
        }
    }
}
impl QuadioNode for ScopeNode {
    fn show_ui(&mut self, ui: &mut egui::Ui) {
        ui.monospace("SCOPE");
        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(&mut self.length).clamp_range(1..=16384));
            ui.checkbox(&mut self.depth, "3D");
            ui.checkbox(&mut self.free_run, "FREE");
        });



        let deepen = |i| {
            1.0 + if self.depth { 8.0 * (i as f64 / self.full_waveform.len() as f64) } else { 0.0 }
        };
        let points: egui::plot::PlotPoints = self.full_waveform.iter()
            .enumerate()
            .map(|(i, sample)| (deepen(i), sample))
            .map(|(depth, sample)| [sample.re as f64 / depth, sample.im as f64 / depth])
            .collect();
        
        let line = egui::plot::Line::new(points);
        egui::plot::Plot::new("plot").view_aspect(1.0)
            .width(256.0)
            .height(256.0)
            .data_aspect(1.0)
            .center_x_axis(true)
            .center_y_axis(true).show(ui, |plot_ui| {
                plot_ui.set_plot_bounds(egui::plot::PlotBounds::from_min_max([-1.0, -1.0], [1.0,  1.0]));
                plot_ui.line(line);
            });
    }

    fn process(&mut self, _ctx: &AudioContext, inputs: &[&[QuadioSample]], outputs: &mut [&mut [QuadioSample]]) {
        for inp in inputs[0] {
            if (inp.im.is_sign_positive() && self.last_sample.im.is_sign_negative()) || self.free_run {
                self.triggered = true;
            }
            self.last_sample = *inp;

            if self.triggered {
                self.capture_buf.push(*inp);
                if self.capture_buf.len() >= self.length {
                    std::mem::swap(&mut self.capture_buf, &mut self.full_waveform);
                    self.capture_buf.clear();
                    self.triggered = false;
                }
            } 
        }

        outputs[0].copy_from_slice(inputs[0])
    }
}

*/

#[derive(Default)]
pub struct OutputNode;
impl graph::Node for OutputNode {
    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            input_sockets: vec![SocketDescriptor {
                label: "Out".to_owned(),
            }],
            output_sockets: vec![],
        }
    }
}

impl QuadioNode for OutputNode {
    fn show_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Out");
    }

    fn process(&mut self, _ctx: &AudioContext, _inputs: &[&[QuadioSample]], _outputs: &mut [&mut [QuadioSample]]) {
        // nop, this one's magic
    }
}

/*

pub struct PhasorNode {
    f_mul: f32,
    f_div: f32,

    mod_mag_scale: f32,
    mod_ang_scale: f32,

    phase: f32,

    last_mod_phase: f32,
}
impl Default for PhasorNode {
    fn default() -> Self {
        PhasorNode {
            f_mul: 1.0,
            f_div: 1.0,
            mod_mag_scale: 0.0,
            mod_ang_scale: 0.1,
            phase: 0.0,

            last_mod_phase: 0.0,
        }
    }
}
impl graph::Node for PhasorNode {
    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            input_sockets: vec![SocketDescriptor {
                label: "Mod".to_owned(),
            }],
            output_sockets: vec![SocketDescriptor {
                label: "Out".to_owned(),
            }],
        }
    }
}
impl QuadioNode for PhasorNode {
    fn show_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Phasor");
        ui.horizontal(|ui| {
            ui.monospace("FREQ MUL/DIV");
            ui.add(egui::DragValue::new(&mut self.f_mul));
            ui.add(egui::DragValue::new(&mut self.f_div).clamp_range(0.001..=f32::MAX));
        });
        ui.monospace("MOD MAG-FM SCL");
        ui.add(egui::Slider::new(&mut self.mod_mag_scale, 0.0..=8.0).logarithmic(true));
        ui.monospace("MOD ANG SCL");
        ui.add(egui::Slider::new(&mut self.mod_ang_scale, 0.0..=32.0).logarithmic(true));
    }

    fn process(&mut self, _ctx: &AudioContext, inputs: &[&[QuadioSample]], outputs: &mut [&mut [QuadioSample]]) {
        for (mod_in, out) in inputs[0].iter().zip(outputs[0].iter_mut()) {
            let (_mod_mag, mod_ang) = mod_in.to_polar();

            let est_mod_freq = crate::math::clean_angle_radians(mod_ang - self.last_mod_phase);

            let mod_f_scale = 0.02 * (mod_in.re * self.mod_mag_scale);

            self.phase += 0.02 * self.f_mul / self.f_div; // main accumulator
            self.phase += est_mod_freq * self.mod_ang_scale; // PM (previously differentiated)
            self.phase += mod_f_scale; // FM
            self.phase %= TAU;

            *out = QuadioSample::from_polar(
                1.0,
                self.phase
            );

            self.last_mod_phase = mod_ang;
        }
    }
}
*/