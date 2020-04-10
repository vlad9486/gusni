use super::beam::Beam;
use super::color::Density;
use crate::geometry::{Event, Material};

use serde::{Serialize, Deserialize};
use num::Float;
use generic_array::ArrayLength;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct BeamMaterial<N, C>
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

impl<N, C> BeamMaterial<N, C>
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
        BeamMaterial {
            emission: emission,
            diffuse: diffuse,
            reflection: reflection,
            refraction: refraction,
            refraction_factor: refraction_factor,
        }
    }
}

impl<N, C> Material for BeamMaterial<N, C>
where
    N: ArrayLength<u32> + ArrayLength<C> + ArrayLength<Density>,
    C: Default + Float,
{
    type Coordinate = C;
    type Probability = Density;
    type FrequencySize = N;

    fn fate(
        &self,
        frequency: usize,
        emission: Self::Probability,
        event: Self::Probability,
    ) -> Event<Self::Coordinate> {
        if emission < self.emission.project(frequency) {
            Event::Emission
        } else {
            let diffuse = self.diffuse.project(frequency);
            let reflection = self.reflection.project(frequency);
            let refraction = self.refraction.project(frequency);
            if event < diffuse {
                Event::Diffuse
            } else if event < diffuse + reflection {
                Event::Reflect
            } else if event < diffuse + reflection + refraction {
                Event::Refract(self.refraction_factor.project(frequency))
            } else {
                Event::Decay
            }
        }
    }
}
