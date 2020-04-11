use super::algebra::V3;
use super::scene::{Scene, Event, Material};
use super::wave::WaveLength;

use serde::{Serialize, Deserialize};
use num::Float;
use rand::Rng;

#[derive(Clone, Serialize, Deserialize)]
pub struct Ray<C>
where
    C: Float,
{
    position: V3<C>,
    direction: V3<C>,
    wave_length: WaveLength,
}

impl<C> Ray<C>
where
    C: Float,
{
    pub fn new(position: V3<C>, direction: V3<C>, wave_length: WaveLength) -> Self {
        Ray {
            position: position,
            direction: direction,
            wave_length: wave_length,
        }
    }

    pub fn position(&self) -> &V3<C> {
        &self.position
    }

    pub fn direction(&self) -> &V3<C> {
        &self.direction
    }

    pub fn trace<S, R>(&self, scene: &S, rng: &mut R) -> bool
    where
        S: Scene<C>,
        R: Rng,
    {
        self.trace_inner(scene, rng, 0)
    }

    fn trace_inner<S, R>(&self, scene: &S, rng: &mut R, level: usize) -> bool
    where
        S: Scene<C>,
        R: Rng,
    {
        use std::f32::consts::PI;

        let max_level = 7;
        if level > max_level {
            return false;
        };

        match scene.find_intersect(self) {
            Some(result) => {
                let emission = rng.gen_range(0.0, 1.0);
                let event = rng.gen_range(0.0, 1.0);
                let fate = result.material.fate(&self.wave_length, emission, event);
                match fate {
                    Event::Emission => true,
                    Event::Decay => false,
                    Event::Diffuse => {
                        let a = C::from(rng.gen_range(0.0, 2.0 * PI)).unwrap();
                        let z = C::from(rng.gen_range(-1.0, 1.0)).unwrap();
                        let new = self.diffuse(&result.position, &result.normal, a, z);
                        new.trace_inner(scene, rng, level + 1)
                    },
                    Event::Reflect => {
                        let new = self.reflect(&result.position, &result.normal);
                        new.trace_inner(scene, rng, level + 1)
                    },
                    Event::Refract(factor) => {
                        let new = self.refract(&result.position, &result.normal, factor);
                        new.trace_inner(scene, rng, level + 1)
                    },
                }
            },
            None => false,
        }
    }

    fn diffuse(&self, position: &V3<C>, normal: &V3<C>, a: C, z: C) -> Self {
        let r = (C::one() - z * z).sqrt();
        let x = r * a.sin();
        let y = r * a.cos();

        let v = V3::new(x, y, z);
        let direction = if &v * normal >= C::zero() { v } else { -&v };

        Ray {
            position: position + &(&direction * C::epsilon()),
            direction: direction,
            wave_length: self.wave_length.clone(),
        }
    }

    fn reflect(&self, position: &V3<C>, normal: &V3<C>) -> Self {
        let incident = &self.direction;
        let dot_product = incident * normal;
        let direction = &(normal * (C::from(-2.0).unwrap() * dot_product)) + incident;

        Ray {
            position: position + &(&direction * C::epsilon()),
            direction: direction,
            wave_length: self.wave_length.clone(),
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
                wave_length: self.wave_length.clone(),
            }
        } else {
            self.reflect(position, normal)
        }
    }
}