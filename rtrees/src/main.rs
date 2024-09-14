use std::collections::{HashMap, HashSet};
// #![feature(test)]
use std::time::{Duration, Instant};

use graphics::app::{App, AppState};
use graphics::canvas::{Canvas, Drawable};
use graphics::geometry::{Box, BoxLines};
use graphics::math::Transform;
mod omt;
mod rtree;
use omt::{rand_aabb, rand_aabbs, Leaf, QueryData, RTree as Omt, AABB};
use rtree::RTree;

fn main() {
    App::run_with(State::new());
}

struct State {
    prev: Instant,
    omt: Omt<()>,
    query_aabb: AABB,
}

const BOUNDS: AABB = AABB {
    min: [-10.0, -10.0, -10.0],
    max: [10.0, 10.0, 10.0],
};

impl State {
    fn new() -> Self {
        let leaves: Vec<Leaf<()>> = rand_aabbs(10000, BOUNDS, 0.01..0.1)
            .into_iter()
            .map(Leaf::new_empty)
            .collect();
        Self {
            prev: Instant::now(),
            omt: Omt::new(leaves),
            query_aabb: rand_aabb(BOUNDS, 0.02..0.2),
        }
    }
}

impl AppState for State {
    fn update(&mut self) {
        let delay = Duration::from_secs(5);
        if self.prev.elapsed() > delay {
            self.prev += delay;
            self.query_aabb = rand_aabb(BOUNDS, 0.02..0.2);
        } else {
            let pos = self.query_aabb.pos();
            self.query_aabb.min[0] -= pos[0] / 100.0;
            self.query_aabb.min[1] -= pos[1] / 100.0;
            self.query_aabb.min[2] -= pos[2] / 100.0;
            self.query_aabb.max[0] -= pos[0] / 100.0;
            self.query_aabb.max[1] -= pos[1] / 100.0;
            self.query_aabb.max[2] -= pos[2] / 100.0;
        }
    }

    fn draw(&self, canvas: &mut Canvas) {
        let query_aabb = self.query_aabb;
        let size = query_aabb.size().map(|f| f as f32);
        let pos = query_aabb.pos().map(|f| f as f32);
        canvas
            .draw(BoxLines)
            .scale(size[0], size[1], size[2])
            .translate(pos[0], pos[1], pos[2])
            .color([1.0, 0.0, 0.0]);
        let query: Vec<_> = self.omt.query(query_aabb).collect();
        let mut levels = HashMap::new();
        for q in &query {
            if let QueryData::Node { depth } = q.data {
                *levels.entry(depth).or_insert(0usize) += 1;
            }
        }
        let levels: HashSet<_> = levels
            .into_iter()
            .filter(|(_, i)| i >= &2)
            .map(|(l, _)| l)
            .collect();
        for item in query {
            let size = item.aabb.size().map(|f| f as f32);
            let pos = item.aabb.pos().map(|f| f as f32);
            let drawing = if let QueryData::Node { depth } = item.data {
                if levels.contains(&depth) {
                    canvas.draw(BoxLines).color([0.0, 0.0, 0.0])
                } else {
                    canvas.draw(BoxLines)
                }
            } else {
                canvas.draw(Box)
            };
            drawing
                .scale(size[0], size[1], size[2])
                .translate(pos[0], pos[1], pos[2]);
        }
    }

    // fn draw(&self, canvas: &mut Canvas) {
    //     let height = self.omt.height().max(self.rtree.height());
    //     let t = self.start.elapsed().as_secs_f32() / 5.0;
    //     let max_height = t as usize % height;
    //     let angle = t.to_radians() / 2.0 * 360.0;
    //     let colors = [
    //         [1.0, 0.0, 0.0],
    //         [0.75, 0.25, 0.0],
    //         [0.5, 0.5, 0.0],
    //         [0.25, 0.75, 0.0],
    //         [0.0, 1.0, 0.0],
    //         [0.0, 0.75, 0.25],
    //         [0.0, 0.5, 0.5],
    //         [0.0, 0.25, 0.75],
    //         [0.0, 0.0, 1.0],
    //         [0.25, 0.0, 0.75],
    //         [0.5, 0.0, 0.5],
    //         [0.75, 0.0, 0.25],
    //     ];
    //     canvas
    //         .group(|canvas| {
    //             for (i, (aabb, level)) in self.omt.aabbs().enumerate() {
    //                 let size = aabb.size().map(|f| f as f32);
    //                 let pos = aabb.pos().map(|f| f as f32);
    //                 let drawing = match level.cmp(&max_height) {
    //                     std::cmp::Ordering::Less => canvas.draw(BoxLines),
    //                     std::cmp::Ordering::Equal => canvas.draw(Box),
    //                     std::cmp::Ordering::Greater => continue,
    //                 };
    //                 drawing
    //                     .color(colors[i % colors.len()])
    //                     .scale(size[0], size[1], size[2])
    //                     .translate(pos[0], pos[1], pos[2]);
    //             }
    //         })
    //         .rotate_y(angle)
    //         .translate_x(15.0)
    //         .translate_z(40.0)
    //         .translate_y(-20.0);
    //     canvas
    //         .group(|canvas| {
    //             for (i, (level, aabb)) in
    //                 self.rtree.aabbs().into_iter().enumerate()
    //             {
    //                 let size = aabb.size().map(|f| f as f32);
    //                 let pos = aabb.pos().map(|f| f as f32);
    //                 let drawing = match level.cmp(&max_height) {
    //                     std::cmp::Ordering::Less => canvas.draw(BoxLines),
    //                     std::cmp::Ordering::Equal => canvas.draw(Box),
    //                     std::cmp::Ordering::Greater => continue,
    //                 };
    //                 drawing
    //                     .color(colors[i % colors.len()])
    //                     .scale(size[0], size[1], size[2])
    //                     .translate(pos[0], pos[1], pos[2]);
    //             }
    //         })
    //         .rotate_y(angle)
    //         .translate_x(-15.0)
    //         .translate_z(40.0)
    //         .translate_y(-20.0);
    // }
}
