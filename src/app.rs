use spectrum_analyzer::scaling::divide_by_N_sqrt;
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};

use dasp::{signal, Sample, Signal};
use egui_plot::{Line, Plot, PlotPoints};
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    // Example stuff:
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,
}

impl Default for App {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let one_sec = 44000;
        let hz = signal::rate(44000.0).const_hz(440.0);
        let synth: Vec<_> = hz
            .clone()
            // .noise_simplex()
            .sine()
            .take(one_sec)
            // .hain(hz.clone().saw().take(one_sec))
            // .chain(hz.clone().square().take(one_sec))
            // .chain(hz.clone().noise_simplex().take(one_sec))
            // .chain(signal::noise(0).take(one_sec))
            .map(|s| s.to_sample::<f32>() * 0.2)
            .collect();
        let samples = synth.as_slice();
        let hann_window = hann_window(&samples[0..2048]);
        // calc spectrum
        let spectrum_hann_window = samples_fft_to_spectrum(
            // (windowed) samples
            &hann_window,
            // sampling rate
            44100,
            // optional frequency limit: e.g. only interested in frequencies 50 <= f <= 150?
            FrequencyLimit::All,
            // optional scale
            Some(&divide_by_N_sqrt),
        )
        .unwrap();

        let sin: PlotPoints = spectrum_hann_window
            .data()
            .iter()
            .map(|(fr, fr_val)| [fr.val() as f64, fr_val.val() as f64])
            .collect();
        let line = Line::new(sin);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.label("Spectrum");

                Plot::new("my_plot")
                    .view_aspect(2.0)
                    .show(ui, |plot_ui| plot_ui.line(line));
            });
        });
    }
}
