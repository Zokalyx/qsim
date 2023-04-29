use std::{ops::{Add, Sub, Mul, Div}, iter::Sum};
use std::f32::consts::E;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Complex {
    real: f32,
    imag: f32,
}
impl Complex {
    pub fn zero() -> Self {
        Self { real: 0.0, imag: 0.0 }
    }

    pub fn new(real: f32, imag: f32) -> Self {
        Self { real, imag }
    }

    pub fn from_polar_radians(modulus: f32, angle: f32) -> Self {
        Self {
            real: modulus * angle.cos(),
            imag: modulus * angle.sin()
        }
    }

    pub fn modulus_squared(&self) -> f32 {
        self.real * self.real + self.imag * self.imag
    }

    pub fn modulus(&self) -> f32 {
        self.modulus_squared().sqrt()
    }

    pub fn conjugate(&self) -> Complex {
        Self {
            real: self.real,
            imag: -self.imag
        }
    }

    pub fn angle_radians(&self) -> f32 {
        self.imag.atan2(self.real)
    }

    pub fn powf(&self, exponent: Complex) -> Complex {
        if self.modulus_squared() == 0.0 {
            return Complex::zero()
        }
        let angle = self.angle_radians() * exponent.real + self.modulus().ln() * exponent.imag;
        let modulus = self.modulus().powf(exponent.real) * E.powf(-self.angle_radians() * exponent.imag);
        Self::from_polar_radians(modulus, angle)
    }

    pub fn is_zero(&self) -> bool {
        self.real == 0.0 && self.imag == 0.0
    }
}
impl Add for Complex {
    type Output = Complex;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            real: self.real + rhs.real,
            imag: self.imag + rhs.imag,
        }
    }
}
impl Sum for Complex {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Complex::zero(), |acc: Complex, x| acc + x)
    }
}
impl Sub for Complex {
    type Output = Complex;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            real: self.real - rhs.real,
            imag: self.imag - rhs.imag,
        }
    }
}
impl Mul<Complex> for Complex {
    type Output = Complex;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            real: self.real * rhs.real - self.imag * rhs.imag,
            imag: self.real * rhs.imag + self.imag * rhs.real,
        }
    }
}
impl Mul<f32> for Complex {
    type Output = Complex;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            real: self.real * rhs,
            imag: self.real * rhs,
        }
    }
}
impl Div<Complex> for Complex {
    type Output = Complex;

    fn div(self, rhs: Self) -> Self::Output {
        let conjugate = rhs.conjugate();
        (self * conjugate) / conjugate.modulus_squared()
    }
}
impl Div<f32> for Complex {
    type Output = Complex;

    fn div(self, rhs: f32) -> Self::Output {
        Self {
            real: self.real / rhs,
            imag: self.real / rhs
        }
    }
}
impl From<f32> for Complex {
    fn from(f: f32) -> Self {
        Self {
            real: f,
            imag: 0.0
        }
    }
}
