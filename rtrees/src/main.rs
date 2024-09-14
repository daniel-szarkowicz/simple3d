// #![feature(test)]
use std::time::Instant;

use graphics::app::{App, AppState};
use graphics::canvas::Canvas;
use graphics::geometry::{Box, BoxLines};
use graphics::math::Transform;
mod omt;
mod rtree;
use omt::{rand_aabbs, Leaf, RTree as Omt};
use rtree::RTree;

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
        let height = self.omt.height().max(self.rtree.height());
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
            .translate_x(15.0)
            .translate_z(40.0)
            .translate_y(-20.0);
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
            .translate_x(-15.0)
            .translate_z(40.0)
            .translate_y(-20.0);
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     extern crate test;
//     use test::Bencher;

//     const AABBS: usize = 100_000;

//     #[bench]
//     fn omt(b: &mut Bencher) {
//         let leaves: Vec<Leaf<()>> =
//             rand_aabbs(AABBS).into_iter().map(Leaf::new_empty).collect();
//         b.iter(|| {
//             let omt = Omt::new(leaves.clone());
//             test::black_box(&omt);
//         });
//     }

//     #[bench]
//     fn rtree(b: &mut Bencher) {
//         let aabbs = rand_aabbs(AABBS);
//         b.iter(|| {
//             let mut rtree = RTree::new();
//             for aabb in &aabbs {
//                 rtree.insert(*aabb, ());
//             }
//             test::black_box(&rtree);
//         });
//     }
// }
