use crate::core::{V3, Ray, Scene, Side, Intersect, Material};

use std::cmp::Ordering;
use serde::{Serialize, Deserialize};
use num::Float;

pub trait Surface<C>
where
    Self: Sized,
    C: Float,
{
    type Info: PartialOrd;
    type Material: Material<C>;

    fn intersect(&self, ray: &Ray<C>) -> Option<Self::Info>;

    fn result<'a>(&'a self, ray: &Ray<C>, info: Self::Info) -> Intersect<'a, Self::Material, C>;

    fn find_intersect<'a>(v: &'a [Self], ray: &Ray<C>) -> Option<(&'a Self, Self::Info)> {
        v.iter()
            .flat_map(|this| this.intersect(ray).map(|info| (this, info)))
            .min_by(|lhs, rhs| lhs.1.partial_cmp(&rhs.1).unwrap_or(Ordering::Less))
    }
}

impl<S, C> Scene<C> for Vec<S>
where
    S: Surface<C>,
    C: Float,
{
    type Material = S::Material;

    fn find_intersect<'a>(&'a self, ray: &Ray<C>) -> Option<Intersect<'a, Self::Material, C>> {
        Surface::find_intersect(self.as_ref(), ray).map(|(this, info)| this.result(ray, info))
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Sphere<M, C>
where
    M: Material<C>,
    C: Float,
{
    center: V3<C>,
    radius: C,
    material: M,
}

impl<M, C> Sphere<M, C>
where
    M: Material<C>,
    C: Float,
{
    pub fn new(center: V3<C>, radius: C, material: M) -> Self {
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
    side: Side,
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

impl<M, C> Surface<C> for Sphere<M, C>
where
    M: Material<C>,
    C: Float,
{
    type Info = SphereInfo<C>;
    type Material = M;

    fn intersect(&self, ray: &Ray<C>) -> Option<Self::Info> {
        use num::Zero;

        let zero = <C as Zero>::zero();

        let q = &self.center - ray.position();
        let p = ray.direction();
        let r = self.radius;

        let b = p * &q;
        let (outer, d) = {
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
            side: if outer { Side::Outer } else { Side::Inner },
        })
    }

    fn result<'a>(&'a self, ray: &Ray<C>, info: Self::Info) -> Intersect<'a, Self::Material, C> {
        let position = ray.position() + &(ray.direction() * info.time);
        let radius = if info.side.outer() {
            self.radius
        } else {
            -self.radius
        };
        let normal = &(&position - &self.center) / radius;
        Intersect {
            position: position,
            normal: normal,
            material: &self.material,
            side: info.side,
        }
    }
}
