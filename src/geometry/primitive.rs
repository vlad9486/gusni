use super::ray::Ray;
use super::algebra::V3;
use super::scene::{Scene, Intersect, Material};

use std::cmp::Ordering;
use serde::{Serialize, Deserialize};
use num::Float;

pub trait Surface
where
    Self: Sized,
{
    type Info: PartialOrd;
    type Material: Material;

    fn intersect(&self, ray: &Ray<<Self::Material as Material>::Coordinate>) -> Option<Self::Info>;

    fn result<'a>(
        &'a self,
        ray: &Ray<<Self::Material as Material>::Coordinate>,
        info: Self::Info,
    ) -> Intersect<'a, Self::Material>;

    fn find_intersect<'a>(
        v: &'a [Self],
        ray: &Ray<<Self::Material as Material>::Coordinate>,
    ) -> Option<(&'a Self, Self::Info)> {
        v.iter()
            .flat_map(|this| match this.intersect(ray) {
                Some(info) => Some((this, info)),
                None => None,
            })
            .min_by(|lhs, rhs| lhs.1.partial_cmp(&rhs.1).unwrap_or(Ordering::Less))
    }
}

impl<S> Scene for Vec<S>
where
    S: Surface,
{
    type Material = S::Material;

    fn find_intersect<'a>(
        &'a self,
        ray: &Ray<<Self::Material as Material>::Coordinate>,
    ) -> Option<Intersect<'a, Self::Material>> {
        Surface::find_intersect(self.as_ref(), ray).map(|(this, info)| this.result(ray, info))
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Sphere<M>
where
    M: Material,
{
    center: V3<M::Coordinate>,
    radius: M::Coordinate,
    material: M,
}

impl<M> Sphere<M>
where
    M: Material,
{
    pub fn new(center: V3<M::Coordinate>, radius: M::Coordinate, material: M) -> Self {
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

impl<M> Surface for Sphere<M>
where
    M: Material,
{
    type Info = SphereInfo<<Self::Material as Material>::Coordinate>;
    type Material = M;

    fn intersect(&self, ray: &Ray<<Self::Material as Material>::Coordinate>) -> Option<Self::Info> {
        use num::Zero;

        let zero = <<Self::Material as Material>::Coordinate as Zero>::zero();

        let q = &self.center - &ray.position();
        let p = ray.direction();
        let r = self.radius;

        let b = p * &q;
        let (side, d) = {
            let s = &q * &q - r * r;
            (s >= zero, b * b - s)
        };

        let time = if d < zero {
            None
        } else {
            let t0 = b - d.sqrt();
            let t1 = b + d.sqrt();
            if t0 >= zero {
                Some(t0)
            } else if t1 >= zero {
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

    fn result<'a>(
        &'a self,
        ray: &Ray<<Self::Material as Material>::Coordinate>,
        info: Self::Info,
    ) -> Intersect<'a, Self::Material> {
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
