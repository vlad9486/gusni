use crate::core::{Material, WaveLength, Event};

use num::Float;

pub enum CustomMaterial {
    SemiMirrorRed,
    DiffuseBlue,
    DiffuseWhite,
    Light,
}

impl<C> Material<C> for CustomMaterial
where
    C: Float,
{
    fn fate(&self, wave_length: &WaveLength, emission: f64, event: f64) -> Event<C> {
        match self {
            &CustomMaterial::SemiMirrorRed => {
                let (r, _, _) = wave_length.clone().color().tuple(false);
                if event < r {
                    Event::Diffuse
                } else if event < 2.0 * r {
                    Event::Reflect
                } else {
                    Event::Decay
                }
            },
            &CustomMaterial::DiffuseBlue => {
                let (_, _, b) = wave_length.clone().color().tuple(false);
                if event < b {
                    Event::Diffuse
                } else {
                    Event::Decay
                }
            },
            &CustomMaterial::DiffuseWhite => Event::Diffuse,
            &CustomMaterial::Light => {
                let _ = emission;
                Event::Emission
            },
        }
    }
}
