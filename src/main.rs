use glicol_synth::{
    envelope::EnvPerc,
    filter::ResonantLowPassFilter,
    operator::{Add, Mul},
    oscillator::SinOsc,
    sequencer::{Sequencer, Speed},
    signal::{ConstSig, Impulse},
    AudioContext as GlicolAudioContext, AudioContextBuilder, GlicolPara, Message,
};
use nannou::prelude::*;
use nannou_audio::{self as audio, Buffer};
use nannou_egui::{
    self,
    egui::{self, style::Spacing},
    Egui,
};
use petgraph::adj::NodeIndices;
use sound::{brownish_noise::BrownishNoise, debug::DebugNode, NodeIndexSet, SoundParam};

mod sound;
mod state;

const SAMPLE_RATE: usize = 44_1000;
const BPM: usize = 128;
const FRAME_SIZE: usize = 64;
const CONFIG_VERSION: usize = 1;

type AudioContext = GlicolAudioContext<FRAME_SIZE>;

fn main() {
    nannou::app(model).exit(handle_exit).update(update).run();
}

fn update(_app: &App, model: &mut state::Model, update: Update) {
    let egui = &mut model.egui;

    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();
    egui::Window::new("Controls").show(&ctx, |ui| {
        if ui.add(egui::Button::new("Add Sine Wave")).clicked() {
            model.settings.ui_params.push(SoundParam::Sine {
                volume: 1.0,
                freq: 440.0,
            });
        }
    });

    for (index, (ui_params, glicol_indices)) in model
        .settings
        .ui_params
        .iter_mut()
        .zip(model.settings.glicol_indices.iter())
        .enumerate()
    {
        match (ui_params, glicol_indices) {
            (
                sound::SoundParam::Brownish {
                    knob_a,
                    volume,
                    low_pass_freq,
                },
                sound::NodeIndexSet::Brownish {
                    volume_index,
                    low_pass_index,
                    knob_a_index,
                },
            ) => {
                egui::Window::new(format!("Brownish Noise {index}")).show(&ctx, |ui| {
                    ui.label("Volume");
                    let volume_ui_element = ui.add(egui::Slider::new(volume, 0.0..=1.0));
                    if volume_ui_element.changed() {
                        let volume_index = volume_index.clone();
                        let set_volume_message = Message::SetToNumber(0, *volume);
                        model
                            .stream
                            .send(move |audio: &mut sound::Audio| {
                                audio.context.send_msg(volume_index, set_volume_message)
                            })
                            .unwrap();
                    }

                    ui.label("Low Pass Frequency");
                    let low_pass_ui_element =
                        ui.add(egui::Slider::new(low_pass_freq, 0.0..=5000.0));
                    if low_pass_ui_element.changed() {
                        let low_pass_index = low_pass_index.clone();
                        let set_low_pass_message = Message::SetToNumber(0, *low_pass_freq);
                        model
                            .stream
                            .send(move |audio: &mut sound::Audio| {
                                audio.context.send_msg(low_pass_index, set_low_pass_message)
                            })
                            .unwrap();
                    }

                    ui.label("Knob A");
                    let knob_a_ui_element = ui.add(egui::Slider::new(knob_a, 0.0..=1.0));
                    if knob_a_ui_element.changed() {
                        let knob_a_index = knob_a_index.clone();
                        let set_knob_a_message =
                            Message::SetParam(0, glicol_synth::GlicolPara::Number(*knob_a));
                        model
                            .stream
                            .send(move |audio: &mut sound::Audio| {
                                audio.context.send_msg(knob_a_index, set_knob_a_message)
                            })
                            .unwrap();
                    }
                });
            }
            (
                sound::SoundParam::Sine { volume, freq },
                sound::NodeIndexSet::Sine {
                    volume_index,
                    freq_index,
                },
            ) => {
                let volume_index = volume_index.clone();
                let set_volume_message = Message::SetToNumber(0, *volume);
                model
                    .stream
                    .send(move |audio: &mut sound::Audio| {
                        audio.context.send_msg(volume_index, set_volume_message)
                    })
                    .unwrap();

                let freq_index = freq_index.clone();
                let set_freq_message = Message::SetToNumber(0, *freq);
                model
                    .stream
                    .send(move |audio: &mut sound::Audio| {
                        audio.context.send_msg(freq_index, set_freq_message)
                    })
                    .unwrap();

                egui::Window::new(format!("Sine Wave {index}")).show(&ctx, |ui| {
                    ui.label("Volume");
                    ui.add(egui::Slider::new(volume, 0.0..=1.0));
                    ui.label("Frequency");
                    ui.add(egui::Slider::new(freq, 40.0..=440.0));
                });
            }
            (
                SoundParam::Kick {
                    volume,
                    bpm,
                    volume_attack,
                    volume_decay,
                    pitch_attack,
                    pitch_decay,
                },
                NodeIndexSet::Kick {
                    volume_envelope_index,
                    pitch_envelope_index,
                    beat_index,
                    volume_index,
                },
            ) => {
                let volume_index = volume_index.clone();
                let set_volume_message = Message::SetToNumber(0, *volume);
                model
                    .stream
                    .send(move |audio: &mut sound::Audio| {
                        audio.context.send_msg(volume_index, set_volume_message)
                    })
                    .unwrap();

                egui::Window::new(format!("Kick {index}")).show(&ctx, |ui| {
                    ui.label("Volume");
                    ui.add(egui::Slider::new(volume, 0.0..=1.0));

                    ui.label("BPM");
                    if ui.add(egui::Slider::new(bpm, 10.0..=200.0)).changed() {
                        let bpm_index = beat_index.clone();
                        let set_bpm_message = Message::SetToNumber(0, *bpm / 6.0);
                        model
                            .stream
                            .send(move |audio: &mut sound::Audio| {
                                audio.context.send_msg(bpm_index, set_bpm_message)
                            })
                            .unwrap();
                    }

                    ui.label("Volume Attack");
                    if ui
                        .add(egui::Slider::new(volume_attack, 0.0..=1.0))
                        .changed()
                    {
                        let volume_envelope_index = volume_envelope_index.clone();
                        let set_volume_attack_message = Message::SetToNumber(0, *volume_attack);
                        model
                            .stream
                            .send(move |audio: &mut sound::Audio| {
                                audio
                                    .context
                                    .send_msg(volume_envelope_index, set_volume_attack_message)
                            })
                            .unwrap();
                    }

                    ui.label("Volume Decay");
                    if ui.add(egui::Slider::new(volume_decay, 0.0..=1.0)).changed() {
                        let volume_envelope_index = volume_envelope_index.clone();
                        let set_volume_decay_message = Message::SetToNumber(1, *volume_decay);
                        model
                            .stream
                            .send(move |audio: &mut sound::Audio| {
                                audio
                                    .context
                                    .send_msg(volume_envelope_index, set_volume_decay_message)
                            })
                            .unwrap();
                    }

                    ui.label("Pitch Attack");
                    if ui.add(egui::Slider::new(pitch_attack, 0.0..=1.0)).changed() {
                        let pitch_envelope_index = pitch_envelope_index.clone();
                        let set_pitch_attack_message = Message::SetToNumber(0, *pitch_attack);
                        model
                            .stream
                            .send(move |audio: &mut sound::Audio| {
                                audio
                                    .context
                                    .send_msg(pitch_envelope_index, set_pitch_attack_message)
                            })
                            .unwrap();
                    }

                    ui.label("Pitch Decay");
                    if ui.add(egui::Slider::new(pitch_decay, 0.0..=1.0)).changed() {
                        let pitch_envelope_index = pitch_envelope_index.clone();
                        let set_pitch_decay_message = Message::SetToNumber(1, *pitch_decay);
                        model
                            .stream
                            .send(move |audio: &mut sound::Audio| {
                                audio
                                    .context
                                    .send_msg(pitch_envelope_index, set_pitch_decay_message)
                            })
                            .unwrap();
                    }
                });
            }
            (_, _) => todo!(),
        }
    }
}

fn model(app: &App) -> state::Model {
    // Create a window to receive key pressed events.
    let window_id = app
        .new_window()
        .title("Useful Noise 🔊")
        .size(600, 600)
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

    let mut settings = get_settings();
    let mut glicol_indices = vec![];

    // Each of our settings will need to have glicol nodes created
    for ui_param in &settings.ui_params {
        // We use this "match" statement to create the node configuration
        // based on which type of `SourceParam` we're setting up
        let node_index_set = match ui_param {
            // Handle "Brownish" noise
            sound::SoundParam::Brownish {
                knob_a,
                volume,
                low_pass_freq,
            } => brownish_node_set_from_sound_param(&mut context, knob_a, low_pass_freq, volume),

            // Handle sine wave config
            sound::SoundParam::Sine { volume, freq } => {
                sine_node_set_from_sound_param(&mut context, freq, volume)
            }
            sound::SoundParam::Kick {
                volume,
                bpm,
                volume_attack,
                volume_decay,
                pitch_attack,
                pitch_decay,
            } => kick_node_set_from_sound_param(
                &mut context,
                volume_attack,
                volume_decay,
                pitch_attack,
                pitch_decay,
                bpm,
                volume,
            ),
        };
        glicol_indices.push(node_index_set);
    }

    settings.glicol_indices = glicol_indices;
    let model = sound::Audio { context };

    let stream = audio_host
        .new_output_stream(model)
        .render(render_audio)
        .build()
        .unwrap();

    stream.play().unwrap();

    state::Model {
        egui,
        stream,
        settings,
    }
}

fn kick_node_set_from_sound_param(
    context: &mut GlicolAudioContext<64>,
    volume_attack: &f32,
    volume_decay: &f32,
    pitch_attack: &f32,
    pitch_decay: &f32,
    bpm: &f32,
    volume: &f32,
) -> NodeIndexSet {
    let volume_envelope_index = context.add_mono_node(
        EnvPerc::new()
            .sr(SAMPLE_RATE)
            .attack(*volume_attack)
            .decay(*volume_decay),
    );

    let pitch_envelope_index = context.add_mono_node(
        EnvPerc::new()
            .sr(SAMPLE_RATE)
            .attack(*pitch_attack)
            .decay(*pitch_decay),
    );

    let beat_index = context.add_mono_node(Impulse::new().freq(*bpm / 60.0).sr(SAMPLE_RATE));

    let kick_pitch = context.add_mono_node(Mul::new(500.0));
    let kick_pitch_baseline = context.add_mono_node(Add::new(400.0));
    let kick_synth = context.add_mono_node(SinOsc::new().sr(SAMPLE_RATE));

    let pre_master = context.add_mono_node(Mul::new(1.0));

    context.chain(vec![beat_index, volume_envelope_index, pre_master]);

    let volume_index = context.add_mono_node(Mul::new(*volume));

    context.chain(vec![
        beat_index,
        pitch_envelope_index,
        kick_pitch,
        kick_pitch_baseline,
        kick_synth,
        pre_master,
        volume_index,
        context.destination,
    ]);

    NodeIndexSet::Kick {
        volume_envelope_index,
        pitch_envelope_index,
        beat_index,
        volume_index,
    }
}

fn sine_node_set_from_sound_param(
    context: &mut GlicolAudioContext<64>,
    freq: &f32,
    volume: &f32,
) -> NodeIndexSet {
    let sine_id = context.add_mono_node(SinOsc::new().freq(*freq));
    let sine_volume = context.add_mono_node(Mul::new(*volume));
    context.chain(vec![sine_id, sine_volume, context.destination]);
    sound::NodeIndexSet::Sine {
        volume_index: sine_volume,
        freq_index: sine_id,
    }
}

fn brownish_node_set_from_sound_param(
    context: &mut GlicolAudioContext<64>,
    knob_a: &f32,
    low_pass_freq: &f32,
    volume: &f32,
) -> NodeIndexSet {
    // Create glicol node indices for each of the node types that
    // will make up the settings. For the Brownish noise, this is
    // the BrownishNoise itself, and two additional nodes, one
    // for the low-pass cut-off and one for the volume control.
    // These are then "chained" in a sequence, effectively passing
    // our noise signal through a series of filters.
    let knob_a_index = context.add_mono_node(BrownishNoise::new_with_scale(*knob_a));

    let low_pass_index = context.add_mono_node(ResonantLowPassFilter::new().cutoff(*low_pass_freq));
    let volume_index = context.add_mono_node(Mul::new(*volume));

    context.chain(vec![
        knob_a_index,
        volume_index,
        low_pass_index,
        context.destination,
    ]);

    // Return the newly created indices so they can be stored in
    // the application state and used for calling `send_msg` to the
    // correct glicol node
    sound::NodeIndexSet::Brownish {
        volume_index,
        low_pass_index,
        knob_a_index,
    }
}

/// Load settings from a file, or use the default settings if no settings are
/// found
fn get_settings() -> state::Settings {
    if let Ok(toml) = std::fs::read_to_string("useful-noise.toml") {
        let settings = toml::from_str::<state::Settings>(&toml);
        if let Ok(settings) = settings {
            if settings.config_version < CONFIG_VERSION {
                state::Settings::default()
            } else {
                settings
            }
        } else {
            state::Settings::default()
        }
    } else {
        state::Settings::default()
    }
}

/// Copies the audio data from the glicol buffer into the nannou audio buffer
fn render_audio(audio: &mut sound::Audio, buffer: &mut Buffer) {
    let block = audio.context.next_block();
    for (frame_index, frame) in buffer.frames_mut().enumerate() {
        for channel_index in 0..frame.len() {
            frame[channel_index] = block[channel_index][frame_index];
        }
    }
}

fn key_pressed(_app: &App, _model: &mut state::Model, key: Key) {
    match key {
        Key::A => {}
        _ => {}
    }
}

fn raw_window_event(
    _app: &App,
    model: &mut state::Model,
    event: &nannou::winit::event::WindowEvent,
) {
    // Let egui handle things like keyboard and mouse input.
    model.egui.handle_raw_event(event);
}

fn view(app: &App, model: &state::Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);

    draw.to_frame(app, &frame).unwrap();
    model.egui.draw_to_frame(&frame).unwrap();
}

// Called when the application exits
fn handle_exit(_app: &App, model: state::Model) {
    // Convert our `Settings` into a string (which is possible thanks to the
    // `Serialize` and `Deserialize` traits), then write it to a file
    let settings = toml::to_string(&model.settings).unwrap();
    std::fs::write("useful-noise.toml", settings).unwrap();
}
