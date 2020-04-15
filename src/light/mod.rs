use crate::core::{Material, WaveLength, Event, Side};

pub enum CustomMaterial {
    SemiMirrorRed,
    Glass(bool),
    DiffuseRed,
    DiffuseGreen,
    DiffuseBlue,
    DiffuseWhite,
    Light,
}

impl Material<f64> for CustomMaterial {
    fn fate(&self, wave_length: &WaveLength, side: Side, emission: f64, event: f64) -> Event<f64> {
        match self {
            &CustomMaterial::SemiMirrorRed => {
                let (r, _, _) = wave_length.color().tuple(false);
                if event < r {
                    Event::Diffuse
                } else if event < 2.0 * r {
                    Event::Reflect
                } else {
                    Event::Decay
                }
            },
            &CustomMaterial::Glass(inverse) => {
                if event < 0.0 {
                    Event::Diffuse
                } else {
                    let l = wave_length.0 / 1000.0;
                    let x = (0.9 - l) / 0.5;
                    let index = 1.51 + 0.04 * x * x;
                    Event::Refract(if inverse ^ side.outer() { index } else { 1.0 / index })
                }
            },
            &CustomMaterial::DiffuseRed => {
                let (r, g, b) = wave_length.color().tuple(false);
                if event < r + g * 0.2 + b * 0.2 {
                    Event::Diffuse
                } else {
                    Event::Decay
                }
            },
            &CustomMaterial::DiffuseGreen => {
                let (r, g, b) = wave_length.color().tuple(false);
                if event < r * 0.2 + g + b * 0.2 {
                    Event::Diffuse
                } else {
                    Event::Decay
                }
            },
            &CustomMaterial::DiffuseBlue => {
                let (r, g, b) = wave_length.color().tuple(false);
                if event < r * 0.2 + g * 0.2 + b {
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
