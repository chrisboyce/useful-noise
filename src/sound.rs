use super::AudioContext;

use petgraph::graph::NodeIndex;
use serde::{Deserialize, Serialize};

pub mod brownish_noise;

/// Holds glicol node indices relevant for a particular sound
#[derive(Clone, Debug)]
pub enum NodeIndexSet {
    Sine {
        volume_index: NodeIndex,
        freq_index: NodeIndex,
    },
    Brownish {
        volume_index: NodeIndex,
        low_pass_index: NodeIndex,
        knob_a_index: NodeIndex,
    },
}

/// Parameters for a particular sounds type
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum SoundParam {
    Sine {
        volume: f32,
        freq: f32,
    },
    Brownish {
        knob_a: f32,
        volume: f32,
        low_pass_freq: f32,
    },
}

pub struct Audio {
    pub context: AudioContext,
}
