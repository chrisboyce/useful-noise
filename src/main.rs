use egui_plot::{Line, Plot, PlotPoints};
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
    frequency: f32,
    resolution: u32,
    scale: f32,
    rotation: f32,
    color: Srgb<u8>,
    position: Vec2,
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

// for i in 0..N {
//     for j in 0..output.len() {
//         output[j][i] = (self.phase * 2.0 * std::f32::consts::PI).sin();
//     }
//     self.phase += self.freq / self.sr as f32;
//     if self.phase > 1.0 {
//         self.phase -= 1.0
//     }
// }
impl<const N: usize> Node<N> for BrownishNoise {
    fn process(
        &mut self,
        _inputs: &mut hashbrown::HashMap<usize, glicol_synth::Input<N>>,
        output: &mut [glicol_synth::Buffer<N>],
    ) {
        let mut value = self.previous_brown_value;
        // let r = fastrand::f32();
        // println!("Random Walk: {r}");
        for i in 0..N {
            // value += random_walk_value;
            // To keep our signal within sane values, we enforce that the next
            // value is within the  -1..1 range
            let random_walk_value = self.brown_walk_scale * (fastrand::f32() * 2. - 1.0);
            value = {
                let new_value_before_bounds = value + random_walk_value;
                // println!("A {new_value_before_bounds}");
                if new_value_before_bounds > 1.0 {
                    let bounce = new_value_before_bounds - 1.0;
                    let value_after_bounce = 1.0 - bounce;
                    value_after_bounce
                } else if new_value_before_bounds < -1.0 {
                    let bounce = new_value_before_bounds + 1.0;
                    let value_after_bounce = 1.0 + bounce;
                    value_after_bounce
                } else {
                    new_value_before_bounds
                }
            };

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

struct NoiseContext {
    brown_walk_scale: f32,
    previous_brown_value: f32,
}

struct Audio {
    fft: Vec<Complex<f32>>,
    tone: NodeIndex,
    context: AudioContext,
}

static FFT: OnceLock<Vec<Complex<f32>>> = OnceLock::new();
fn update(_app: &App, model: &mut Model, update: Update) {
    let egui = &mut model.egui;
    let settings = &mut model.settings;

    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();
    let f = settings.frequency;

    model.stream.send(move |audio: &mut Audio| {
        audio.context.send_msg(
            audio.tone,
            Message::SetParam(0, glicol_synth::GlicolPara::Number(f)),
        )
    });

    egui::Window::new("Settings").show(&ctx, |ui| {
        ui.label("Walk Scaler:");
        ui.add(egui::Slider::new(&mut settings.frequency, -1.0..=1.0));
    });
}
fn model(app: &App) -> Model {
    // Create a window to receive key pressed events.
    let window_id = app
        .new_window()
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
    context.chain(vec![noise_id, context.destination]);

    // let mix = context.add_stereo_node(Sum {});
    // context.chain(vec![sin1, context.destination]);
    // context.chain(vec![sin2, context.destination]);
    let model = Audio {
        context,
        fft: Vec::with_capacity(64),
        tone: noise_id,
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
            resolution: 10,
            scale: 200.0,
            rotation: 0.0,
            color: WHITE,
            position: vec2(0.0, 0.0),
            frequency: 440.0,
        },
    }
}

// fn render_brownian_noise(context: &mut NoiseContext, buffer: &mut Buffer) {
//     let mut value = *context.previous_brown_value;
//     for (frame_index, frame) in buffer.frames_mut().enumerate() {
//         let random_walk_value = context.brown_walk_scale * fastrand::f32();
//         // To keep our signal within sane values, we enforce that the next
//         // value is within the  -1..1 range
//         value = {
//             let new_value_before_bounds = value + random_walk_value;
//             if new_value_before_bounds > 1.0 {
//                 let bounce = new_value_before_bounds - 1.0;
//                 let value_after_bounce = 1.0 - bounce;
//                 value_after_bounce
//             } else if new_value_before_bounds < -1.0 {
//                 let bounce = new_value_before_bounds + 1.0;
//                 let value_after_bounce = 1.0 + bounce;
//                 value_after_bounce
//             } else {
//                 new_value_before_bounds
//             }
//         };

//         frame[0] = value;
//         frame[1] = value;
//     }
//     *context.previous_brown_value = value;
// }
fn render_audio(audio: &mut Audio, buffer: &mut Buffer) {
    let block = audio.context.next_block();
    audio.fft = block[0]
        // let mut fft_input = block[0]
        .into_iter()
        .map(|value| Complex::new(*value, 0.))
        .collect::<Vec<_>>();
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(64);

    fft.process(&mut audio.fft);
    for (frame_index, frame) in buffer.frames_mut().enumerate() {
        for channel_index in 0..frame.len() {
            frame[channel_index] = block[channel_index][frame_index];
            let i_value = (frame[channel_index] * (i16::MAX as f32)) as i16 as u16;

            let mut buffer: [u8; 4] = [0, 0, 0, 0];
            buffer[0] = (i_value & 0x00ff) as u8;
            buffer[0 + 1] = ((i_value & 0xff00) >> 8) as u8;
            buffer[0 + 2] = (i_value & 0x00ff) as u8;
            buffer[0 + 3] = ((i_value & 0xff00) >> 8) as u8;
            // i2s.write(buffer, 1000);
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

    // let rotation_radians = deg_to_rad(settings.rotation);
    // draw.ellipse()
    //     .resolution(settings.resolution as f32)
    //     .xy(settings.position)
    //     .color(settings.color)
    //     .rotate(-rotation_radians)
    //     .radius(settings.scale);

    draw.to_frame(app, &frame).unwrap();
    model.egui.draw_to_frame(&frame).unwrap();
}
