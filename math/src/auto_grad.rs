use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

pub use num_traits::Float;

use num_traits::{Num, NumCast, One, ToPrimitive, Zero};

#[derive(Clone, Copy)]
pub struct AutoGrad<F: Float, const DIMS: usize> {
    val: F,
    grad: [F; DIMS],
}

impl<F: Float, const DIMS: usize> AutoGrad<F, DIMS> {
    pub fn new(val: F, grad: [F; DIMS]) -> Self {
        Self { val, grad }
    }
    pub fn val(&self) -> F {
        self.val
    }

    pub fn grad(&self) -> [F; DIMS] {
        self.grad
    }
}

impl<F: Float, const DIMS: usize> From<F> for AutoGrad<F, DIMS> {
    fn from(val: F) -> Self {
        Self {
            val,
            grad: [F::zero(); DIMS],
        }
    }
}

impl<F: Float, const DIMS: usize> Neg for AutoGrad<F, DIMS> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            val: -self.val,
            grad: self.grad.map(|g| -g),
        }
    }
}

impl<F: Float, const DIMS: usize> PartialEq for AutoGrad<F, DIMS> {
    fn eq(&self, other: &Self) -> bool {
        self.val.eq(&other.val) && self.grad.eq(&other.grad)
    }
}

impl<F: Float, const DIMS: usize> PartialOrd for AutoGrad<F, DIMS> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(
            self.val
                .partial_cmp(&other.val)?
                .then(self.grad.partial_cmp(&other.grad)?),
        )
    }
}

impl<F: Float, const DIMS: usize> NumCast for AutoGrad<F, DIMS> {
    fn from<T: num_traits::ToPrimitive>(n: T) -> Option<Self> {
        Some(From::from(F::from(n)?))
    }
}

impl<F: Float, const DIMS: usize> ToPrimitive for AutoGrad<F, DIMS> {
    fn to_i64(&self) -> Option<i64> {
        self.val.to_i64()
    }

    fn to_u64(&self) -> Option<u64> {
        self.val.to_u64()
    }
}

impl<F: Float, const DIMS: usize> One for AutoGrad<F, DIMS> {
    fn one() -> Self {
        From::from(F::one())
    }
}
impl<F: Float, const DIMS: usize> Zero for AutoGrad<F, DIMS> {
    fn zero() -> Self {
        From::from(F::zero())
    }

    fn is_zero(&self) -> bool {
        self.val.is_zero()
    }
}
impl<F: Float, const DIMS: usize> Num for AutoGrad<F, DIMS> {
    type FromStrRadixErr = F::FromStrRadixErr;

    fn from_str_radix(
        str: &str,
        radix: u32,
    ) -> Result<Self, Self::FromStrRadixErr> {
        Ok(From::from(F::from_str_radix(str, radix)?))
    }
}
impl<F: Float, const DIMS: usize> Add for AutoGrad<F, DIMS> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut result = [F::zero(); DIMS];
        #[allow(clippy::needless_range_loop)]
        for i in 0..DIMS {
            result[i] = self.grad[i] + rhs.grad[i];
        }
        Self {
            val: self.val + rhs.val,
            grad: result,
        }
    }
}
impl<F: Float, const DIMS: usize> Sub for AutoGrad<F, DIMS> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut result = [F::zero(); DIMS];
        #[allow(clippy::needless_range_loop)]
        for i in 0..DIMS {
            result[i] = self.grad[i] - rhs.grad[i];
        }
        Self {
            val: self.val - rhs.val,
            grad: result,
        }
    }
}
impl<F: Float, const DIMS: usize> Mul for AutoGrad<F, DIMS> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut result = [F::zero(); DIMS];
        #[allow(clippy::needless_range_loop)]
        for i in 0..DIMS {
            result[i] = self.grad[i] * rhs.val + self.val * rhs.grad[i];
        }
        Self {
            val: self.val * rhs.val,
            grad: result,
        }
    }
}
impl<F: Float, const DIMS: usize> Div for AutoGrad<F, DIMS> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let mut result = [F::zero(); DIMS];
        #[allow(clippy::needless_range_loop)]
        for i in 0..DIMS {
            result[i] = (self.grad[i] * rhs.val - self.val * rhs.grad[i])
                / (rhs.val * rhs.val);
        }
        Self {
            val: self.val / rhs.val,
            grad: result,
        }
    }
}

impl<F: Float, const DIMS: usize> Rem for AutoGrad<F, DIMS> {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        let mut result = [F::zero(); DIMS];
        #[allow(clippy::needless_range_loop)]
        // TODO: implement x < 0 case to match % operator
        for i in 0..DIMS {
            if self.val < F::zero() {
                result[i] = -rhs.grad[i] * F::ceil(self.val / rhs.val);
            } else {
                result[i] = -rhs.grad[i] * F::floor(self.val / rhs.val);
            }
        }
        Self {
            val: self.val % rhs.val,
            grad: result,
        }
    }
}

impl<F: Float, const DIMS: usize> Float for AutoGrad<F, DIMS> {
    fn nan() -> Self {
        From::from(F::nan())
    }

    fn infinity() -> Self {
        From::from(F::infinity())
    }

    fn neg_infinity() -> Self {
        From::from(F::neg_infinity())
    }

    fn neg_zero() -> Self {
        From::from(F::neg_zero())
    }

    fn min_value() -> Self {
        From::from(F::min_value())
    }

    fn min_positive_value() -> Self {
        From::from(F::min_positive_value())
    }

    fn max_value() -> Self {
        From::from(F::max_value())
    }

    fn is_nan(self) -> bool {
        self.val.is_nan()
    }

    fn is_infinite(self) -> bool {
        self.val.is_infinite()
    }

    fn is_finite(self) -> bool {
        self.val.is_finite()
    }

    fn is_normal(self) -> bool {
        self.val.is_normal()
    }

    fn classify(self) -> std::num::FpCategory {
        self.val.classify()
    }

    fn floor(self) -> Self {
        From::from(self.val.floor())
    }

    fn ceil(self) -> Self {
        From::from(self.val.ceil())
    }

    fn round(self) -> Self {
        From::from(self.val.round())
    }

    fn trunc(self) -> Self {
        From::from(self.val.round())
    }

    fn fract(self) -> Self {
        Self {
            val: self.val.fract(),
            grad: self.grad,
        }
    }

    fn abs(self) -> Self {
        let mut result = [F::zero(); DIMS];
        #[allow(clippy::needless_range_loop)]
        for i in 0..DIMS {
            result[i] = self.grad[i] * self.val.signum();
        }
        Self {
            val: self.val.abs(),
            grad: result,
        }
    }

    fn signum(self) -> Self {
        From::from(self.val.signum())
    }

    fn is_sign_positive(self) -> bool {
        self.val.is_sign_positive()
    }

    fn is_sign_negative(self) -> bool {
        self.val.is_sign_negative()
    }

    fn mul_add(self, a: Self, b: Self) -> Self {
        todo!()
    }

    fn recip(self) -> Self {
        let mut result = [F::zero(); DIMS];
        #[allow(clippy::needless_range_loop)]
        for i in 0..DIMS {
            result[i] = -self.grad[i] / (self.val * self.val);
        }
        Self {
            val: self.val.recip(),
            grad: result,
        }
    }

    fn powi(self, n: i32) -> Self {
        let mut result = [F::zero(); DIMS];
        #[allow(clippy::needless_range_loop)]
        for i in 0..DIMS {
            result[i] =
                self.grad[i] * F::from(n).unwrap() * self.val.powi(n - 1);
        }
        Self {
            val: self.val.powi(n),
            grad: result,
        }
    }

    // TODO: do real power
    fn powf(self, n: Self) -> Self {
        let mut result = [F::zero(); DIMS];
        #[allow(clippy::needless_range_loop)]
        for i in 0..DIMS {
            result[i] = self.grad[i] * n.val * self.val.powf(n.val - F::one());
        }
        Self {
            val: self.val.powf(n.val),
            grad: result,
        }
    }

    fn sqrt(self) -> Self {
        let mut result = [F::zero(); DIMS];
        #[allow(clippy::needless_range_loop)]
        for i in 0..DIMS {
            result[i] =
                self.grad[i] / ((F::one() + F::one()) * self.val.sqrt());
        }
        Self {
            val: self.val.sqrt(),
            grad: result,
        }
    }

    fn exp(self) -> Self {
        let mut result = [F::zero(); DIMS];
        #[allow(clippy::needless_range_loop)]
        for i in 0..DIMS {
            result[i] = self.grad[i] * self.val.exp();
        }
        Self {
            val: self.val.exp(),
            grad: result,
        }
    }

    fn exp2(self) -> Self {
        todo!()
    }

    fn ln(self) -> Self {
        todo!()
    }

    fn log(self, base: Self) -> Self {
        todo!()
    }

    fn log2(self) -> Self {
        todo!()
    }

    fn log10(self) -> Self {
        todo!()
    }

    fn max(self, other: Self) -> Self {
        todo!()
    }

    fn min(self, other: Self) -> Self {
        todo!()
    }

    fn abs_sub(self, other: Self) -> Self {
        todo!()
    }

    fn cbrt(self) -> Self {
        todo!()
    }

    fn hypot(self, other: Self) -> Self {
        todo!()
    }

    fn sin(self) -> Self {
        let mut result = [F::zero(); DIMS];
        #[allow(clippy::needless_range_loop)]
        for i in 0..DIMS {
            result[i] = self.grad[i] * self.val.cos();
        }
        Self {
            val: self.val.sin(),
            grad: result,
        }
    }

    fn cos(self) -> Self {
        let mut result = [F::zero(); DIMS];
        #[allow(clippy::needless_range_loop)]
        for i in 0..DIMS {
            result[i] = -self.grad[i] * self.val.sin();
        }
        Self {
            val: self.val.cos(),
            grad: result,
        }
    }

    fn tan(self) -> Self {
        todo!()
    }

    fn asin(self) -> Self {
        todo!()
    }

    fn acos(self) -> Self {
        todo!()
    }

    fn atan(self) -> Self {
        todo!()
    }

    fn atan2(self, other: Self) -> Self {
        todo!()
    }

    fn sin_cos(self) -> (Self, Self) {
        todo!()
    }

    fn exp_m1(self) -> Self {
        todo!()
    }

    fn ln_1p(self) -> Self {
        todo!()
    }

    fn sinh(self) -> Self {
        todo!()
    }

    fn cosh(self) -> Self {
        todo!()
    }

    fn tanh(self) -> Self {
        todo!()
    }

    fn asinh(self) -> Self {
        todo!()
    }

    fn acosh(self) -> Self {
        todo!()
    }

    fn atanh(self) -> Self {
        todo!()
    }

    fn integer_decode(self) -> (u64, i16, i8) {
        todo!()
    }
}
