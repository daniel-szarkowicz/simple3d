use std::time::Instant;

use demo::omt::{rand_aabbs, Leaf, RTree as Omt};
use demo::rtree::RTree;
use graphics::app::{App, AppState};
use graphics::canvas::Canvas;
use graphics::geometry::{Box, BoxLines};
use graphics::math::Transform;

fn main() {
    App::run_with(State::new());
}

struct State {
    start: Instant,
    omt: Omt<()>,
    rtree: RTree<()>,
}

impl State {
    fn new() -> Self {
        let leaves: Vec<Leaf<()>> =
            rand_aabbs(10000).into_iter().map(Leaf::new_empty).collect();
        let mut rtree = RTree::new();
        for leaf in &leaves {
            rtree.insert(leaf.aabb, ());
        }
        Self {
            start: Instant::now(),
            omt: Omt::new(leaves),
            rtree,
        }
    }
}

impl AppState for State {
    fn update(&mut self) {}

    fn draw(&self, canvas: &mut Canvas) {
        let height = self.omt.height().max(self.rtree.height() + 1);
        let t = self.start.elapsed().as_secs_f32() / 5.0;
        let max_height = t as usize % height;
        let angle = t.to_radians() / 2.0 * 360.0;
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
        canvas
            .group(|canvas| {
                for (i, (aabb, level)) in self.omt.aabbs().enumerate() {
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
            })
            .rotate_y(angle)
            .translate_x(15.0);
        canvas
            .group(|canvas| {
                for (i, (level, aabb)) in
                    self.rtree.aabbs().into_iter().enumerate()
                {
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
            })
            .rotate_y(angle)
            .translate_x(-15.0);
    }
}
