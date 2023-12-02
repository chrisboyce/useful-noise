use egui_plot::{Line, Plot, PlotPoints};
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
    stream: audio::Stream<f32>,
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

    egui::Window::new("Settings").show(&ctx, |ui| {
        // Resolution slider
        ui.label("Resolution:");
        ui.add(egui::Slider::new(&mut settings.resolution, 1..=40));

        ui.label("Frequency:");
        ui.add(egui::Slider::new(&mut settings.frequency, 40.0..=40000.));

        // Scale slider
        ui.label("Scale:");
        ui.add(egui::Slider::new(&mut settings.scale, 0.0..=1000.0));

        // Rotation slider
        ui.label("Rotation:");
        ui.add(egui::Slider::new(&mut settings.rotation, 0.0..=360.0));

        // Random color button
        let clicked = ui.button("Random color").clicked();

        if clicked {
            settings.color = rgb(random(), random(), random());
        }
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
    let sin2 = context.add_stereo_node(SinOsc::new().freq(80.0));
    let mix = context.add_stereo_node(Sum {});
    context.chain(vec![sin1, context.destination]);
    context.chain(vec![sin2, context.destination]);
    let model = Audio {
        context,
        fft: Vec::with_capacity(64),
        tone: sin1,
    };

    let mut brown = 0.;
    let stream = audio_host
        .new_output_stream(brown)
        .render(render_brownian_noise)
        // .new_output_stream(model)
        // .render(render_audio)
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

fn render_brownian_noise(previous_value: &mut f32, buffer: &mut Buffer) {
    let mut value = *previous_value;
    for (frame_index, frame) in buffer.frames_mut().enumerate() {
        // dbg!(frame);
        // for channel_index in 0..frame.len() {
        let random_walk_value = 0.1 * fastrand::f32();
        // To keep our signal within sane values, we enforce that the next
        // value is within the  -1..1 range
        value = {
            let new_value_before_bounds = value + random_walk_value;
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

        frame[0] = value;
        frame[1] = value;
    }
    *previous_value = value;
}
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

    let rotation_radians = deg_to_rad(settings.rotation);
    draw.ellipse()
        .resolution(settings.resolution as f32)
        .xy(settings.position)
        .color(settings.color)
        .rotate(-rotation_radians)
        .radius(settings.scale);

    draw.to_frame(app, &frame).unwrap();
    model.egui.draw_to_frame(&frame).unwrap();
}
