use super::AudioContext;

use petgraph::graph::NodeIndex;
use serde::{Deserialize, Serialize};

pub mod brownish_noise;

pub mod debug {
    use glicol_synth::{Buffer, Input, Message, Node};
    use hashbrown::HashMap;

    #[derive(Clone, Debug, PartialEq)]
    pub struct DebugNode;

    impl<const N: usize> Node<N> for DebugNode {
        fn process(&mut self, inputs: &mut HashMap<usize, Input<N>>, output: &mut [Buffer<N>]) {
            let input = match inputs.values().next() {
                None => return,
                Some(input) => input,
            };
            println!("{:?}", &inputs);
            if input.buffers().len() == 1 && output.len() == 2 {
                output[0].copy_from_slice(&input.buffers()[0]);
                output[1].copy_from_slice(&input.buffers()[0]);
            } else {
                for (out_buf, in_buf) in output.iter_mut().zip(input.buffers()) {
                    out_buf.copy_from_slice(in_buf);
                }
            }
        }
        fn send_msg(&mut self, _info: Message) {}
    }
}
/// Holds glicol node indices relevant for a particular sound
#[derive(Clone, Debug)]
pub enum NodeIndexSet {
    Kick {
        am_env_index: NodeIndex,
        fm_env_index: NodeIndex,
        impulse_index: NodeIndex,
        volume_index: NodeIndex,
    },
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
    Kick {
        volume: f32,
        bpm: f32,
        amplitude_attack: f32,
        amplitude_decay: f32,
        pitch_attack: f32,
        pitch_decay: f32,
    },
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
