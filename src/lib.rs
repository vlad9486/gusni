#![forbid(unsafe_code)]
#![allow(non_shorthand_field_patterns)]

mod geometry;
mod light;
mod sample;

pub use self::sample::{Sample, Eye};
pub use self::geometry::{V3, Ray, Scene, Intersect, Surface, Sphere, Material, Event};
pub use self::light::{Density, Beam, BeamMaterial};
