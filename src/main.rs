use glicol_synth::{
    filter::ResonantLowPassFilter, operator::Mul, oscillator::SinOsc,
    AudioContext as GlicolAudioContext, AudioContextBuilder, Message,
};
use nannou::prelude::*;
use nannou_audio::{self as audio, Buffer};
use nannou_egui::{self, egui, Egui};

const SAMPLE_RATE: usize = 44_1000;
const FRAME_SIZE: usize = 64;
type AudioContext = GlicolAudioContext<FRAME_SIZE>;
fn main() {
    nannou::app(model).exit(handle_exit).update(update).run();
}

mod sound;
mod state;

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
                sound::SourceParam::Brownish {
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
                    ui.add(egui::Slider::new(low_pass_freq, 0.0..=10000.0));
                    ui.label("Knob A");
                    ui.add(egui::Slider::new(knob_a, 0.0..=1.0));
                });
            }
            (
                sound::SourceParam::Sine { volume, freq },
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
                    ui.add(egui::Slider::new(freq, 40.0..=10000.0));
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

    // Each of our settings will need to have glicol nodes created
    for ui_param in &settings.ui_params {
        // We use this "match" statement to create the node configuration
        // based on which type of `SourceParam` we're setting up
        let node_index_set = match ui_param {
            // Handle "Brownish" noise
            sound::SourceParam::Brownish {
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
                let knob_a_index =
                    context.add_stereo_node(sound::BrownishNoise::new_with_scale(*knob_a));
                let low_pass_index =
                    context.add_stereo_node(ResonantLowPassFilter::new().cutoff(*low_pass_freq));
                let volume_index = context.add_stereo_node(Mul::new(*volume));

                context.chain(vec![
                    knob_a_index,
                    low_pass_index,
                    volume_index,
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
            sound::SourceParam::Sine { volume, freq } => {
                let sine = SinOsc::new().freq(*freq);
                let sine_id = context.add_stereo_node(sine);
                let sine_volume = context.add_stereo_node(Mul::new(*volume));
                context.chain(vec![sine_id, sine_volume, context.destination]);
                sound::NodeIndexSet::Sine {
                    volume_index: sine_volume,
                    freq_index: sine_id,
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
