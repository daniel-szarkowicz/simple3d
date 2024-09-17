pub mod drawing;
pub mod group;

use crate::math::Transform;
use crate::mesh::{MeshId, MeshKind, MeshManager, MeshProvider};
use drawing::Drawing;
use group::Group;
use nalgebra::Matrix4;

pub struct Canvas<'c> {
    pub(crate) commands: Vec<DrawCommand>,
    meshes: &'c mut MeshManager,
    // shaders: &'c mut ShaderManager,
}

impl<'c> Canvas<'c> {
    #[allow(clippy::new_without_default)]
    pub fn new(
        meshes: &'c mut MeshManager,
        // shaders: &'c mut ShaderManager,
    ) -> Self {
        Self {
            commands: vec![],
            meshes,
            // shaders,
        }
    }

    fn add_command(&mut self, command: DrawCommand) {
        self.commands.push(command);
    }

    pub fn group<'cref, GroupFn: FnOnce(&mut Canvas)>(
        &'cref mut self,
        group_fn: GroupFn,
    ) -> Group<'c, 'cref, GroupFn> {
        Group::new(self, group_fn)
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
        self,
        canvas: &'cref mut Canvas<'c>,
    ) -> Drawing<'c, 'cref>;
}

impl<Provider: MeshProvider> Drawable for Provider {
    fn draw<'c, 'cref>(
        self,
        canvas: &'cref mut Canvas<'c>,
    ) -> Drawing<'c, 'cref> {
        let mesh = Provider::Kind::get_or_insert(canvas.meshes, self);
        Drawing::new(canvas, mesh)
    }
}

#[derive(Clone)]
pub struct DrawCommand {
    pub mesh_id: MeshId,
    // pub shader: ShaderId,
    pub transform: Matrix4<f32>,
    pub color: [f32; 3],
}

impl Transform for DrawCommand {
    fn mat_mut(&mut self) -> &mut Matrix4<f32> {
        &mut self.transform
    }
}
