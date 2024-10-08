use nalgebra::{Matrix4, Perspective3, Point3, Rotation3, Vector3};
use winit::event::{DeviceEvent, Event, KeyEvent, WindowEvent};
use winit::keyboard::{Key, NamedKey};

// use crate::ray::Ray;

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone)]
pub struct FirstPersonCamera {
    position: Point3<f32>,
    forwards: bool,
    backwards: bool,
    left: bool,
    right: bool,
    up: bool,
    down: bool,
    fast: bool,
    focus: bool,
    aspect: f32,
    yaw: f32,
    pitch: f32,
    slow_speed: f32,
    fast_speed: f32,
    near: f32,
    far: f32,
}

impl FirstPersonCamera {
    const UP: Vector3<f32> = Vector3::new(0.0, 1.0, 0.0);
    const FORWARD: Vector3<f32> = Vector3::new(0.0, 0.0, 1.0);
    const LEFT: Vector3<f32> = Vector3::new(1.0, 0.0, 0.0);

    #[must_use]
    pub const fn position(&self) -> Point3<f32> {
        self.position
    }

    fn yaw_rotation(&self) -> Rotation3<f32> {
        Rotation3::new(Self::UP * self.yaw.to_radians())
    }

    fn pitch_rotation(&self) -> Rotation3<f32> {
        Rotation3::new(Self::LEFT * self.pitch.to_radians())
    }

    #[must_use]
    pub fn look_direction(&self) -> Vector3<f32> {
        self.yaw_rotation() * self.pitch_rotation() * Self::FORWARD
    }

    fn look_at(&self) -> Point3<f32> {
        self.position + self.look_direction()
    }

    pub fn move_facing(&mut self, direction: Vector3<f32>) {
        self.position += self.yaw_rotation() * direction;
    }

    #[must_use]
    pub fn view_proj(&self) -> Matrix4<f32> {
        self.partial_view_proj(0.0, 1.0)
    }

    /// # Panics
    /// If the invariant `0.0 <= start < end <= 1.0` is not satisfied.
    #[must_use]
    pub fn partial_view_proj(&self, start: f32, end: f32) -> Matrix4<f32> {
        assert!(0.0 <= start && start < end && end <= 1.0);
        Perspective3::new(
            self.aspect,
            60.0f32.to_radians(),
            (1.0 - start).mul_add(self.near, start * self.far),
            (1.0 - end).mul_add(self.near, end * self.far),
        )
        .to_homogeneous()
            * Matrix4::look_at_rh(&self.position, &self.look_at(), &Self::UP)
    }

    #[rustfmt::skip]
    pub fn update(&mut self, delta: f32) {
        if !self.focus {
            return;
        }
        let mut dir = Vector3::zeros();
        if self.forwards  { dir += Self::FORWARD; }
        if self.backwards { dir -= Self::FORWARD; }
        if self.left      { dir += Self::LEFT;    }
        if self.right     { dir -= Self::LEFT;    }
        if self.up        { dir += Self::UP;      }
        if self.down      { dir -= Self::UP;      }
        let speed = if self.fast {
            self.fast_speed
        } else {
            self.slow_speed
        };
        if dir.magnitude() != 0.0 {
            self.move_facing(dir.normalize() * delta * speed);
        }
    }

    #[must_use]
    pub const fn focus(&self) -> bool {
        self.focus
    }

    pub fn event<T>(&mut self, event: &Event<T>) -> bool {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(size) => {
                    self.aspect = size.width as f32 / size.height as f32;
                    false
                }
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            logical_key, state, ..
                        },
                    ..
                } => match logical_key {
                    Key::Character(ch) => match ch.as_str() {
                        "w" | "W" => {
                            self.forwards = state.is_pressed();
                            self.focus
                        }
                        "s" | "S" => {
                            self.backwards = state.is_pressed();
                            self.focus
                        }
                        "a" | "A" => {
                            self.left = state.is_pressed();
                            self.focus
                        }
                        "d" | "D" => {
                            self.right = state.is_pressed();
                            self.focus
                        }
                        _ => false,
                    },
                    Key::Named(key) => match key {
                        NamedKey::Shift if self.focus => {
                            self.down = state.is_pressed();
                            true
                        }
                        NamedKey::Space if self.focus => {
                            self.up = state.is_pressed();
                            true
                        }
                        NamedKey::Control if self.focus => {
                            self.fast = state.is_pressed();
                            true
                        }
                        NamedKey::Escape if state.is_pressed() => {
                            self.focus = !self.focus;
                            true
                        }
                        _ => false,
                    },
                    _ => false,
                },
                _ => false,
            },
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } if self.focus => {
                self.yaw -= delta.0 as f32 * 0.15;
                self.pitch = (delta.1 as f32)
                    .mul_add(0.15, self.pitch)
                    .clamp(-89.9, 89.9);
                false
            }
            _ => false,
        }
    }

    // #[must_use]
    // pub fn get_ray(&self) -> Ray {
    //     Ray {
    //         start: self.position.cast::<f64>(),
    //         direction: self.look_direction().cast::<f64>(),
    //     }
    // }
}

impl Default for FirstPersonCamera {
    fn default() -> Self {
        Self {
            position: Point3::new(0.0, 0.0, 0.0),
            yaw: 0.0,
            pitch: 0.0,
            forwards: false,
            backwards: false,
            left: false,
            right: false,
            fast: false,
            focus: false,
            up: false,
            down: false,
            aspect: 1.0,
            slow_speed: 1.0,
            fast_speed: 2.0,
            near: 0.01,
            far: 1000.0,
        }
    }
}
