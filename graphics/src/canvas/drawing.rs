use std::sync::Arc;

use nalgebra::Matrix4;

// use crate::{
//     context::MeshId,
//     // math::{Mat4, Transform},
//     shader::DefaultShader,
// };

use crate::{math::Transform, mesh::MeshBuffers};

use super::{Canvas, DrawCommand};

pub struct Drawing<'c, 'cref> {
    canvas: &'cref mut Canvas<'c>,
    command: DrawCommand,
}

impl<'c, 'cref> Drawing<'c, 'cref> {
    pub fn new(canvas: &'cref mut Canvas<'c>, mesh: Arc<MeshBuffers>) -> Self {
        // let shader = canvas.shaders.get_or_insert(DefaultShader);
        Self {
            canvas,
            command: DrawCommand {
                mesh,
                // shader,
                transform: Matrix4::identity(),
                color: [1.0, 1.0, 1.0],
            },
        }
    }

    pub fn color(mut self, color: [f32; 3]) -> Self {
        self.command.color = color;
        self
    }

    pub fn finish(self) {}
}

impl<'c, 'cref> Drop for Drawing<'c, 'cref> {
    fn drop(&mut self) {
        self.canvas.add_command(self.command.clone())
    }
}

impl<'c, 'cref> Transform for Drawing<'c, 'cref> {
    #[inline(always)]
    fn mat_mut(&mut self) -> &mut Matrix4<f32> {
        &mut self.command.transform
    }
}
