use super::ray::Ray;
use super::algebra::V3;

use num::Float;
use generic_array::ArrayLength;

pub enum Event<C>
where
    C: Float,
{
    Emission,
    Decay,
    Diffuse,
    Reflect,
    Refract(C),
}

pub trait Material {
    type Coordinate: Float;
    type Probability: Float;
    type FrequencySize: ArrayLength<u32>
        + ArrayLength<Self::Coordinate>
        + ArrayLength<Self::Probability>;

    fn fate(
        &self,
        frequency: usize,
        emission: Self::Probability,
        event: Self::Probability,
    ) -> Event<Self::Coordinate>;
}

pub struct Intersect<'a, M>
where
    M: Material,
{
    pub position: V3<M::Coordinate>,
    pub normal: V3<M::Coordinate>,
    pub material: &'a M,
}

pub trait Scene {
    type Material: Material;

    fn find_intersect<'a>(
        &'a self,
        ray: &Ray<<Self::Material as Material>::Coordinate>,
    ) -> Option<Intersect<'a, Self::Material>>;
}
