use super::light::{Rgb, Density};
use super::geometry::{V3, Scene, Ray};

use serde::{Serialize, Deserialize};
use num::Float;
use rand::Rng;
use generic_array::ArrayLength;

#[derive(Serialize, Deserialize)]
pub struct Size {
    pub horizontal_count: u64,
    pub vertical_count: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Eye<C>
where
    C: Default + Float,
{
    pub position: V3<C>,
    pub forward: V3<C>,
    pub right: V3<C>,
    pub up: V3<C>,

    pub width: C,
    pub height: C,
    pub distance: C,
}

impl<C> Eye<C>
where
    C: Default + Float,
{
    pub fn ray(&self, x: C, y: C, size: &Size, frequency: usize) -> Ray<C> {
        let x = self.width * (x / C::from(size.horizontal_count).unwrap() - C::from(0.5).unwrap());
        let y = self.height * (y / C::from(size.vertical_count).unwrap() - C::from(0.5).unwrap());
        let tangent = &(&self.right * x) + &(&self.up * y);
        let direction = (&(&self.forward * self.distance) + &tangent).normalize();
        Ray::new(self.position.clone(), direction, frequency)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Sample {
    size: Size,
    sample_count: u64,
    data: Vec<Rgb>,
}

impl Sample {
    pub fn new(size: Size) -> Self {
        let capacity = (size.horizontal_count * size.vertical_count) as usize;
        let mut data = Vec::with_capacity(capacity);
        data.resize(capacity, Rgb::default());
        Sample {
            size: size,
            sample_count: 0,
            data: data,
        }
    }

    pub fn sample<S, N, C, R>(&mut self, rng: &mut R, eye: &Eye<C>, scene: &S)
    where
        S: Scene<N, C>,
        N: ArrayLength<C> + ArrayLength<Density>,
        C: Default + Float,
        R: Rng,
    {
        for i in 0..self.size.vertical_count {
            for j in 0..self.size.horizontal_count {
                for frequency in 0..N::to_usize() {
                    let dx = rng.gen_range(-0.5, 0.5);
                    let dy = rng.gen_range(-0.5, 0.5);
                    let x = C::from(j).unwrap() + C::from(dx).unwrap();
                    let y = C::from(i).unwrap() + C::from(dy).unwrap();
                    let ray = eye.ray(x, y, &self.size, frequency);
                    let photon_number = ray.trace(scene, rng) as Density;
                    let index = (i * self.size.horizontal_count + j) as usize;
                    self.data[index] += Rgb::monochromatic(frequency) * photon_number;
                }
            }
        }

        self.sample_count += 1;
    }

    pub fn bitmap(&self, scale: Density) -> Vec<u8> {
        let capacity = (3 * self.size.horizontal_count * self.size.vertical_count) as usize;
        let mut b = Vec::with_capacity(capacity);
        let to_byte = |a: Density| -> u8 {
            if a > 1.0 {
                255
            } else if a < 0.0 {
                0
            } else {
                (a * 255.0) as u8
            }
        };
        for pixel in &self.data {
            b.push(to_byte(
                pixel.project(0) * scale / (self.sample_count as Density),
            ));
            b.push(to_byte(
                pixel.project(1) * scale / (self.sample_count as Density),
            ));
            b.push(to_byte(
                pixel.project(2) * scale / (self.sample_count as Density),
            ));
        }
        b
    }
}
