use nannou::prelude::*;
use nannou_audio as audio;
use nannou_audio::Buffer;
use nannou_egui::{self, egui, Egui};
use std::f64::consts::PI;

const SAMPLE_RATE: usize = 44_1000;
const BIT_DEPTH: usize = 64;
use glicol_synth::{
    operator::Mul, oscillator::SinOsc, AudioContext as GlicolAudioContext, AudioContextBuilder,
    Message,
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
    glicol: AudioContext,
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

    let node_a = context.add_mono_node(SinOsc::new().freq(440.0));
    let noise = glicol_synth::signal::Noise::new(0);
    let noise_node = context.add_mono_node(noise);
    let node_b = context.add_stereo_node(Mul::new(0.1));
    // context.connect(noise_node, node_b);
    context.connect(node_a, node_b);
    context.connect(node_b, context.destination);
    let model = Audio { glicol: context };

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
    // println!("{}", buffer.frames_mut().len());
    let block = audio.glicol.next_block();
    // dbg!(&block);
    for (frame_index, frame) in buffer.frames_mut().enumerate() {
        for channel_index in 0..frame.len() {
            frame[channel_index] = block[channel_index][frame_index];
            // let mut channel = &frame[channel_index];
            // *channel = block[channel_index][frame_index]
            // *channel = block[]
        }
    }
}
// A function that renders the given `Audio` to the given `Buffer`.
// In this case we play a simple sine wave at the audio's current frequency in `hz`.
// fn audio(audio: &mut Audio, buffer: &mut Buffer) {
//     let sample_rate = buffer.sample_rate() as f64;
//     let volume = 0.5;
//     for frame in buffer.frames_mut() {
//         let sine_amp = (2.0 * PI * audio.phase).sin() as f32;
//         audio.phase += audio.hz / sample_rate;
//         audio.phase %= sample_rate;
//         for channel in frame {
//             *channel = sine_amp * volume;
//         }
//     }
// }

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    match key {
        // Pause or unpause the audio when Space is pressed.
        Key::Space => {
            if model.stream.is_playing() {
                model.stream.pause().unwrap();
            } else {
                model.stream.play().unwrap();
            }
        }
        // Raise the frequency when the up key is pressed.
        Key::Up => {
            model
                .stream
                .send(|audio| {
                    // audio.hz += 10.0;
                })
                .unwrap();
        }
        // Lower the frequency when the down key is pressed.
        Key::Down => {
            model
                .stream
                .send(|audio| {
                    // audio.hz -= 10.0;
                })
                .unwrap();
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
