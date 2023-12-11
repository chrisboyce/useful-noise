use nannou_audio::Stream;
use nannou_egui::Egui;
use serde::{Deserialize, Serialize};

use crate::sound::{Audio, SoundParam};

impl Default for Settings {
    fn default() -> Self {
        Self {
            config_version: 1,
            glicol_indices: vec![],
            ui_params: vec![
                SoundParam::Kick {
                    volume: 0.9800000190734863,
                    bpm: 50.0,
                    volume_attack: 0.0020000000949949026,
                    volume_decay: 0.010999999940395355,
                    pitch_attack: 0.009999999776482582,
                    pitch_decay: 0.10000000149011612,
                },
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
    pub config_version: usize,
    pub ui_params: Vec<crate::sound::SoundParam>,
}
