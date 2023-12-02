use glicol_synth::{oscillator::SinOsc, Node};
use nannou::prelude::*;
use nannou_audio as audio;
use nannou_audio::Buffer;
use nannou_egui::{self, egui, Egui};
use petgraph::graph::NodeIndex;

const SAMPLE_RATE: usize = 44_1000;
const FRAME_SIZE: usize = 64;
use glicol_synth::{
    filter::ResonantLowPassFilter, operator::Mul, AudioContext as GlicolAudioContext,
    AudioContextBuilder, Message,
};
type AudioContext = GlicolAudioContext<FRAME_SIZE>;
fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    egui: Egui,
    settings: Settings,
    stream: audio::Stream<Audio>,
}

struct Settings {
    brownish_noise_knob_a: f32,
    brownish_noise_volume: f32,
    low_pass_freq: f32,
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
            // To keep our signal within sane values, we enforce that the next
            // value is within the  -1..1 range
            let random_walk_value = self.brown_walk_scale * (fastrand::f32() * 2. - 1.0);
            value = {
                let new_value_before_bounds = value + random_walk_value;
                if new_value_before_bounds > 1.0 {
                    1.0
                } else if new_value_before_bounds < -1.0 {
                    -1.0
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

struct Audio {
    brownish_node_index: NodeIndex,
    brownish_node_volume_index: NodeIndex,
    brownish_low_pass_index: NodeIndex,
    context: AudioContext,
}

fn update(_app: &App, model: &mut Model, update: Update) {
    let egui = &mut model.egui;
    let settings = &mut model.settings;

    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();

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

        ui.label("Knob A");
        ui.add(egui::Slider::new(
            &mut settings.brownish_noise_knob_a,
            0.0..=1.0,
        ));
    });

    let set_knob_a_message = Message::SetParam(
        0,
        glicol_synth::GlicolPara::Number(settings.brownish_noise_knob_a),
    );
    model
        .stream
        .send(move |audio: &mut Audio| {
            audio
                .context
                .send_msg(audio.brownish_node_index, set_knob_a_message)
        })
        .unwrap();

    let set_volume_message = Message::SetToNumber(0, settings.brownish_noise_volume);
    model
        .stream
        .send(move |audio: &mut Audio| {
            audio
                .context
                .send_msg(audio.brownish_node_volume_index, set_volume_message)
        })
        .unwrap();

    let set_low_pass_message = Message::SetToNumber(0, settings.low_pass_freq);
    model
        .stream
        .send(move |audio: &mut Audio| {
            audio
                .context
                .send_msg(audio.brownish_low_pass_index, set_low_pass_message)
        })
        .unwrap();
}

fn model(app: &App) -> Model {
    // Create a window to receive key pressed events.
    let window_id = app
        .new_window()
        .title("Useful Noise 🔊")
        .size(240, 200)
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
    let noise = BrownishNoise::new();
    let noise_id = context.add_stereo_node(noise);
    let noise_low_pass = context.add_stereo_node(ResonantLowPassFilter::new().cutoff(1000.0));
    let noise_volume = context.add_stereo_node(Mul::new(1.));
    context.chain(vec![
        noise_id,
        noise_low_pass,
        noise_volume,
        context.destination,
    ]);

    let model = Audio {
        context,
        brownish_node_index: noise_id,
        brownish_node_volume_index: noise_volume,
        brownish_low_pass_index: noise_low_pass,
    };

    let mut brown = 0.;
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
            brownish_noise_knob_a: 0.07,
            brownish_noise_volume: 0.65,
            low_pass_freq: 10000.0,
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
        Key::A => {}
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
