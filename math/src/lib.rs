pub mod auto_grad;
use std::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign,
};

use num_traits::{Float, Zero};

#[derive(Clone, Copy)]
pub struct AutoDiff<F> {
    val: F,
    diff: F,
}

// Automatic differentiation using Dual Numbers
// stolen from: https://en.wikipedia.org/wiki/Automatic_differentiation#Automatic_differentiation_using_dual_numbers
impl<F: Copy> AutoDiff<F> {
    pub fn new(val: F, diff: F) -> Self {
        Self { val, diff }
    }

    pub fn val(&self) -> F {
        self.val
    }

    pub fn diff(&self) -> F {
        self.diff
    }
}

impl<F: Add> Add for AutoDiff<F> {
    type Output = AutoDiff<F::Output>;

    fn add(self, rhs: Self) -> Self::Output {
        AutoDiff {
            val: self.val + rhs.val,
            diff: self.diff + rhs.diff,
        }
    }
}

impl<F: Float> Add<F> for AutoDiff<F> {
    type Output = <AutoDiff<F> as Add>::Output;

    fn add(self, rhs: F) -> Self::Output {
        self.add(AutoDiff {
            val: rhs,
            diff: F::zero(),
        })
    }
}

impl<F: AddAssign> AddAssign for AutoDiff<F> {
    fn add_assign(&mut self, rhs: Self) {
        self.val += rhs.val;
        self.diff += rhs.diff;
    }
}

impl<F: Sub> Sub for AutoDiff<F> {
    type Output = AutoDiff<F::Output>;

    fn sub(self, rhs: Self) -> Self::Output {
        AutoDiff {
            val: self.val - rhs.val,
            diff: self.diff - rhs.diff,
        }
    }
}

impl<F: SubAssign> SubAssign for AutoDiff<F> {
    fn sub_assign(&mut self, rhs: Self) {
        self.val -= rhs.val;
        self.diff -= rhs.diff;
    }
}

impl<F: Mul + Copy> Mul for AutoDiff<F>
where
    F::Output: Add<Output = F::Output>,
{
    type Output = AutoDiff<F::Output>;

    fn mul(self, rhs: Self) -> Self::Output {
        AutoDiff {
            val: self.val * rhs.val,
            diff: self.diff * rhs.val + self.val * rhs.diff,
        }
    }
}

impl<F: Float> Mul<F> for AutoDiff<F> {
    type Output = <AutoDiff<F> as Mul>::Output;

    fn mul(self, rhs: F) -> Self::Output {
        self.mul(AutoDiff {
            val: rhs,
            diff: F::zero(),
        })
    }
}

impl<F> MulAssign for AutoDiff<F>
where
    F: MulAssign + Mul + Copy + AddAssign<<F as Mul>::Output>,
{
    fn mul_assign(&mut self, rhs: Self) {
        self.diff *= rhs.val;
        self.diff += self.val * rhs.diff;
        self.val *= rhs.diff;
    }
}

impl<F> Div for AutoDiff<F>
where
    F: Div + Mul + Copy,
    <F as Mul>::Output: Sub,
    <<F as Mul>::Output as Sub>::Output:
        Div<<F as Mul>::Output, Output = <F as Div>::Output>,
{
    type Output = AutoDiff<<F as Div>::Output>;

    fn div(self, rhs: Self) -> Self::Output {
        AutoDiff {
            val: self.val / rhs.val,
            diff: (self.diff * rhs.val - self.val * rhs.diff)
                / (rhs.val * rhs.val),
        }
    }
}

impl<F: Float> Div<F> for AutoDiff<F> {
    type Output = <AutoDiff<F> as Div>::Output;

    fn div(self, rhs: F) -> Self::Output {
        self.div(AutoDiff {
            val: rhs,
            diff: F::zero(),
        })
    }
}

impl<F> DivAssign for AutoDiff<F>
where
    F: DivAssign
        + DivAssign<<F as Mul>::Output>
        + MulAssign
        + Mul
        + Copy
        + SubAssign<<F as Mul>::Output>,
{
    fn div_assign(&mut self, rhs: Self) {
        self.diff *= rhs.val;
        self.diff -= self.val * rhs.diff;
        self.diff /= rhs.val * rhs.val;
        self.val /= rhs.val;
    }
}

impl<F: Float + MulAssign> AutoDiff<F> {
    pub fn sin(mut self) -> Self {
        self.diff *= self.val.cos();
        self.val = self.val.sin();
        self
    }

    pub fn cos(mut self) -> Self {
        self.diff *= -self.val.sin();
        self.val = self.val.cos();
        self
    }

    pub fn exp(mut self) -> Self {
        self.val = self.val.exp();
        self.diff *= self.val;
        self
    }

    pub fn pow(mut self, n: F) -> Self {
        self.diff *= n * self.val.powf(n - F::one());
        self.val = self.val.powf(n);
        self
    }

    pub fn abs(mut self) -> Self {
        self.diff *= self.val.signum();
        self.val = self.val.abs();
        self
    }
}
impl<F: Float + DivAssign> AutoDiff<F> {
    pub fn ln(mut self) -> Self {
        self.diff /= self.val;
        self.val = self.val.ln();
        self
    }

    pub fn log(mut self, base: F) -> Self {
        self.diff /= self.val * base.ln();
        self.val = self.val.log(base);
        self
    }
}
