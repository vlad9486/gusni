use crate::core::{Material, WaveLength, Event, Side};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum CustomMaterial {
    SemiMirrorRed,
    Mirror,
    Glass { inverse: bool },
    DiffuseRed,
    DiffuseGreen,
    DiffuseBlue,
    DiffuseWhite,
    Light { temperature: f64 },
}

impl Material<f64> for CustomMaterial {
    fn fate(&self, wave_length: &WaveLength, side: Side, emission: f64, event: f64) -> Event<f64> {
        match self {
            &CustomMaterial::SemiMirrorRed => {
                let (r, _, _) = wave_length.color().tuple(false);
                if event < r * 0.5 {
                    Event::Diffuse
                } else {
                    Event::Reflect(1.0)
                }
            },
            &CustomMaterial::Mirror => Event::Reflect(1.0),
            &CustomMaterial::Glass { inverse: inverse } => {
                if event < 0.0 {
                    Event::Diffuse
                } else {
                    let l = wave_length.0 / 1000.0;
                    let x = (0.9 - l) / 0.5;
                    let index = 1.51 + 0.04 * x * x;
                    Event::Refract(if inverse ^ side.outer() {
                        index
                    } else {
                        1.0 / index
                    })
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
            &CustomMaterial::Light { temperature: t } => {
                if emission < 1.0 {
                    let l = wave_length.0;
                    let n = b(l, t);
                    Event::Emission(n)
                } else {
                    Event::Decay
                }
            },
        }
    }
}

// not normalized
fn b(l: f64, t: f64) -> f64 {
    use std::f64;

    // constants
    let k = 1.380649;
    let c = 2.99792458;
    let h = 6.62607015;

    // Planck's law
    1e12 * 2.0 * h * c * c / ((l * l * l * l * l) * (f64::exp(1e6 * h * c / (l * k * t)) - 1.0))
}

#[cfg(test)]
mod test {
    #[test]
    fn basic() {
        use super::b;

        (360..830)
            .map(|l| b(l as f64, 6000.0))
            .for_each(|b| println!("{}", b));
    }
}
