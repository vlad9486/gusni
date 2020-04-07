use super::color::{Rgb, Density};

use std::ops::{Add, Mul, Div};
use serde::{Serialize, Deserialize};
use num::Float;
use generic_array::{GenericArray, ArrayLength};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Beam<C, N>
where
    C: Default + Float,
    N: ArrayLength<C>,
{
    #[serde(bound(
        serialize = "C: Serialize",
        deserialize = "C: Default + Deserialize<'de>"
    ))]
    powers: GenericArray<C, N>,
}

impl<C, N> Beam<C, N>
where
    C: Default + Float,
    N: ArrayLength<C>,
{
    pub fn generate<F>(f: F) -> Self
    where
        F: FnMut(usize) -> C,
    {
        use generic_array::sequence::GenericSequence;

        Beam {
            powers: GenericArray::generate(f),
        }
    }

    pub fn project(&self, frequency: usize) -> C {
        self.powers[frequency]
    }
}

impl<N> Beam<Density, N>
where
    N: ArrayLength<Density>,
{
    // memorize?
    fn basis(index: usize) -> Self {
        let s = Beam::generate(|frequency| Rgb::monochromatic::<N>(frequency).project(index));
        let l = (&s * &s).sqrt();
        &s / l
    }

    pub fn red() -> Self {
        Self::basis(0)
    }

    pub fn green() -> Self {
        Self::basis(1)
    }

    pub fn blue() -> Self {
        Self::basis(2)
    }
}

impl<'a, 'b, C, N> Mul<&'b Beam<C, N>> for &'a Beam<C, N>
where
    C: Default + Float,
    N: ArrayLength<C>,
{
    type Output = C;

    fn mul(self, rhs: &'b Beam<C, N>) -> Self::Output {
        let mut product = C::default();
        for i in 0..N::to_usize() {
            product = product + self.powers[i] * rhs.powers[i];
        }

        product
    }
}

impl<C, N> Add<Beam<C, N>> for Beam<C, N>
where
    C: Default + Float,
    N: ArrayLength<C>,
{
    type Output = Beam<C, N>;

    fn add(self, rhs: Beam<C, N>) -> Self::Output {
        Self::generate(|frequency| self.project(frequency) + rhs.project(frequency))
    }
}

impl<'a, C, N> Mul<C> for &'a Beam<C, N>
where
    C: Default + Float,
    N: ArrayLength<C>,
{
    type Output = Beam<C, N>;

    fn mul(self, rhs: C) -> Self::Output {
        Beam::generate(|frequency| self.powers[frequency] * rhs)
    }
}

impl<'a, C, N> Div<C> for &'a Beam<C, N>
where
    C: Default + Float,
    N: ArrayLength<C>,
{
    type Output = Beam<C, N>;

    fn div(self, rhs: C) -> Self::Output {
        Beam::generate(|frequency| self.powers[frequency] / rhs)
    }
}
