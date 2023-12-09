use nannou_audio::Stream;
use nannou_egui::Egui;
use serde::{Deserialize, Serialize};

use crate::sound::{Audio, SoundParam};

impl Default for Settings {
    fn default() -> Self {
        Self {
            glicol_indices: vec![],
            ui_params: vec![
                SoundParam::Sine {
                    volume: 0.5,
                    freq: 200.0,
                },
                SoundParam::Brownish {
                    knob_a: 0.1,
                    volume: 0.5,
                    low_pass_freq: 500.0,
                },
            ],
        }
    }
}

pub struct Model {
    pub egui: Egui,
    pub settings: Settings,
    pub stream: Stream<Audio>,
}

#[derive(Serialize, Deserialize)]
pub struct Settings {
    #[serde(skip)]
    pub glicol_indices: Vec<crate::sound::NodeIndexSet>,
    pub ui_params: Vec<crate::sound::SoundParam>,
}
