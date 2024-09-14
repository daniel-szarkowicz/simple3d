use nalgebra::Matrix4;

use crate::math::Transform;

use super::Canvas;

pub struct Group<'c, 'cref, GroupFn: FnOnce(&mut Canvas)> {
    canvas: &'cref mut Canvas<'c>,
    group_fn: Option<GroupFn>,
    transform: Matrix4<f32>,
}

impl<'c, 'cref, GroupFn: FnOnce(&mut Canvas)> Group<'c, 'cref, GroupFn> {
    pub fn new(canvas: &'cref mut Canvas<'c>, group_fn: GroupFn) -> Self {
        Self {
            canvas,
            group_fn: Some(group_fn),
            transform: Matrix4::identity(),
        }
    }
}

impl<'c, 'cref, GroupFn: FnOnce(&mut Canvas)> Drop
    for Group<'c, 'cref, GroupFn>
{
    fn drop(&mut self) {
        let mut canv = Canvas::new(self.canvas.meshes);
        (self.group_fn.take().unwrap())(&mut canv);
        self.canvas.commands.extend(
            canv.commands
                .into_iter()
                .map(|cmd| cmd.transform(&self.transform)),
        )
    }
}

impl<'c, 'cref, GroupFn: FnOnce(&mut Canvas)> Transform
    for Group<'c, 'cref, GroupFn>
{
    fn mat_mut(&mut self) -> &mut Matrix4<f32> {
        &mut self.transform
    }
}
