mod transform;
use std::ops::{Mul, MulAssign};

pub use transform::Transform;

type Float = f32;

// #[derive(Debug, Clone, Copy, PartialEq)]
// pub struct Mat4 {
//     data: [[Float; 4]; 4],
// }

// impl Mat4 {
//     pub fn zeros() -> Self {
//         Self {
//             data: [
//                 [0.0, 0.0, 0.0, 0.0],
//                 [0.0, 0.0, 0.0, 0.0],
//                 [0.0, 0.0, 0.0, 0.0],
//                 [0.0, 0.0, 0.0, 0.0],
//             ],
//         }
//     }

//     pub fn identity() -> Self {
//         Self {
//             data: [
//                 [1.0, 0.0, 0.0, 0.0],
//                 [0.0, 1.0, 0.0, 0.0],
//                 [0.0, 0.0, 1.0, 0.0],
//                 [0.0, 0.0, 0.0, 1.0],
//             ],
//         }
//     }

//     pub fn scale(x: Float, y: Float, z: Float) -> Self {
//         Self {
//             data: [
//                 [x, 0.0, 0.0, 0.0],
//                 [0.0, y, 0.0, 0.0],
//                 [0.0, 0.0, z, 0.0],
//                 [0.0, 0.0, 0.0, 1.0],
//             ],
//         }
//     }
// }

// impl Mul for Mat4 {
//     type Output = Mat4;

//     fn mul(self, rhs: Self) -> Self::Output {
//         let mut out = Mat4::zeros();
//         for i in 0..4 {
//             for j in 0..4 {
//                 for k in 0..4 {
//                     out.data[i][j] += self.data[i][k] * rhs.data[k][j];
//                 }
//             }
//         }
//         out
//     }
// }

// impl MulAssign for Mat4 {
//     fn mul_assign(&mut self, rhs: Self) {
//         *self = *self * rhs;
//     }
// }
