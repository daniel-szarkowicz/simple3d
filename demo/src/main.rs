use std::time::Instant;

use demo::omt::{rand_aabbs, Leaf, RTree};
use graphics::app::{App, AppState};
use graphics::canvas::Canvas;
use graphics::geometry::{Box, BoxLines};
use graphics::math::Transform;

fn main() {
    App::run_with(State::new());
}

struct State {
    start: Instant,
    tree: RTree<()>,
}

impl State {
    fn new() -> Self {
        let leaves =
            rand_aabbs(10000).into_iter().map(Leaf::new_empty).collect();
        Self {
            start: Instant::now(),
            tree: RTree::new(leaves),
        }
    }
}

impl AppState for State {
    fn update(&mut self) {}

    fn draw(&self, canvas: &mut Canvas) {
        let height = self.tree.height();
        let max_height =
            (self.start.elapsed().as_secs() / 10) as usize % height;
        let colors = [
            [1.0, 0.0, 0.0],
            [0.75, 0.25, 0.0],
            [0.5, 0.5, 0.0],
            [0.25, 0.75, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.75, 0.25],
            [0.0, 0.5, 0.5],
            [0.0, 0.25, 0.75],
            [0.0, 0.0, 1.0],
            [0.25, 0.0, 0.75],
            [0.5, 0.0, 0.5],
            [0.75, 0.0, 0.25],
        ];
        for (i, (aabb, level)) in self.tree.aabbs().enumerate() {
            let size = aabb.size().map(|f| f as f32);
            let pos = aabb.pos().map(|f| f as f32);
            let drawing = match level.cmp(&max_height) {
                std::cmp::Ordering::Less => canvas.draw(BoxLines),
                std::cmp::Ordering::Equal => canvas.draw(Box),
                std::cmp::Ordering::Greater => continue,
            };
            drawing
                .color(colors[i % colors.len()])
                .scale(size[0], size[1], size[2])
                .translate(pos[0], pos[1], pos[2]);
        }
    }
}
