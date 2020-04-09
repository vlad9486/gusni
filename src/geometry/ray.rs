use super::algebra::V3;
use super::scene::Scene;
use crate::light::Density;

use serde::{Serialize, Deserialize};
use num::Float;
use generic_array::ArrayLength;
use rand::Rng;

#[derive(Clone, Serialize, Deserialize)]
pub struct Ray<C>
where
    C: Float,
{
    position: V3<C>,
    direction: V3<C>,
    frequency: usize,
}

impl<C> Ray<C>
where
    C: Default + Float,
{
    pub fn new(position: V3<C>, direction: V3<C>, frequency: usize) -> Self {
        Ray {
            position: position,
            direction: direction,
            frequency: frequency,
        }
    }

    pub fn position(&self) -> &V3<C> {
        &self.position
    }

    pub fn direction(&self) -> &V3<C> {
        &self.direction
    }

    pub fn trace<S, N, R>(&self, scene: &S, rng: &mut R) -> usize
    where
        S: Scene<N, C>,
        N: ArrayLength<C> + ArrayLength<Density>,
        R: Rng,
    {
        self.trace_inner(scene, rng, 0)
    }

    fn trace_inner<S, N, R>(&self, scene: &S, rng: &mut R, level: usize) -> usize
    where
        S: Scene<N, C>,
        N: ArrayLength<C> + ArrayLength<Density>,
        R: Rng,
    {
        use crate::light::Event;
        use std::f32::consts::PI;

        let max_level = 7;
        if level > max_level {
            return 0;
        };

        match scene.find_intersect(self) {
            Some(result) => {
                let dice = [rng.gen_range(0.0, 1.0), rng.gen_range(0.0, 1.0)];
                let fate = result.material.fate(self.frequency, dice);
                let photon = if fate.emission { 1 } else { 0 };
                match fate.event {
                    Event::Decay => photon,
                    Event::Diffuse => {
                        let a = C::from(rng.gen_range(0.0, 2.0 * PI)).unwrap();
                        let z = C::from(rng.gen_range(-1.0, 1.0)).unwrap();
                        let new = self.diffuse(&result.position, &result.normal, [a, z]);
                        photon + new.trace_inner(scene, rng, level + 1)
                    },
                    Event::Reflect => {
                        let new = self.reflect(&result.position, &result.normal);
                        photon + new.trace_inner(scene, rng, level + 1)
                    },
                    Event::Refract(factor) => {
                        let new = self.refract(&result.position, &result.normal, factor);
                        photon + new.trace_inner(scene, rng, level + 1)
                    },
                }
            },
            None => 0,
        }
    }

    fn diffuse(&self, position: &V3<C>, normal: &V3<C>, dice: [C; 2]) -> Self {
        let [a, z] = dice;
        let r = (C::one() - z * z).sqrt();
        let x = r * a.sin();
        let y = r * a.cos();

        let v = V3::new(x, y, z);
        let direction = if &v * normal >= C::zero() { v } else { -&v };

        Ray {
            position: position + &(&direction * C::epsilon()),
            direction: direction,
            frequency: self.frequency.clone(),
        }
    }

    fn reflect(&self, position: &V3<C>, normal: &V3<C>) -> Self {
        let incident = &self.direction;
        let dot_product = incident * normal;
        let direction = &(normal * (C::from(-2.0).unwrap() * dot_product)) + incident;

        Ray {
            position: position + &(&direction * C::epsilon()),
            direction: direction,
            frequency: self.frequency.clone(),
        }
    }

    fn refract(&self, position: &V3<C>, normal: &V3<C>, factor: C) -> Self {
        let incident = &self.direction;
        let temp = incident.cross(normal).cross(normal);
        let sin_b_sq = (&temp * &temp) * (factor * factor);
        if sin_b_sq < C::one() {
            let cos_b = (C::one() - sin_b_sq).sqrt();
            let direction = &(&temp * factor) - &(normal * cos_b);
            Ray {
                position: position + &(&direction * C::epsilon()),
                direction: direction,
                frequency: self.frequency.clone(),
            }
        } else {
            self.reflect(position, normal)
        }
    }
}
