use super::ray::Ray;
use super::algebra::V3;
use super::wave::WaveLength;

use num::Float;

pub enum Side {
    Outer,
    Inner,
}

impl Side {
    pub fn outer(&self) -> bool {
        match self {
            &Side::Outer => true,
            &Side::Inner => false,
        }
    }
}

pub enum Event<C>
where
    C: Float,
{
    Emission(f64),
    Decay,
    Diffuse,
    Reflect,
    Refract(C),
}

pub trait Material<C>
where
    C: Float,
{
    fn fate(&self, wave_length: &WaveLength, side: Side, emission: f64, event: f64) -> Event<C>;
}

pub struct Intersect<'a, M, C>
where
    M: Material<C>,
    C: Float,
{
    pub position: V3<C>,
    pub normal: V3<C>,
    pub material: &'a M,
    pub side: Side,
}

pub trait Scene<C>
where
    C: Float,
{
    type Material: Material<C>;

    fn find_intersect<'a>(&'a self, ray: &Ray<C>) -> Option<Intersect<'a, Self::Material, C>>;
}
