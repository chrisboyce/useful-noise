use egui_plot::{Line, Plot, PlotPoints};
use nannou::prelude::*;
use nannou_audio as audio;
use nannou_audio::Buffer;
use nannou_egui::{self, egui, Egui};
use petgraph::graph::NodeIndex;
use std::f64::consts::PI;

const SAMPLE_RATE: usize = 44_1000;
const BIT_DEPTH: usize = 64;
use glicol_synth::{
    filter::ResonantLowPassFilter,
    operator::{Add, Mul},
    oscillator::{SawOsc, SinOsc},
    signal::{ConstSig, Noise},
    AudioContext as GlicolAudioContext, AudioContextBuilder, Message, Sum,
};
type AudioContext = GlicolAudioContext<BIT_DEPTH>;
fn main() {
    // let b = context.next_block();
    nannou::app(model).update(update).run();
}

struct Model {
    egui: Egui,
    settings: Settings,
    stream: audio::Stream<Audio>,
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
    tone: NodeIndex,
    context: AudioContext,
}
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

    // let tone = context.add_mono_node(SinOsc::new().freq(440.0));
    // let volume = context.add_mono_node(Mul::new(0.3));
    // let node_b = context.add_stereo_node(Mul::new(0.1));
    // context.chain(vec![node_a, noise_node]);
    // context.connect(noise_node, filter_node);
    // context.connect(filter_node, node_b);
    // context.connect(noise_node, node_b);
    // context.connect(node_a, node_b);
    // context.connect(noise_node, context.destination);
    // context.connect(tone, volume);
    // context.connect(noise_node, volume);
    // context.connect(volume, context.destination);
    // let node_a = context.add_mono_node(SawOsc::new().freq(440.));
    // let node_b = context.add_mono_node(ResonantLowPassFilter::new().cutoff(1000.0));
    // let node_c = context.add_mono_node(ConstSig::new(10.0));
    // context.chain(vec![node_a, node_b, context.destination]);
    // context.connect(node_c, node_b);
    // let filter = glicol_synth::filter::ResonantLowPassFilter::new().cutoff(5000.0);
    // let filter_node = context.add_mono_node(filter);
    // let noise = Noise::new(0);
    // let noise_node = context.add_mono_node(noise);
    // let t1 = context.add_stereo_node(SinOsc::new().freq(40.0));
    // let m1 = context.add_stereo_node(Mul::new(.));
    // let t2 = context.add_stereo_node(SinOsc::new().freq(80.0));
    // let m2 = context.add_stereo_node(Mul::new(20.));
    // let add = context.add_stereo_node(Add::new(600.0));
    // context.connect(t1, m1);
    // context.connect(t2, m2);
    // context.connect(m1, add);
    // context.connect(add, t2);
    let sin1 = context.add_stereo_node(SinOsc::new().freq(440.0));
    let sin2 = context.add_stereo_node(SinOsc::new().freq(80.0));
    let mix = context.add_stereo_node(Sum {});
    // let mul2 = context.add_stereo_node(Mul::new(0.1));
    // let add = context.add_stereo_node(Add::new(500.));
    // let mul1 = context.add_stereo_node(Mul::new(0.1));
    // let mul2 = context.add_stereo_node(Mul::new(300.));
    // let add2 = context.add_stereo_node(Add::new(600.));
    // context.connect(sin1, mul1);
    // context.connect(noise_node, mul2);
    // context.connect(mul2, add2);
    // context.connect(add2, sin1);
    // context.connect(t1, t2);
    // context.connect(mul1, context.destination);
    context.chain(vec![sin1, context.destination]);
    context.chain(vec![sin2, context.destination]);
    let model = Audio {
        context,
        tone: sin1,
    };

    let stream = audio_host
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
            model.stream.send(|audio: &mut Audio| {
                audio
                    .context
                    .send_msg(audio.tone, Message::SetToNumber(0, 300.))
            });
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
