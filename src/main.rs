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

type AudioContext = GlicolAudioContext<FRAME_SIZE>;

fn main() {
    nannou::app(model).exit(handle_exit).update(update).run();
}

fn update(_app: &App, model: &mut state::Model, update: Update) {
    let egui = &mut model.egui;

    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();

    for (ui_params, glicol_indices) in model
        .settings
        .ui_params
        .iter_mut()
        .zip(model.settings.glicol_indices.iter())
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
                let volume_index = volume_index.clone();
                let set_volume_message = Message::SetToNumber(0, *volume);
                model
                    .stream
                    .send(move |audio: &mut sound::Audio| {
                        audio.context.send_msg(volume_index, set_volume_message)
                    })
                    .unwrap();

                let low_pass_index = low_pass_index.clone();
                let set_low_pass_message = Message::SetToNumber(0, *low_pass_freq);
                model
                    .stream
                    .send(move |audio: &mut sound::Audio| {
                        audio.context.send_msg(low_pass_index, set_low_pass_message)
                    })
                    .unwrap();

                let knob_a_index = knob_a_index.clone();
                let set_knob_a_message =
                    Message::SetParam(0, glicol_synth::GlicolPara::Number(*knob_a));
                model
                    .stream
                    .send(move |audio: &mut sound::Audio| {
                        audio.context.send_msg(knob_a_index, set_knob_a_message)
                    })
                    .unwrap();

                egui::Window::new("Brownish Noise").show(&ctx, |ui| {
                    ui.label("Volume");
                    ui.add(egui::Slider::new(volume, 0.0..=1.0));
                    ui.label("Low Pass Frequency");
                    ui.add(egui::Slider::new(low_pass_freq, 0.0..=5000.0));
                    ui.label("Knob A");
                    ui.add(egui::Slider::new(knob_a, 0.0..=1.0));
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

                egui::Window::new("Sine Wave").show(&ctx, |ui| {
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
                // let bpm_index = beat_index.clone();
                // let set_bpm_message = Message::SetToNumber(0, *bpm / 60.0);
                // model
                //     .stream
                //     .send(move |audio: &mut sound::Audio| {
                //         audio.context.send_msg(bpm_index, set_bpm_message)
                //     })
                //     .unwrap();
                let am_env_index = volume_envelope_index.clone();
                let set_amplitude_attack_message = Message::SetToNumber(0, *volume_attack);
                model
                    .stream
                    .send(move |audio: &mut sound::Audio| {
                        audio
                            .context
                            .send_msg(am_env_index, set_amplitude_attack_message)
                    })
                    .unwrap();
                let set_amplitude_decay_message = Message::SetToNumber(1, *volume_decay);
                model
                    .stream
                    .send(move |audio: &mut sound::Audio| {
                        audio
                            .context
                            .send_msg(am_env_index, set_amplitude_decay_message)
                    })
                    .unwrap();
                let fm_env_index = pitch_envelope_index.clone();
                let set_pitch_attack_message = Message::SetToNumber(0, *pitch_attack);
                model
                    .stream
                    .send(move |audio: &mut sound::Audio| {
                        audio
                            .context
                            .send_msg(fm_env_index, set_pitch_attack_message)
                    })
                    .unwrap();
                let set_pitch_decay_message = Message::SetToNumber(1, *pitch_decay);
                model
                    .stream
                    .send(move |audio: &mut sound::Audio| {
                        audio
                            .context
                            .send_msg(fm_env_index, set_pitch_decay_message)
                    })
                    .unwrap();

                egui::Window::new("Kick").show(&ctx, |ui| {
                    ui.label("Volume");
                    ui.add(egui::Slider::new(volume, 0.0..=1.0));
                    ui.label("BPM");
                    ui.add(egui::Slider::new(bpm, 10.0..=200.0));
                    ui.label("Volume Attack");
                    ui.add(egui::Slider::new(volume_attack, 0.0..=1.0));
                    ui.label("Volume Decay");
                    ui.add(egui::Slider::new(volume_decay, 0.0..=1.0));
                    ui.label("Pitch Attack");
                    ui.add(egui::Slider::new(pitch_attack, 0.0..=1.0));
                    ui.label("Pitch Decay");
                    ui.add(egui::Slider::new(pitch_decay, 0.0..=1.0));

                    // ui.add(egui::Slider::new(freq, 40.0..=440.0));
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
        .size(400, 400)
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

    //     out: sin ~pitch >> mul ~envb >> mul 0.9
    // ~envb: ~triggerb >> envperc 0.01 0.4;
    // ~env_pitch: ~triggerb >> envperc 0.01 0.1;
    // ~pitch: ~env_pitch >> mul 50 >> add 60;
    // ~triggerb: speed 4.0 >> seq 60
    // let trigger_speed = context.add_mono_node(Speed::new(4.0));
    // let trigger_seq = context.add_mono_node(
    //     Sequencer::new(vec![
    //         (60.0, GlicolPara::Number(2.0)),
    //         (20.0, GlicolPara::Number(1.0)),
    //     ])
    //     .sr(SAMPLE_RATE)
    //     .bpm(BPM as f32),
    // );

    let env_perc = context.add_mono_node(EnvPerc::new().sr(SAMPLE_RATE).attack(0.01).decay(0.4));
    let ep2 = context.add_mono_node(EnvPerc::new().attack(0.01).decay(0.1).sr(SAMPLE_RATE));
    let p_m = context.add_mono_node(Mul::new(50.0));
    let p_a = context.add_mono_node(Add::new(60.0));
    let s = context.add_mono_node(SinOsc::new().freq(440.0));
    let m2 = context.add_mono_node(Mul::new(1.0));
    let m3 = context.add_mono_node(Mul::new(0.9));
    // let c = context.add_mono_node(ConstSig::new(220.0));
    let i = context.add_mono_node(Impulse::new().freq(1.0).sr(SAMPLE_RATE));

    // // Connect the Impulse to the percussion envelope
    // // -> envb
    // context.chain(vec![i, env_perc]);

    // // -> env_pitch
    // context.chain(vec![i, ep2]);

    // // -> pitch
    // context.chain(vec![ep2, p_m, p_a]);

    // context.chain(vec![env_perc, m2]);
    // context.chain(vec![ep2, s, m2, m3, context.destination]);

    // // triggerb
    // context.chain(vec![trigger_speed, trigger_seq]);

    // // envb
    // context.chain(vec![trigger_seq, env_perc]);

    // // env_pitch
    // context.chain(vec![trigger_seq, ep2]);

    // // pitch
    // context.chain(vec![ep2, p_m, p_a]);

    // context.chain(vec![env_perc, m2]);

    // out
    // context.chain(vec![p_a, s, m2, m3, context.destination]);
    // context.chain(vec![trigger_seq, m2]);
    // context.chain(vec![trigger_speed, trigger_seq, m2]);

    // context.chain(vec![i, env_perc, m2]);
    // context.chain(vec![s, m2, context.destination]);

    // let a = context.add_mono_node(SinOsc::new().freq(440.));
    // let b = context.add_mono_node(Mul::new(1.0));
    // let c = context.add_mono_node(SinOsc::new().freq(1.2));
    // let d = context.add_mono_node(Mul::new(0.3));
    // let e = context.add_mono_node(Add::new(0.5));
    // context.chain(vec![a, b]);
    // context.chain(vec![c, d, e]);
    // context.chain(vec![e, b]);
    // context.chain(vec![b, context.destination]);

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
            } => {
                // Create glicol node indices for each of the node types that
                // will make up the settings. For the Brownish noise, this is
                // the BrownishNoise itself, and two additional nodes, one
                // for the low-pass cut-off and one for the volume control.
                // These are then "chained" in a sequence, effectively passing
                // our noise signal through a series of filters.
                let knob_a_index = context.add_mono_node(BrownishNoise::new_with_scale(*knob_a));

                let low_pass_index =
                    context.add_mono_node(ResonantLowPassFilter::new().cutoff(*low_pass_freq));
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

            // Handle sine wave config
            sound::SoundParam::Sine { volume, freq } => {
                // ~mod: sin 1.2 >> mul 0.3 >> add 0.5
                // let module = context.add_mono_node(SinOsc::new().freq(1.0));
                // let f = context.add_mono_node(Mul::new(0.3));

                let sine_id = context.add_mono_node(SinOsc::new().freq(440.0));
                let sine_volume = context.add_mono_node(Mul::new(*volume));
                context.chain(vec![sine_id, sine_volume, context.destination]);
                sound::NodeIndexSet::Sine {
                    volume_index: sine_volume,
                    freq_index: sine_id,
                }
            }
            sound::SoundParam::Kick {
                volume,
                bpm,
                volume_attack: amplitude_attack,
                volume_decay: amplitude_decay,
                pitch_attack,
                pitch_decay,
            } => {
                let volume_envelope_index = context.add_mono_node(
                    EnvPerc::new()
                        .sr(SAMPLE_RATE)
                        .attack(*amplitude_attack)
                        .decay(*amplitude_decay),
                );

                let pitch_envelope_index = context.add_mono_node(
                    EnvPerc::new()
                        .sr(SAMPLE_RATE)
                        .attack(*pitch_attack)
                        .decay(*pitch_decay),
                );

                let beat_index =
                    context.add_mono_node(Impulse::new().freq(*bpm / 60.0).sr(SAMPLE_RATE));

                let kick_pitch = context.add_mono_node(Mul::new(500.0));
                let kick_pitch_baseline = context.add_mono_node(Add::new(400.0));
                let kick_synth = context.add_mono_node(SinOsc::new().sr(SAMPLE_RATE));

                let pre_master = context.add_mono_node(Mul::new(1.0));
                let volume_index = context.add_mono_node(Mul::new(1.0));

                context.chain(vec![beat_index, volume_envelope_index, pre_master]);

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

/// Load settings from a file, or use the default settings if no settings are
/// found
fn get_settings() -> state::Settings {
    if let Ok(toml) = std::fs::read_to_string("useful-noise.toml") {
        let settings = toml::from_str::<state::Settings>(&toml);
        if let Ok(settings) = settings {
            settings
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
