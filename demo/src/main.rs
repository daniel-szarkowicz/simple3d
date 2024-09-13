use demo::omt::{rand_aabbs, Leaf, RTree};
use graphics::app::{App, AppState};
use graphics::canvas::Canvas;
use graphics::geometry::{Box, BoxLines};
use graphics::math::Transform;

fn main() {
    App::run_with(State::new());
}

struct State {
    tree: RTree<()>,
}

impl State {
    fn new() -> Self {
        let leaves = rand_aabbs(100).into_iter().map(Leaf::new_empty).collect();
        Self {
            tree: RTree::new(leaves),
        }
    }
}

impl AppState for State {
    fn update(&mut self) {}

    fn draw(&self, canvas: &mut Canvas) {
        for aabb in self.tree.aabbs() {
            let size = aabb.size().map(|f| f as f32);
            let pos = aabb.pos().map(|f| f as f32);
            canvas
                .draw(BoxLines)
                .scale(size[0], size[1], size[2])
                .translate(pos[0], pos[1], pos[2]);
            // canvas.draw(BoxLines);
        }
        // let t = self.start.elapsed().as_secs_f32();
        // let angle = t * 2.0 * std::f32::consts::PI / 5.0;
        // let pos_cos = (angle.cos() + 1.0) / 2.0;
        // let pos_sin = (angle.sin() + 1.0) / 2.0;
        // canvas
        //     .group(|canvas| {
        //         canvas
        //             .draw(Ellipsoid)
        //             .rotate_y(angle)
        //             .translate_x(-2.0)
        //             .color([pos_cos, pos_sin, 0.0]);
        //         canvas
        //             .draw(Box)
        //             .rotate_y(angle)
        //             .translate_x(2.0)
        //             .color([0.0, pos_cos, pos_sin]);
        //     })
        //     .rotate_y(angle / 2.0);
    }
}
