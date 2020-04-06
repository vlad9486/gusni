use std::ops::{Mul, Div, Add, Sub, Neg};
use serde::{Serialize, Deserialize};
use num::Float;

#[derive(Clone, Serialize, Deserialize)]
pub struct V3<C>
where
    C: Float,
{
    x: C,
    y: C,
    z: C,
}

impl<C> V3<C>
where
    C: Float,
{
    pub fn new(x: C, y: C, z: C) -> Self {
        V3 { x: x, y: y, z: z }
    }

    pub fn cross(&self, rhs: &Self) -> Self {
        V3 {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    pub fn normalize(&self) -> Self {
        let l = (self * self).sqrt();
        self / l
    }
}

impl<'a, 'b, C> Mul<&'b V3<C>> for &'a V3<C>
where
    C: Float,
{
    type Output = C;

    fn mul(self, rhs: &'b V3<C>) -> Self::Output {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }
}

impl<'a, C> Mul<C> for &'a V3<C>
where
    C: Float,
{
    type Output = V3<C>;

    fn mul(self, rhs: C) -> Self::Output {
        V3::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl<'a, C> Div<C> for &'a V3<C>
where
    C: Float,
{
    type Output = V3<C>;

    fn div(self, rhs: C) -> Self::Output {
        V3::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl<'a, 'b, C> Add<&'b V3<C>> for &'a V3<C>
where
    C: Float,
{
    type Output = V3<C>;

    fn add(self, rhs: &'b V3<C>) -> Self::Output {
        V3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl<'a, 'b, C> Sub<&'b V3<C>> for &'a V3<C>
where
    C: Float,
{
    type Output = V3<C>;

    fn sub(self, rhs: &'b V3<C>) -> Self::Output {
        V3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl<'a, C> Neg for &'a V3<C>
where
    C: Float,
{
    type Output = V3<C>;

    fn neg(self) -> Self::Output {
        V3::new(-self.x, -self.y, -self.z)
    }
}
