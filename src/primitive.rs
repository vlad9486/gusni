use super::ray::Ray;
use super::algebra::V3;
use super::color::Density;
use super::material::Material;
use super::scene::{Scene, Intersect};

use std::cmp::Ordering;
use serde::{Serialize, Deserialize};
use num::Float;
use generic_array::ArrayLength;

pub trait Surface<N, C>
where
    Self: Sized,
    N: ArrayLength<C> + ArrayLength<Density>,
    C: Default + Float,
{
    type Info: PartialOrd;

    fn intersect(&self, ray: &Ray<C>) -> Option<Self::Info>;
    fn result<'a>(&'a self, ray: &Ray<C>, info: Self::Info) -> Intersect<'a, N, C>;

    fn find_intersect<'a, I>(v: I, ray: &Ray<C>) -> Option<(&'a Self, Self::Info)>
    where
        I: Iterator<Item = &'a Self>,
    {
        v
            .flat_map(|this| match this.intersect(ray) {
                Some(info) => Some((this, info)),
                None => None,
            })
            .min_by(|lhs, rhs| {
                lhs.1.partial_cmp(&rhs.1).unwrap_or(Ordering::Less)
            })
    }
}

impl<S, N, C> Scene<N, C> for [S]
where
    N: ArrayLength<C> + ArrayLength<Density>,
    C: Default + Float,
    S: Surface<N, C>,
{
    fn find_intersect<'a>(&'a self, ray: &Ray<C>) -> Option<Intersect<'a, N, C>> {
        Surface::find_intersect(self.iter(), ray)
            .map(|(this, info)| this.result(ray, info))
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Sphere<N, C>
where
    N: ArrayLength<C> + ArrayLength<Density>,
    C: Default + Float,
{
    center: V3<C>,
    radius: C,
    material: Material<N, C>,
}

impl<N, C> Sphere<N, C>
where
    N: ArrayLength<C> + ArrayLength<Density>,
    C: Default + Float,
{
    pub fn new(center: V3<C>, radius: C, material: Material<N, C>) -> Self {
        Sphere {
            center: center,
            radius: radius,
            material: material,
        }
    }
}

pub struct SphereInfo<C>
where
    C: Float,
{
    time: C,
    side: bool,
}

impl<C> PartialEq for SphereInfo<C>
where
    C: Float,
{
    fn eq(&self, other: &Self) -> bool {
        self.time.eq(&other.time)
    }
}

impl<C> PartialOrd for SphereInfo<C>
where
    C: Float,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.time.partial_cmp(&other.time)
    }
}

impl<N, C> Surface<N, C> for Sphere<N, C>
where
    N: ArrayLength<C> + ArrayLength<Density>,
    C: Default + Float,
{
    type Info = SphereInfo<C>;

    fn intersect(&self, ray: &Ray<C>) -> Option<Self::Info> {
        let q = &self.center - &ray.position();
        let p = ray.direction();
        let r = self.radius;

        let b = p * &q;
        let (side, d) = {
            let s = &q * &q - r * r;
            (s >= C::zero(), b * b - s)
        };

        let time = if d < C::zero() {
            None
        } else {
            let t0 = b - d.sqrt();
            let t1 = b + d.sqrt();
            if t0 >= C::zero() {
                Some(t0)
            } else if t1 >= C::zero() {
                Some(t1)
            } else {
                None
            }
        };

        time.map(|time| SphereInfo {
            time: time,
            side: side,
        })
    }

    fn result(&self, ray: &Ray<C>, info: Self::Info) -> Intersect<N, C> {
        let position = ray.position() + &(ray.direction() * info.time);
        let radius = if info.side { self.radius } else { -self.radius };
        let normal = &(&position - &self.center) / radius;
        Intersect {
            position: position,
            normal: normal,
            material: &self.material,
        }
    }
}
