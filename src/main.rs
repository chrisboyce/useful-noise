use egui_plot::{Line, Plot, PlotPoints};
use glicol_synth::filter::ResonantHighPassFilter;
use glicol_synth::Node;
use nannou::prelude::*;
use nannou_audio as audio;
use nannou_audio::Buffer;
use nannou_egui::{self, egui, Egui};
use petgraph::graph::NodeIndex;
use rustfft::{num_complex::Complex, FftPlanner};
use std::f64::consts::PI;
use std::sync::{Arc, RwLock};
use std::{cell::OnceCell, sync::OnceLock};

const SAMPLE_RATE: usize = 44_1000;
const FRAME_SIZE: usize = 64;
use glicol_synth::{
    filter::ResonantLowPassFilter,
    operator::{Add, Mul},
    oscillator::{SawOsc, SinOsc},
    signal::{ConstSig, Noise},
    AudioContext as GlicolAudioContext, AudioContextBuilder, Message, Sum,
};
type AudioContext = GlicolAudioContext<FRAME_SIZE>;
fn main() {
    // let b = context.next_block();
    nannou::app(model).update(update).run();
}

struct Model {
    egui: Egui,
    settings: Settings,
    stream: audio::Stream<Audio>,
    // stream: audio::Stream<Audio>,
}

struct Settings {
    brownish_noise_knob_a: f32,
    brownish_noise_volume: f32,
    low_pass_freq: f32,
    high_pass_freq: f32,
}

struct BrownishNoise {
    brown_walk_scale: f32,
    previous_brown_value: f32,
}

impl BrownishNoise {
    fn new() -> Self {
        Self {
            brown_walk_scale: 0.01,
            previous_brown_value: 0.,
        }
    }
}

impl<const N: usize> Node<N> for BrownishNoise {
    fn process(
        &mut self,
        _inputs: &mut hashbrown::HashMap<usize, glicol_synth::Input<N>>,
        output: &mut [glicol_synth::Buffer<N>],
    ) {
        let mut value = self.previous_brown_value;
        for i in 0..N {
            // value += random_walk_value;
            // To keep our signal within sane values, we enforce that the next
            // value is within the  -1..1 range
            let random_walk_value = self.brown_walk_scale * (fastrand::f32() * 2. - 1.0);
            let new_value = {
                let new_value_before_bounds = value + random_walk_value;
                // println!("A {new_value_before_bounds}");
                if new_value_before_bounds > 1.0 {
                    // let bounce = new_value_before_bounds - 1.0;
                    // let value_after_bounce = 1.0 - bounce;
                    // value_after_bounce
                    1.0
                } else if new_value_before_bounds < -1.0 {
                    // let bounce = new_value_before_bounds + 1.0;
                    // let value_after_bounce = 1.0 + bounce;
                    // value_after_bounce
                    -1.0
                } else {
                    new_value_before_bounds
                }
            };
            if new_value - value < -1. {
                println!("Value difference {}", new_value - value);
            }
            value = new_value;
            for j in 0..output.len() {
                output[j][i] = value;
            }
        }

        self.previous_brown_value = value;
    }

    fn send_msg(&mut self, info: glicol_synth::Message) {
        match info {
            Message::SetParam(0, glicol_synth::GlicolPara::Number(scale)) => {
                self.brown_walk_scale = scale
            }
            _ => todo!(),
        }
    }
}

struct Audio {
    fft: Vec<Complex<f32>>,
    brownish_node_index: NodeIndex,
    brownish_node_volume_index: NodeIndex,
    brownish_high_pass_index: NodeIndex,
    brownish_low_pass_index: NodeIndex,
    context: AudioContext,
}

fn update(_app: &App, model: &mut Model, update: Update) {
    let egui = &mut model.egui;
    let settings = &mut model.settings;

    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();
    let f = settings.brownish_noise_knob_a;
    let volume = settings.brownish_noise_volume;

    model.stream.send(move |audio: &mut Audio| {
        audio.context.send_msg(
            audio.brownish_node_index,
            Message::SetParam(0, glicol_synth::GlicolPara::Number(f)),
        )
    });

    model.stream.send(move |audio: &mut Audio| {
        audio.context.send_msg(
            audio.brownish_node_volume_index,
            Message::SetToNumber(0, volume),
        )
    });

    // let f = settings.high_pass_freq;
    // model.stream.send(move |audio: &mut Audio| {
    //     audio
    //         .context
    //         .send_msg(audio.brownish_high_pass_index, Message::SetToNumber(0, f))
    // });
    let f = settings.low_pass_freq;
    model.stream.send(move |audio: &mut Audio| {
        audio
            .context
            .send_msg(audio.brownish_low_pass_index, Message::SetToNumber(0, f))
    });

    egui::Window::new("Brownish Noise").show(&ctx, |ui| {
        ui.label("Volume");
        ui.add(egui::Slider::new(
            &mut settings.brownish_noise_volume,
            0.0..=1.0,
        ));
        ui.label("Low Pass");
        ui.add(egui::Slider::new(
            &mut settings.low_pass_freq,
            0.0..=10000.0,
        ));
        // ui.label("High Pass");
        // ui.add(egui::Slider::new(
        //     &mut settings.high_pass_freq,
        //     0.0..=20000.0,
        // ));

        ui.label("Knob A");
        ui.add(egui::Slider::new(
            &mut settings.brownish_noise_knob_a,
            0.0..=1.0,
        ));
    });
}

fn model(app: &App) -> Model {
    // Create a window to receive key pressed events.
    let window_id = app
        .new_window()
        .size(200, 200)
        .key_pressed(key_pressed)
        .view(view)
        .raw_event(raw_window_event)
        .build()
        .unwrap();

    let window = app.window(window_id).unwrap();
    let egui = Egui::from_window(&window);

    // Initialise the audio API so we can spawn an audio stream.
    let audio_host = audio::Host::new();

    // Initialise the state that we want to live on the audio thread.
    let mut context = AudioContextBuilder::<64>::new()
        .sr(SAMPLE_RATE)
        .channels(2)
        .build();

    let sin1 = context.add_stereo_node(SinOsc::new().freq(440.0));
    // let sin2 = context.add_stereo_node(SinOsc::new().freq(80.0));
    let noise = BrownishNoise::new();
    let noise_id = context.add_stereo_node(noise);
    let noise_high_pass = context.add_stereo_node(ResonantHighPassFilter::new().cutoff(1000.0));
    let noise_low_pass = context.add_stereo_node(ResonantLowPassFilter::new().cutoff(1000.0));
    let noise_volume = context.add_stereo_node(Mul::new(1.));
    context.chain(vec![
        noise_id,
        noise_low_pass,
        noise_volume,
        context.destination,
    ]);

    // let mix = context.add_stereo_node(Sum {});
    // context.chain(vec![sin1, context.destination]);
    // context.chain(vec![sin2, context.destination]);
    let model = Audio {
        context,
        fft: Vec::with_capacity(64),
        brownish_node_index: noise_id,
        brownish_node_volume_index: noise_volume,
        brownish_high_pass_index: noise_high_pass,
        brownish_low_pass_index: noise_low_pass,
    };

    let mut brown = 0.;
    let stream = audio_host
        // .new_output_stream(brown)
        // .render(render_brownian_noise)
        .new_output_stream(model)
        .render(render_audio)
        .build()
        .unwrap();

    stream.play().unwrap();

    Model {
        egui,
        stream,
        settings: Settings {
            brownish_noise_knob_a: 0.07,
            brownish_noise_volume: 0.65,
            low_pass_freq: 10000.0,
            high_pass_freq: 0.0,
        },
    }
}
/// Copies the audio data from the glicol buffer into the nannou audio buffer
fn render_audio(audio: &mut Audio, buffer: &mut Buffer) {
    let block = audio.context.next_block();
    for (frame_index, frame) in buffer.frames_mut().enumerate() {
        for channel_index in 0..frame.len() {
            frame[channel_index] = block[channel_index][frame_index];
        }
    }
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    match key {
        Key::A => {
            // model.stream.send(|audio: &mut Audio| {
            //     audio
            //         .context
            //         .send_msg(audio.tone, Message::SetToNumber(0, 300.))
            // });
        }
        _ => {}
    }
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    // Let egui handle things like keyboard and mouse input.
    model.egui.handle_raw_event(event);
}
fn view(app: &App, model: &Model, frame: Frame) {
    let settings = &model.settings;

    let draw = app.draw();
    draw.background().color(BLACK);

    draw.to_frame(app, &frame).unwrap();
    model.egui.draw_to_frame(&frame).unwrap();
}
