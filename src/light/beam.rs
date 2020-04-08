use super::color::{Rgb, Density};

use std::ops::{Add, Mul, Div, AddAssign};
use serde::{Serialize, Deserialize};
use generic_array::{GenericArray, ArrayLength};

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct Beam<C, N>
where
    C: Default + Clone,
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
    C: Default + Clone,
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
        self.powers[frequency].clone()
    }
}

impl<N> Beam<Density, N>
where
    N: ArrayLength<Density>,
{
    // memorize?
    fn basis(index: usize) -> Self {
        let s = Beam::generate(|frequency| Rgb::monochromatic::<N>(frequency).project(index));
        let l = &s * &s;
        &s / (l * 2.0)
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

impl<C, N> Beam<C, N>
where
    C: Default + Clone + Into<Density>,
    N: ArrayLength<C>,
{
    pub fn to_rgb(&self) -> Rgb {
        self
            .powers
            .iter()
            .enumerate()
            .fold(Rgb::default(), |a, (frequency, density)| {
                a + (Rgb::monochromatic::<N>(frequency) * density.clone().into())
            })
    }
}

impl<N> Beam<u32, N>
where
    N: ArrayLength<u32>,
{
    pub fn add_photons(&mut self, frequency: usize, number: u32) {
        self.powers[frequency] += number;
    }
}

impl<'a, 'b, C, N> Mul<&'b Beam<C, N>> for &'a Beam<C, N>
where
    C: Default + Clone + Mul<C, Output = C> + Add<C, Output = C>,
    N: ArrayLength<C>,
{
    type Output = C;

    fn mul(self, rhs: &'b Beam<C, N>) -> Self::Output {
        let mut product = C::default();
        for frequency in 0..N::to_usize() {
            product = product + self.project(frequency) * rhs.project(frequency);
        }

        product
    }
}

impl<C, N> Add<Beam<C, N>> for Beam<C, N>
where
    C: Default + Clone + Add<C, Output = C>,
    N: ArrayLength<C>,
{
    type Output = Beam<C, N>;

    fn add(self, rhs: Beam<C, N>) -> Self::Output {
        Self::generate(|frequency| self.project(frequency) + rhs.project(frequency))
    }
}

impl<C, N> AddAssign<Beam<C, N>> for Beam<C, N>
where
    C: Default + Clone + AddAssign<C>,
    N: ArrayLength<C>,
{
    fn add_assign(&mut self, rhs: Beam<C, N>) {
        for frequency in 0..N::to_usize() {
            self.powers[frequency] += rhs.project(frequency);
        }
    }
}

impl<'a, C, N> Mul<C> for &'a Beam<C, N>
where
    C: Default + Clone + Mul<C, Output = C>,
    N: ArrayLength<C>,
{
    type Output = Beam<C, N>;

    fn mul(self, rhs: C) -> Self::Output {
        Beam::generate(|frequency| self.project(frequency) * rhs.clone())
    }
}

impl<'a, C, N> Div<C> for &'a Beam<C, N>
where
    C: Default + Clone + Div<C, Output = C>,
    N: ArrayLength<C>,
{
    type Output = Beam<C, N>;

    fn div(self, rhs: C) -> Self::Output {
        Beam::generate(|frequency| self.project(frequency) / rhs.clone())
    }
}

#[cfg(test)]
mod test {
    use super::{Beam, Density};

    #[test]
    fn colors() {
        use generic_array::typenum::U12;

        let b = Beam::<Density, U12>::red();
        println!("red: {:?}", b.to_rgb());
        let b = Beam::<Density, U12>::green();
        println!("green: {:?}", b.to_rgb());
        let b = Beam::<Density, U12>::blue();
        println!("blue: {:?}", b.to_rgb());
        let b = Beam::<Density, U12>::red() + Beam::<Density, U12>::green() + Beam::<Density, U12>::blue();
        println!("white: {:?}", b.to_rgb());
    }

    #[test]
    fn basic() {
        use generic_array::typenum::Unsigned;

        type N = generic_array::typenum::U8;
        let mut white = Beam::<u32, N>::default();
        for f in 0..N::to_usize() {
            white.add_photons(f, 1);
        }
        let rgb = white.to_rgb();
        println!("{:?}", rgb);
    }
}
