use super::light::{Density, Beam};
use super::geometry::{V3, Scene, Ray};

use std::{
    marker::PhantomData,
    ops::{Add, AddAssign},
};
use serde::{Serialize, Deserialize};
use num::Float;
use rand::Rng;
use generic_array::ArrayLength;

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
    pub fn ray(&self, x: C, y: C, width: u32, height: u32, frequency: usize) -> Ray<C> {
        let x = self.width * (x / C::from(width).unwrap() - C::from(0.5).unwrap());
        let y = self.height * (y / C::from(height).unwrap() - C::from(0.5).unwrap());
        let tangent = &(&self.right * x) + &(&self.up * y);
        let direction = (&(&self.forward * self.distance) + &tangent).normalize();
        Ray::new(self.position.clone(), direction, frequency)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Sample<N, C>
where
    Beam<u32, N>: Default + Clone,
    N: ArrayLength<u32> + ArrayLength<C> + ArrayLength<Density>,
    C: Default + Float,
{
    width: u32,
    height: u32,
    sample_count: u32,
    data: Vec<Beam<u32, N>>,
    phantom_data: PhantomData<C>,
}

impl<N, C> Sample<N, C>
where
    Beam<u32, N>: Default + Clone,
    N: ArrayLength<u32> + ArrayLength<C> + ArrayLength<Density>,
    C: Default + Float,
{
    pub fn new(width: u32, height: u32) -> Self {
        let capacity = (width * height) as usize;
        let mut data = Vec::with_capacity(capacity);
        data.resize(capacity, Beam::default());
        Sample {
            width: width,
            height: height,
            sample_count: 0,
            data: data,
            phantom_data: PhantomData,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn trace<S, R>(&mut self, rng: &mut R, eye: &Eye<C>, scene: &S)
    where
        S: Scene<N, C>,
        R: Rng,
    {
        for i in 0..self.height {
            for j in 0..self.width {
                for frequency in 0..N::to_usize() {
                    let dx = rng.gen_range(-0.5, 0.5);
                    let dy = rng.gen_range(-0.5, 0.5);
                    let x = C::from(j).unwrap() + C::from(dx).unwrap();
                    let y = C::from(i).unwrap() + C::from(dy).unwrap();
                    let ray = eye.ray(x, y, self.width, self.height, frequency);
                    let photon_number = ray.trace(scene, rng) as u32;
                    let index = (i * self.width + j) as usize;
                    self.data[index].add_photons(frequency, photon_number);
                }
            }
        }

        self.sample_count += 1;
    }

    pub fn bitmap(&self) -> Vec<u8> {
        let capacity = (3 * self.width * self.height) as usize;
        let mut b = Vec::with_capacity(capacity);
        let to_byte = |a: Density| -> u8 {
            if a >= 1.0 {
                255
            } else if a <= 0.0 {
                0
            } else {
                (a * 255.0) as u8
            }
        };
        for beam in &self.data {
            let pixel = beam.to_rgb();
            b.push(to_byte(pixel.project(0) / (self.sample_count as Density)));
            b.push(to_byte(pixel.project(1) / (self.sample_count as Density)));
            b.push(to_byte(pixel.project(2) / (self.sample_count as Density)));
        }
        b
    }
}

impl<N, C> AddAssign for Sample<N, C>
where
    Beam<u32, N>: Default + Clone,
    N: ArrayLength<u32> + ArrayLength<C> + ArrayLength<Density>,
    C: Default + Float,
{
    fn add_assign(&mut self, rhs: Self) {
        assert_eq!(self.width, rhs.width);
        assert_eq!(self.height, rhs.height);

        self.sample_count += rhs.sample_count;
        for i in 0..((self.height * self.width) as usize) {
            self.data[i] += rhs.data[i].clone();
        }
    }
}

impl<N, C> Add for Sample<N, C>
where
    Beam<u32, N>: Default + Clone,
    N: ArrayLength<u32> + ArrayLength<C> + ArrayLength<Density>,
    C: Default + Float,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut s = self;
        s += rhs;
        s
    }
}
