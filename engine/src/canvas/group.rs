use nalgebra::Matrix4;

use crate::math::Transform;

use super::{Canvas, DrawCommand};

pub struct Group<'c, 'cref> {
    canvas: &'cref mut Canvas<'c>,
    commands: Vec<DrawCommand>,
    transform: Matrix4<f32>,
}

impl<'c, 'cref> Group<'c, 'cref> {
    pub fn new(
        canvas: &'cref mut Canvas<'c>,
        commands: Vec<DrawCommand>,
    ) -> Self {
        Self {
            canvas,
            commands,
            transform: Matrix4::identity(),
        }
    }
}

impl<'c, 'cref> Drop for Group<'c, 'cref> {
    fn drop(&mut self) {
        self.canvas.commands.extend(
            std::mem::take(&mut self.commands)
                .into_iter()
                .map(|cmd| cmd.transform(&self.transform)),
        )
    }
}

impl<'c, 'cref> Transform for Group<'c, 'cref> {
    fn mat_mut(&mut self) -> &mut Matrix4<f32> {
        &mut self.transform
    }
}
