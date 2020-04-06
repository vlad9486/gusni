#![forbid(unsafe_code)]
#![allow(non_shorthand_field_patterns)]

mod algebra;
mod ray;
mod primitive;
mod scene;
mod color;
mod beam;
mod material;
mod sample;

pub use self::sample::{Sample, Eye, Size};

pub use self::scene::{Scene, Intersect};
pub use self::primitive::{Surface, Sphere};
pub use self::ray::Ray;
pub use self::algebra::V3;

pub use self::material::{Material, Fate, Event};
pub use self::beam::Beam;
pub use self::color::Density;
