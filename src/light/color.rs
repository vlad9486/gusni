use std::ops::{Add, Mul};
use serde::{Serialize, Deserialize};
use generic_array::typenum::Unsigned;

pub type Density = f64;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Rgb {
    r: Density,
    g: Density,
    b: Density,
}

impl Rgb {
    pub const SIZE: usize = 48;

    pub fn new(r: Density, g: Density, b: Density) -> Self {
        Rgb { r: r, g: g, b: b }
    }

    pub fn project(&self, index: usize) -> Density {
        match index {
            0 => self.r,
            1 => self.g,
            _ => self.b,
        }
    }

    #[inline(always)]
    pub fn monochromatic<N>(frequency: usize) -> Self
    where
        N: Unsigned,
    {
        #[rustfmt::skip]
        let table = [
            (392.16, Rgb::new( 2.7420e-03, -6.6577e-04,  1.8052e-02)),
            (396.04, Rgb::new( 5.4765e-03, -1.2443e-03,  3.6357e-02)),
            (400.00, Rgb::new( 9.6727e-03, -2.2245e-03,  6.2402e-02)),
            (404.04, Rgb::new( 1.7115e-02, -4.2803e-03,  1.1750e-01)),
            (408.16, Rgb::new( 2.6140e-02, -5.5954e-03,  1.8551e-01)),
            (412.37, Rgb::new( 3.7021e-02, -1.0157e-02,  2.8857e-01)),
            (416.67, Rgb::new( 4.5837e-02, -1.4089e-02,  4.0887e-01)),
            (421.05, Rgb::new( 5.3023e-02, -1.7832e-02,  5.6178e-01)),
            (425.53, Rgb::new( 5.2523e-02, -2.1708e-02,  6.9983e-01)),
            (430.11, Rgb::new( 4.4169e-02, -1.9982e-02,  7.9826e-01)),
            (434.78, Rgb::new( 3.3055e-02, -1.6627e-02,  8.9167e-01)),
            (439.56, Rgb::new( 1.6352e-02, -8.2876e-03,  9.5923e-01)),
            (444.44, Rgb::new( 0.0000e+00,  0.0000e+00,  1.0000e+00)),
            (449.44, Rgb::new(-2.5906e-02,  1.7249e-02,  9.3271e-01)),
            (454.55, Rgb::new(-5.7758e-02,  4.1259e-02,  8.2927e-01)),
            (459.77, Rgb::new(-9.4556e-02,  6.9343e-02,  7.8949e-01)),
            (465.12, Rgb::new(-1.3876e-01,  1.1121e-01,  6.6582e-01)),
            (470.59, Rgb::new(-1.7938e-01,  1.5627e-01,  6.0273e-01)),
            (476.19, Rgb::new(-2.2034e-01,  2.0877e-01,  4.5473e-01)),
            (481.93, Rgb::new(-2.4523e-01,  2.5546e-01,  3.2281e-01)),
            (487.80, Rgb::new(-2.6831e-01,  3.0728e-01,  2.2315e-01)),
            (493.83, Rgb::new(-2.8965e-01,  3.8802e-01,  1.5790e-01)),
            (500.00, Rgb::new(-2.9504e-01,  4.9084e-01,  1.0740e-01)),
            (506.33, Rgb::new(-2.9346e-01,  6.2585e-01,  6.9385e-02)),
            (512.82, Rgb::new(-2.4020e-01,  7.6114e-01,  3.7247e-02)),
            (519.48, Rgb::new(-1.5696e-01,  9.0124e-01,  1.4718e-02)),
            (526.32, Rgb::new( 0.0000e+00,  1.0000e+00,  0.0000e+00)),
            (533.33, Rgb::new( 2.0837e-01,  1.0526e+00, -6.9172e-03)),
            (540.54, Rgb::new( 4.3798e-01,  1.0496e+00, -1.2520e-02)),
            (547.95, Rgb::new( 7.0584e-01,  1.0446e+00, -1.4483e-02)),
            (555.56, Rgb::new( 1.0329e+00,  9.9169e-01, -1.4993e-02)),
            (563.38, Rgb::new( 1.3902e+00,  8.9776e-01, -1.4121e-02)),
            (571.43, Rgb::new( 1.8295e+00,  8.0911e-01, -1.2308e-02)),
            (579.71, Rgb::new( 2.2621e+00,  6.5558e-01, -1.0023e-02)),
            (588.24, Rgb::new( 2.6163e+00,  5.0876e-01, -7.4838e-03)),
            (597.01, Rgb::new( 2.8493e+00,  3.4943e-01, -5.1878e-03)),
            (606.06, Rgb::new( 2.8443e+00,  2.1453e-01, -2.7094e-03)),
            (615.38, Rgb::new( 2.5878e+00,  1.1043e-01, -1.9647e-03)),
            (625.00, Rgb::new( 2.1093e+00,  4.6490e-02, -9.3607e-04)),
            (634.92, Rgb::new( 1.5293e+00,  1.2886e-02, -3.1770e-04)),
            (645.16, Rgb::new( 1.0000e+00,  0.0000e+00,  0.0000e+00)),
            (655.74, Rgb::new( 5.7234e-01, -2.6601e-03,  2.0024e-04)),
            (666.67, Rgb::new( 2.9165e-01, -2.1627e-03,  2.0195e-04)),
            (677.97, Rgb::new( 1.3867e-01, -1.2118e-03,  6.5419e-05)),
            (689.66, Rgb::new( 6.1785e-02, -5.8213e-04,  3.1541e-05)),
            (701.75, Rgb::new( 2.4531e-02, -2.2689e-04,  1.4230e-05)),
            (714.29, Rgb::new( 9.9268e-03, -9.0810e-05,  7.5899e-06)),
            (727.27, Rgb::new( 3.8539e-03, -3.2577e-05,  4.5185e-06)),
        ];
        let frequency = frequency * Rgb::SIZE / N::to_usize();
        table[frequency].clone().1
    }
}

impl Mul<Density> for Rgb {
    type Output = Rgb;

    fn mul(self, rhs: Density) -> Self::Output {
        Rgb::new(self.r * rhs, self.g * rhs, self.b * rhs)
    }
}

impl Add for Rgb {
    type Output = Rgb;

    fn add(self, rhs: Self) -> Self::Output {
        Rgb::new(self.r + rhs.r, self.g + rhs.g, self.b + rhs.b)
    }
}
