pub mod drawing;
pub mod group;
use crate::math::Transform;
// use crate::math::{Mat4, Transform};
use crate::mesh::MeshProvider;
use drawing::Drawing;
use group::Group;
use nalgebra::Matrix4;
// use group::Group;

const ASDF: () = ();

pub struct Canvas<'c> {
    pub(crate) commands: Vec<DrawCommand>,
    // meshes: &'c mut MeshManager,
    // shaders: &'c mut ShaderManager,
    // queue: &'c Queue,
    _asdf: &'c (),
}

impl<'c> Canvas<'c> {
    #[allow(clippy::new_without_default)]
    pub fn new(// meshes: &'c mut MeshManager,
        // shaders: &'c mut ShaderManager,
        // queue: &'c Queue,
    ) -> Self {
        Self {
            commands: vec![],
            // meshes,
            // shaders,
            // queue,
            _asdf: &ASDF,
        }
    }

    fn add_command(&mut self, command: DrawCommand) {
        self.commands.push(command);
    }

    pub fn group<'cref>(
        &'cref mut self,
        group_fn: impl FnOnce(&mut Canvas),
    ) -> Group<'c, 'cref> {
        let mut canv = Canvas::new();
        group_fn(&mut canv);
        let commands = canv.commands;
        Group::new(self, commands)
    }

    pub fn draw<'cref, T: Drawable>(
        &'cref mut self,
        thing: T,
    ) -> Drawing<'c, 'cref> {
        thing.draw(self)
    }
}

pub trait Drawable {
    fn draw<'c, 'cref>(
        &self,
        canvas: &'cref mut Canvas<'c>,
    ) -> Drawing<'c, 'cref>;
}

impl<T: MeshProvider> Drawable for T {
    fn draw<'c, 'cref>(
        &self,
        canvas: &'cref mut Canvas<'c>,
    ) -> Drawing<'c, 'cref> {
        // let meshid = canvas.meshes.get_or_insert(*self);
        Drawing::new(canvas /* , meshid */)
    }
}

#[derive(Clone, Copy)]
pub struct DrawCommand {
    // pub mesh: MeshId,
    // pub shader: ShaderId,
    // pub transform: Mat4,
    pub transform: Matrix4<f32>,
}

impl Transform for DrawCommand {
    fn mat_mut(&mut self) -> &mut Matrix4<f32> {
        &mut self.transform
    }
}
