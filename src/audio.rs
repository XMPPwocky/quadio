use anyhow;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, Sample, SizedSample,
};
use std::any::TypeId;
use std::sync::mpsc;

use crate::{
    graph::{NodeGraph, NodeKey},
    node::QuadioNode,
    sample::QuadioSample,
};

use std::sync::{Arc, Mutex};

type SharedGraph = Arc<Mutex<NodeGraph<Box<dyn QuadioNode>>>>;

pub struct AudioIO {
    #[allow(dead_code)] // just keeping it alive
    stream: cpal::Stream,
}

enum DfsState {
    NotVisited,
    Visiting,
    Visited,
}
pub struct AudioEngine {
    buffers: slotmap::SecondaryMap<NodeKey, (DfsState, Vec<Vec<QuadioSample>>)>,
    zeroes_buf: Vec<QuadioSample>,

    sample_rate: f32,
    channels: usize,
    block_size: usize,
}
impl AudioEngine {
    pub fn new(sample_rate: f32, channels: usize) -> AudioEngine {
        AudioEngine {
            buffers: slotmap::SecondaryMap::new(),
            zeroes_buf: Vec::with_capacity(1024),
            block_size: 1024,
            sample_rate,
            channels,
        }
    }
}

pub fn audio_main(graph: SharedGraph) -> anyhow::Result<AudioIO> {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .expect("failed to find output device");
    println!("Output device: {}", device.name()?);

    let config = device.default_output_config().unwrap();
    println!("Default output config: {config:?}");

    match config.sample_format() {
        cpal::SampleFormat::I8 => run::<i8>(graph, &device, &config.into()),
        cpal::SampleFormat::I16 => run::<i16>(graph, &device, &config.into()),
        // cpal::SampleFormat::I24 => run::<I24>(&device, &config.into()),
        cpal::SampleFormat::I32 => run::<i32>(graph, &device, &config.into()),
        // cpal::SampleFormat::I48 => run::<I48>(&device, &config.into()),
        cpal::SampleFormat::I64 => run::<i64>(graph, &device, &config.into()),
        cpal::SampleFormat::U8 => run::<u8>(graph, &device, &config.into()),
        cpal::SampleFormat::U16 => run::<u16>(graph, &device, &config.into()),
        // cpal::SampleFormat::U24 => run::<U24>(&device, &config.into()),
        cpal::SampleFormat::U32 => run::<u32>(graph, &device, &config.into()),
        // cpal::SampleFormat::U48 => run::<U48>(&device, &config.into()),
        cpal::SampleFormat::U64 => run::<u64>(graph, &device, &config.into()),
        cpal::SampleFormat::F32 => run::<f32>(graph, &device, &config.into()),
        cpal::SampleFormat::F64 => run::<f64>(graph, &device, &config.into()),
        sample_format => panic!("Unsupported sample format '{sample_format}'"),
    }
}

fn run<T>(
    graph: SharedGraph,
    device: &cpal::Device,
    config: &cpal::StreamConfig,
) -> Result<AudioIO, anyhow::Error>
where
    T: SizedSample + FromSample<f32>,
{
    let block_queue_max = 2;
    let (tx, rx) = mpsc::sync_channel::<Vec<f32>>(block_queue_max);

    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    let err_fn = |err| eprintln!("an error occurred on stream: {err}");

    let mut main_buf: Vec<_> = std::iter::repeat(0.0f32).take(1024).collect();
    let mut cursor = main_buf.len() - 1;
    let mut engine = AudioEngine::new(sample_rate, channels);

    std::thread::spawn(move || {
        loop {
            // fixme: reuse this or whatever
            let mut prod_buf: Vec<_> = std::iter::repeat(0.0f32).take(1024).collect();

            engine.run_graph(&mut graph.lock().unwrap(), &mut prod_buf);
            // we will block here (backpressure)
            tx.send(prod_buf).unwrap();
        }
    });
    let mut next_value = move || {
        if cursor == main_buf.len() {
            main_buf = rx.recv().unwrap();
            cursor = 0;
        }

        let rv = main_buf[cursor];
        cursor += 1;
        rv
    };

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &mut next_value)
        },
        err_fn,
        None,
    )?;
    stream.play()?;

    Ok(AudioIO { stream })
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: Sample + FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        let value: T = T::from_sample(next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}

impl AudioEngine {
    fn run_graph(&mut self, graph: &mut NodeGraph<Box<dyn QuadioNode>>, output: &mut [f32]) {
        self.block_size = output.len();

        self.zeroes_buf
            .resize(self.block_size, QuadioSample::from(0.0));

        for (node_key, _) in graph.nodes() {
            if !self.buffers.contains_key(node_key) {
                self.buffers
                    .insert(node_key, (DfsState::NotVisited, Vec::new()));
            }
        }

        for (node_key, (dfs_state, bufs)) in self.buffers.iter_mut() {
            *dfs_state = DfsState::NotVisited;

            let num_outputs = graph.node_descriptor(node_key).output_sockets.len();
            bufs.resize(num_outputs, Vec::new());
            for buf in bufs {
                buf.resize(self.block_size, QuadioSample::from(0.0));
            }
        }

        let output_node = {
            let mut outputs = graph.nodes().filter(|(_key, node)| {
                // this sucks a lot
                (***node).type_id() == TypeId::of::<crate::node::OutputNode>()
            });

            let Some((first_output, _)) = outputs.next() else {
                // no outputs no audio
                output.fill(0.0);
                return;
            };

            first_output
        };

        self.run_graph_node(graph, output_node);

        let Some((src_node, src_idx)) = graph.src_for_dest(output_node, 0) else {
                     // unconnected output, no audio
                     output.fill(0.0);
                     return;
        };

        for (computed_sample, output_sample) in self.buffers[src_node].1[src_idx]
            .iter()
            .zip(output.iter_mut())
        {
            *output_sample = computed_sample.re; // output real part only
        }
    }

    fn run_graph_node(&mut self, graph: &mut NodeGraph<Box<dyn QuadioNode>>, node: NodeKey) {
        let (state, _) = &mut self.buffers[node];

        match state {
            DfsState::NotVisited => {
                self.buffers[node].0 = DfsState::Visiting;

                let num_inputs = graph.node_descriptor(node).input_sockets.len();
                let num_outputs = graph.node_descriptor(node).output_sockets.len();

                let mut inputs_ind: Vec<_> = std::iter::repeat(None).take(num_inputs).collect();

                for i in 0..num_inputs {
                    if let Some((src_node, src_idx)) = graph.src_for_dest(node, i) {
                        self.run_graph_node(graph, src_node);
                        inputs_ind[i] = Some((src_node, src_idx));
                    }
                }

                let inputs: Vec<_> = inputs_ind
                    .into_iter()
                    .map(|x| {
                        if let Some((n, i)) = x {
                            &self.buffers[n].1[i]
                        } else {
                            &self.zeroes_buf
                        }
                        .as_slice()
                    })
                    .collect();
                let mut outputs_all: Vec<QuadioSample> = std::iter::repeat(QuadioSample::from(0.0))
                    .take(self.block_size * num_outputs)
                    .collect();
                let mut outputs: Vec<_> = outputs_all.chunks_mut(self.block_size).collect();

                graph.get_node_mut(node).process(&inputs, &mut outputs);

                for (i, output_tmp) in outputs.into_iter().enumerate() {
                    self.buffers[node].1[i].copy_from_slice(output_tmp)
                }
                self.buffers[node].0 = DfsState::Visited;
            }
            DfsState::Visiting => {
                eprintln!("cycle... uh...oh...");
            }
            DfsState::Visited => {
                // nop!
            }
        }
    }
}
