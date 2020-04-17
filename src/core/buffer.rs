use super::algebra::V3;
use super::ray::Ray;
use super::scene::Scene;
use super::wave::{WaveLength, Rgb, WaveLengthFactory};

use std::{
    ops::{Add, AddAssign},
    sync::mpsc,
};
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

#[derive(Debug)]
pub struct Progress {
    pub id: usize,
    pub sample: usize,
    pub index: usize,
}

pub struct Report<'a> {
    pub id: usize,
    pub interval: usize,
    pub sender: &'a mpsc::Sender<Progress>,
}

#[derive(Clone)]
pub struct Buffer<F>
where
    F: WaveLengthFactory,
{
    factory: F,
    width: usize,
    height: usize,
    data: Vec<f64>,
    sample_count: usize,
}

impl<F> Buffer<F>
where
    F: WaveLengthFactory,
{
    pub fn new(width: usize, height: usize, data: Option<Vec<f64>>, factory: F) -> Self {
        Buffer {
            factory: factory,
            width: width,
            height: height,
            data: data.unwrap_or({
                let capacity = width * height * 3;
                let mut data = Vec::with_capacity(capacity);
                data.resize(capacity, 0.0);
                data
            }),
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

    pub fn data(&self) -> &[f64] {
        self.data.as_ref()
    }

    pub fn trace<S, C, R>(
        &mut self,
        rng: &mut R,
        eye: &Eye<C>,
        scene: &S,
        terminate_receiver: Option<&mpsc::Receiver<()>>,
        report: Option<Report<'_>>,
    ) -> bool
    where
        S: Scene<C>,
        C: Float,
        R: Rng,
    {
        for i in 0..self.height {
            for j in 0..self.width {
                let index = i * self.width + j;
                if let &Some(ref report) = &report {
                    if index % report.interval == 0 {
                        let progress = Progress {
                            id: report.id,
                            sample: self.sample_count,
                            index: index,
                        };
                        report.sender.send(progress).unwrap();
                    }
                }
                if self.sample_count == 0 {
                    self.data[index * 3 + 0] = 0.0;
                    self.data[index * 3 + 1] = 0.0;
                    self.data[index * 3 + 2] = 0.0;
                }
                for l in self.factory.iter() {
                    let color = l.color();
                    let dx = rng.gen_range(-0.5, 0.5);
                    let dy = rng.gen_range(-0.5, 0.5);
                    let x = C::from(j).unwrap() + C::from(dx).unwrap();
                    let y = C::from(i).unwrap() + C::from(dy).unwrap();
                    let ray = eye.ray(x, y, self.width, self.height, l);
                    let photon = ray.trace(scene, rng);
                    let (r, g, b) = (color * photon).tuple(false);
                    self.data[index * 3 + 0] += r;
                    self.data[index * 3 + 1] += g;
                    self.data[index * 3 + 2] += b;
                }
                if let Some(terminate_receiver) = terminate_receiver {
                    match terminate_receiver.try_recv() {
                        Ok(()) => {
                            self.sample_count = 0;
                            return false;
                        },
                        _ => (),
                    }
                }
            }
        }

        self.sample_count += 1;
        true
    }

    pub fn write(&self, scale: f64, reverse: bool, buffer: &mut [u8]) {
        if self.sample_count != 0 {
            let mut position = 0;
            for tuple in self.data.chunks(3) {
                let color = Rgb::new(tuple[0].clone(), tuple[1].clone(), tuple[2].clone());
                let color = color * (scale / ((self.sample_count * self.factory.resolution()) as f64));
                color.write(reverse, &mut buffer[position..(position + 3)]);
                position += 3;
            }
        }
    }
}

impl<F> AddAssign<&mut Self> for Buffer<F>
where
    F: WaveLengthFactory,
{
    fn add_assign(&mut self, rhs: &mut Self) {
        if rhs.sample_count == 0 {
            return;
        };

        assert_eq!(self.width, rhs.width);
        assert_eq!(self.height, rhs.height);

        self.sample_count += rhs.sample_count;
        for i in 0..(self.height * self.width * 3) {
            self.data[i] += rhs.data[i];
        }

        rhs.sample_count = 0;
    }
}

impl<F> Add for Buffer<F>
where
    F: WaveLengthFactory,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut s = self;
        let mut rhs = rhs;
        s += &mut rhs;
        s
    }
}
