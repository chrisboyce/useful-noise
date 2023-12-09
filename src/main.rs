use std::fs::read_to_string;
use std::path::Path;

use glicol_synth::{oscillator::SinOsc, Node};
use nannou::prelude::*;
use nannou_audio as audio;
use nannou_audio::Buffer;
use nannou_egui::{self, egui, Egui};
use petgraph::graph::NodeIndex;
use serde::Deserialize;
use serde::Serialize;

const SAMPLE_RATE: usize = 44_1000;
const FRAME_SIZE: usize = 64;
use glicol_synth::{
    filter::ResonantLowPassFilter, operator::Mul, AudioContext as GlicolAudioContext,
    AudioContextBuilder, Message,
};
type AudioContext = GlicolAudioContext<FRAME_SIZE>;
fn main() {
    nannou::app(model).exit(handle_exit).update(update).run();
}

#[derive(Clone, Debug)]
enum NodeIndexSet {
    Brownish {
        volume_index: NodeIndex,
        low_pass_index: NodeIndex,
        knob_a_index: NodeIndex,
    },
}

#[derive(Clone, Serialize, Deserialize, Debug)]
enum SourceParam {
    Brownish {
        knob_a: f32,
        volume: f32,
        low_pass_freq: f32,
    },
}

#[derive(Serialize, Deserialize, Debug)]
struct SourceConfig {
    #[serde(skip)]
    node_index_set: Option<NodeIndexSet>,
    params: SourceParam,
}

impl Default for NewSettings {
    fn default() -> Self {
        Self {
            glicol_indices: vec![],
            ui_params: vec![SourceParam::Brownish {
                knob_a: 0.1,
                volume: 0.5,
                low_pass_freq: 500.0,
            }],
        }
    }
}

struct Model {
    egui: Egui,
    new_settings: NewSettings,
    settings: Settings,
    stream: audio::Stream<Audio>,
}

const MAX_NODES: usize = 1;
#[derive(Serialize, Deserialize)]
struct NewSettings {
    #[serde(skip)]
    glicol_indices: Vec<NodeIndexSet>,
    ui_params: Vec<SourceParam>,
    // source_configs: Vec<SourceConfig>, // sources: Vec<SourceParam>,
    // #[serde(skip)]
    // source_indices: Vec<NodeIndexSet>,
}

#[derive(Serialize, Deserialize)]
struct Settings {
    brownish_noise_knob_a: f32,
    brownish_noise_volume: f32,
    low_pass_freq: f32,
    param: SourceParam,
}

#[derive(Serialize, Deserialize)]
struct BrownishNoise {
    brown_walk_scale: f32,
    previous_brown_value: f32,
}

impl BrownishNoise {
    fn new_with_scale(scale: f32) -> Self {
        Self {
            brown_walk_scale: scale,
            previous_brown_value: 0.,
        }
    }
    fn new() -> Self {
        Self::new_with_scale(0.1)
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
    context: AudioContext,
}

fn update(_app: &App, model: &mut Model, update: Update) {
    let egui = &mut model.egui;

    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();

    let params = model.settings.param.clone();

    if let SourceParam::Brownish {
        knob_a,
        volume,
        low_pass_freq,
    } = &mut model.settings.param
    {
        egui::Window::new("Brownish Noise").show(&ctx, |ui| {
            ui.label("Volume");
            ui.add(egui::Slider::new(volume, 0.0..=1.0));
            ui.label("Low Pass Frequency");
            ui.add(egui::Slider::new(low_pass_freq, 0.0..=1.0));
            ui.label("Knob A");
            ui.add(egui::Slider::new(knob_a, 0.0..=1.0));
        });
    }

    // for index in 0..model.new_settings.ui_params.len() {
    //     let ui_param = model.new_settings.ui_params.get(index);
    //     let glicol_indices = model.new_settings.glicol_indices.get(index);

    //     match (glicol_indices, &ui_param) {
    //         (
    //             Some(Some(NodeIndexSet::Brownish {
    //                 volume_index,
    //                 low_pass_index,
    //                 knob_a_index,
    //             })),
    //             Some(Some(SourceParam::Brownish {
    //                 knob_a,
    //                 mut volume,
    //                 low_pass_freq,
    //             })),
    //         ) => {
    //             let title = "Brownish Noise".to_string();
    //             // let knob_a_index = knob_a_index.clone();
    //             // // Create a message to send to this node to update the value
    //             // let set_knob_a_message =
    //             //     Message::SetParam(0, glicol_synth::GlicolPara::Number(*knob_a));
    //             // model
    //             //     .stream
    //             //     .send(move |audio: &mut Audio| {
    //             //         // println!("{:?}", &set_knob_a_message);
    //             //         audio
    //             //             .context
    //             //             // .send_msg(NodeIndex::new(1), set_knob_a_message)
    //             //             .send_msg(knob_a_index, set_knob_a_message)
    //             //     })
    //             //     .unwrap();
    //             egui::Window::new(title).show(&ctx, |ui| {
    //                 println!("{:p}", &volume);
    //                 ui.label("Volume");
    //                 ui.add(egui::Slider::new(
    //                     &mut volume,
    //                     // &mut model.settings.brownish_noise_volume,
    //                     0.0..=1.0,
    //                 ));
    //             });
    //         }

    //         (_, _) => todo!(),
    //     }

    // egui::Window::new(title).show(&ctx, |ui| {
    //     // Draw the UI controls for the given type of node
    //     match ui_param {
    //         SourceParam::Brownish {
    //             knob_a,
    //             mut volume,
    //             low_pass_freq,
    //         } => {
    //             let mut f = 0.0;
    //             ui.label("Volume");
    //             ui.add(egui::Slider::new(
    //                 &mut volume,
    //                 // &mut model.settings.brownish_noise_volume,
    //                 0.0..=1.0,
    //             ));
    //             // ui.label("Low Pass");
    //             // ui.add(egui::Slider::new(&mut low_pass_freq, 0.0..=10000.0));
    //             // ui.label("Knob A");
    //             // ui.add(egui::Slider::new(&mut knob_a, 0.0..=1.0));
    //         }
    //     }
    // });

    // let params = source_config.params.clone();
    // let node_indices = source_config.node_index_set.clone();
    // match (node_indices, params) {
    //     (
    //         Some(NodeIndexSet::Brownish {
    //             volume_index: volume_index,
    //             low_pass_index: low_pass_index,
    //             knob_a_index: knob_a_index,
    //         }),
    //         SourceParam::Brownish {
    //             knob_a,
    //             volume,
    //             low_pass_freq,
    //         },
    //     ) => {
    //         // Create a message to send to this node to update the value
    //         let set_knob_a_message =
    //             Message::SetParam(0, glicol_synth::GlicolPara::Number(knob_a));
    //         model
    //             .stream
    //             .send(move |audio: &mut Audio| {
    //                 println!("{knob_a}");
    //                 audio.context.send_msg(knob_a_index, set_knob_a_message)
    //             })
    //             .unwrap();
    //     }
    //     _ => todo!(),
    // }

    // let set_volume_message = Message::SetToNumber(0, settings.brownish_noise_volume);
    // model
    //     .stream
    //     .send(move |audio: &mut Audio| {
    //         audio
    //             .context
    //             .send_msg(audio.brownish_node_volume_index, set_volume_message)
    //     })
    //     .unwrap();

    // let set_low_pass_message = Message::SetToNumber(0, settings.low_pass_freq);
    // model
    //     .stream
    //     .send(move |audio: &mut Audio| {
    //         audio
    //             .context
    //             .send_msg(audio.brownish_low_pass_index, set_low_pass_message)
    //     })
    //     .unwrap();
    // }
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

    let mut settings = NewSettings::default();
    let mut glicol_indices = vec![];

    for ui_param in &settings.ui_params {
        let node_index_set = match ui_param {
            SourceParam::Brownish {
                knob_a,
                volume,
                low_pass_freq,
            } => {
                let noise = BrownishNoise::new_with_scale(*knob_a);
                let noise_id = context.add_stereo_node(noise);
                let noise_low_pass =
                    context.add_stereo_node(ResonantLowPassFilter::new().cutoff(*low_pass_freq));
                let noise_volume = context.add_stereo_node(Mul::new(*volume));
                context.chain(vec![
                    noise_id,
                    noise_low_pass,
                    noise_volume,
                    context.destination,
                ]);
                NodeIndexSet::Brownish {
                    volume_index: noise_volume,
                    low_pass_index: noise_low_pass,
                    knob_a_index: noise_id,
                }
            }
        };
        glicol_indices.push(node_index_set);
    }

    settings.glicol_indices = glicol_indices;
    let model = Audio { context };

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
            param: SourceParam::Brownish {
                knob_a: 0.0,
                volume: 0.0,
                low_pass_freq: 0.0,
            },
        },
        new_settings: settings,
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
fn handle_exit(app: &App, model: Model) {
    let settings = toml::to_string(&model.settings).unwrap();
    dbg!(settings);
}
