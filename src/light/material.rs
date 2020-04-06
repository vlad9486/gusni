use super::beam::Beam;
use super::color::Density;

use serde::{Serialize, Deserialize};
use num::Float;
use generic_array::ArrayLength;

pub struct Fate<C>
where
    C: Float,
{
    pub emission: bool,
    pub event: Event<C>,
}

pub enum Event<C>
where
    C: Float,
{
    Decay,
    Diffuse,
    Reflect,
    Refract(C),
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Material<N, C>
where
    N: ArrayLength<C> + ArrayLength<Density>,
    C: Default + Float,
{
    emission: Beam<Density, N>,
    diffuse: Beam<Density, N>,
    reflection: Beam<Density, N>,
    refraction: Beam<Density, N>,
    refraction_factor: Beam<C, N>,
}

impl<N, C> Material<N, C>
where
    N: ArrayLength<C> + ArrayLength<Density>,
    C: Default + Float,
{
    pub fn new(
        emission: Beam<Density, N>,
        diffuse: Beam<Density, N>,
        reflection: Beam<Density, N>,
        refraction: Beam<Density, N>,
        refraction_factor: Beam<C, N>,
    ) -> Self {
        Material {
            emission: emission,
            diffuse: diffuse,
            reflection: reflection,
            refraction: refraction,
            refraction_factor: refraction_factor,
        }
    }

    pub fn fate(&self, frequency: usize, dice: [Density; 2]) -> Fate<C> {
        let [emission, event] = dice;
        let emission = self.emission.project(frequency) < emission;
        let diffuse = self.diffuse.project(frequency);
        let reflection = self.reflection.project(frequency);
        let refraction = self.refraction.project(frequency);
        let event = if event < diffuse {
            Event::Diffuse
        } else if event < diffuse + reflection {
            Event::Reflect
        } else if event < diffuse + reflection + refraction {
            Event::Refract(self.refraction_factor.project(frequency))
        } else {
            Event::Decay
        };

        Fate {
            emission: emission,
            event: event,
        }
    }
}
