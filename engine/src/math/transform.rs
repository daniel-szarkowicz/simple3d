use nalgebra::{
    Matrix4, Rotation3, Scale3, Translation3, Unit, UnitVector3, Vector3,
};

use super::Float;

pub trait Transform: Sized {
    fn mat_mut(&mut self) -> &mut Matrix4<Float>;

    fn transform(mut self, transform: &Matrix4<Float>) -> Self {
        *self.mat_mut() = transform * *self.mat_mut();
        self
    }

    fn scale(self, x: Float, y: Float, z: Float) -> Self {
        self.transform(&Scale3::new(x, y, z).to_homogeneous())
    }

    fn scale_x(self, x: Float) -> Self {
        self.scale(x, 1.0, 1.0)
    }

    fn scale_y(self, y: Float) -> Self {
        self.scale(1.0, y, 1.0)
    }

    fn scale_z(self, z: Float) -> Self {
        self.scale(1.0, 1.0, z)
    }

    fn translate(self, x: Float, y: Float, z: Float) -> Self {
        self.transform(&Translation3::new(x, y, z).to_homogeneous())
    }

    fn translate_x(self, x: Float) -> Self {
        self.translate(x, 0.0, 0.0)
    }

    fn translate_y(self, y: Float) -> Self {
        self.translate(0.0, y, 0.0)
    }

    fn translate_z(self, z: Float) -> Self {
        self.translate(0.0, 0.0, z)
    }

    fn rotate_x(self, angle: Float) -> Self {
        self.transform(
            &Rotation3::from_axis_angle(
                &Unit::new_normalize(Vector3::x()),
                angle,
            )
            .to_homogeneous(),
        )
    }

    fn rotate_y(self, angle: Float) -> Self {
        self.transform(
            &Rotation3::from_axis_angle(
                &Unit::new_normalize(Vector3::y()),
                angle,
            )
            .to_homogeneous(),
        )
    }

    fn rotate_z(self, angle: Float) -> Self {
        self.transform(
            &Rotation3::from_axis_angle(
                &Unit::new_normalize(Vector3::z()),
                angle,
            )
            .to_homogeneous(),
        )
    }
}

impl<T: Transform> Transform for &mut T {
    fn mat_mut(&mut self) -> &mut Matrix4<Float> {
        (**self).mat_mut()
    }
}
