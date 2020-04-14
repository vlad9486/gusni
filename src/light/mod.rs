use crate::core::{Material, WaveLength, Event};

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
    fn fate(&self, wave_length: &WaveLength, side: bool, emission: f64, event: f64) -> Event<f64> {
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
            &CustomMaterial::Glass(inverse) => {
                if event < 0.05 {
                    Event::Diffuse
                } else {
                    let l = wave_length.0 / 1000.0;
                    let x = (0.9 - l) / 0.5;
                    let index = 1.51 + 0.04 * x * x;
                    Event::Refract(if inverse ^ side { index } else { 1.0 / index })
                }
            },
            &CustomMaterial::DiffuseRed => {
                let (f, _, _) = wave_length.clone().color().tuple(false);
                if event < f {
                    Event::Diffuse
                } else {
                    Event::Decay
                }
            },
            &CustomMaterial::DiffuseGreen => {
                let (_, f, _) = wave_length.clone().color().tuple(false);
                if event < f {
                    Event::Diffuse
                } else {
                    Event::Decay
                }
            },
            &CustomMaterial::DiffuseBlue => {
                let (_, _, f) = wave_length.clone().color().tuple(false);
                if event < f {
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
