use super::light::{Rgb, Density};
use super::geometry::{V3, Scene, Ray};

use std::marker::PhantomData;
use serde::{Serialize, Deserialize};
use num::Float;
use rand::Rng;
use generic_array::{GenericArray, ArrayLength};

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
pub struct Sample<N, C>
where
    N: ArrayLength<u32> + ArrayLength<C> + ArrayLength<Density>,
    C: Default + Float,
{
    size: Size,
    sample_count: u64,
    data: Vec<GenericArray<u32, N>>,
    phantom_data: PhantomData<C>,
}

impl<N, C> Sample<N, C>
where
    N: ArrayLength<u32> + ArrayLength<C> + ArrayLength<Density>,
    C: Default + Float,
{
    pub fn new(size: Size) -> Self {
        let capacity = (size.horizontal_count * size.vertical_count) as usize;
        let mut data = Vec::with_capacity(capacity);
        data.resize(capacity, GenericArray::default());
        Sample {
            size: size,
            sample_count: 0,
            data: data,
            phantom_data: PhantomData,
        }
    }

    pub fn size(&self) -> &Size {
        &self.size
    }

    pub fn trace<S, R>(&mut self, rng: &mut R, eye: &Eye<C>, scene: &S)
    where
        S: Scene<N, C>,
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
                    let photon_number = ray.trace(scene, rng) as u32;
                    let index = (i * self.size.horizontal_count + j) as usize;
                    self.data[index][frequency] += photon_number;
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
        for photons in &self.data {
            let pixel =
                photons
                    .iter()
                    .enumerate()
                    .fold(Rgb::default(), |a, (frequency, &density)| {
                        a + (Rgb::monochromatic::<N>(frequency) * (density as Density))
                    });
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
