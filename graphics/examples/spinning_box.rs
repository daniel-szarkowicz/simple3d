use std::time::Instant;

use graphics::{app::App, canvas::Canvas, geometry::Box, math::Transform};

fn main() {
    let app = App::new(Instant::now(), |_| (), draw);
    app.run();
}

fn draw(start: &Instant, canvas: &mut Canvas) {
    let t = start.elapsed().as_secs_f32();
    let angle = t.to_radians() * 90.0;

    canvas.draw(Box).color([0.8, 0.3, 0.1]).rotate_y(angle);
}
