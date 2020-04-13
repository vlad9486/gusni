use super::algebra::V3;
use super::ray::Ray;
use super::scene::Scene;
use super::wave::{WaveLength, WaveLengthLinear, Rgb};

use std::ops::{Add, AddAssign};
use serde::{Serialize, Deserialize};
use num::Float;
use rand::Rng;

#[derive(Clone, Serialize, Deserialize)]
pub struct Eye<C>
where
    C: Float,
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
    C: Float,
{
    pub fn ray(&self, x: C, y: C, width: usize, height: usize, wave_length: WaveLength) -> Ray<C> {
        let x = self.width * (x / C::from(width).unwrap() - C::from(0.5).unwrap());
        let y = self.height * (y / C::from(height).unwrap() - C::from(0.5).unwrap());
        let tangent = &(&self.right * x) + &(&self.up * y);
        let direction = (&(&self.forward * self.distance) + &tangent).normalize();
        Ray::new(self.position.clone(), direction, wave_length)
    }
}

pub struct Buffer {
    width: usize,
    height: usize,
    wave_resolution: usize,
    data: Vec<u32>,
    sample_count: usize,
}

impl Buffer {
    pub fn new(width: usize, height: usize, wave_resolution: usize) -> Self {
        let capacity = width * height * wave_resolution;
        let mut data = Vec::with_capacity(capacity);
        data.resize(capacity, 0);
        Buffer {
            width: width,
            height: height,
            wave_resolution: wave_resolution,
            data: data,
            sample_count: 0,
        }
    }

    pub fn sample_count(&self) -> usize {
        self.sample_count
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn trace<S, C, R>(&mut self, rng: &mut R, eye: &Eye<C>, scene: &S)
    where
        S: Scene<C>,
        C: Float,
        R: Rng,
    {
        for i in 0..self.height {
            for j in 0..self.width {
                for (k, l) in WaveLengthLinear::new(self.wave_resolution).enumerate() {
                    let dx = rng.gen_range(-0.5, 0.5);
                    let dy = rng.gen_range(-0.5, 0.5);
                    let x = C::from(j).unwrap() + C::from(dx).unwrap();
                    let y = C::from(i).unwrap() + C::from(dy).unwrap();
                    let ray = eye.ray(x, y, self.width, self.height, l);
                    let photon = ray.trace(scene, rng);
                    if photon {
                        let index = (i * self.width + j) * self.wave_resolution + k;
                        self.data[index] += 1;
                    }
                }
            }
        }

        self.sample_count += 1;
    }

    pub fn write(&self, scale: f64, reverse: bool, buffer: &mut [u8]) {
        let mut position = 0;
        for beam in self.data.chunks(self.wave_resolution) {
            let color = WaveLengthLinear::new(self.wave_resolution)
                .enumerate()
                .fold(Rgb::default(), |color, (i, wave)| {
                    let density = (beam[i] as f64) * scale
                        / ((self.sample_count * self.wave_resolution) as f64);
                    color + wave.color() * density
                });
            color.write(reverse, &mut buffer[position..(position + 3)]);
            position += 3;
        }
    }
}

impl AddAssign for Buffer {
    fn add_assign(&mut self, rhs: Self) {
        assert_eq!(self.width, rhs.width);
        assert_eq!(self.height, rhs.height);
        assert_eq!(self.wave_resolution, rhs.wave_resolution);

        self.sample_count += rhs.sample_count;
        for i in 0..(self.height * self.width * self.wave_resolution) {
            self.data[i] += rhs.data[i];
        }
    }
}

impl Add for Buffer {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut s = self;
        s += rhs;
        s
    }
}
