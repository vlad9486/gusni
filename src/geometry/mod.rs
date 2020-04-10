mod algebra;
mod ray;
mod primitive;
mod scene;

pub use self::scene::{Scene, Intersect, Event, Material};
pub use self::primitive::{Surface, Sphere};
pub use self::ray::Ray;
pub use self::algebra::V3;
