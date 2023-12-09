use glicol_synth::Message;

use glicol_synth::Node;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct BrownishNoise {
    pub brown_walk_scale: f32,
    pub previous_brown_value: f32,
}

impl BrownishNoise {
    pub fn new_with_scale(scale: f32) -> Self {
        Self {
            brown_walk_scale: scale,
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
