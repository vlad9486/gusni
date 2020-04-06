use super::ray::Ray;
use super::algebra::V3;
use super::color::Density;
use super::material::Material;

use num::Float;
use generic_array::ArrayLength;

pub trait Scene<N, C>
where
    N: ArrayLength<C> + ArrayLength<Density>,
    C: Default + Float,
{
    fn find_intersect<'a>(&'a self, ray: &Ray<C>) -> Option<Intersect<'a, N, C>>;
}

pub struct Intersect<'a, N, C>
where
    N: ArrayLength<C> + ArrayLength<Density>,
    C: Default + Float,
{
    pub position: V3<C>,
    pub normal: V3<C>,
    pub material: &'a Material<N, C>,
}
