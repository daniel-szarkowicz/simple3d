use winit::application::ApplicationHandler;

use crate::{canvas::Canvas, context::Context};

type Update<State> = fn(&mut State) -> ();
type Draw<State> = fn(&mut Canvas, &State) -> ();

pub struct WinitApp<State> {
    state: State,
    update: Update<State>,
    draw: Draw<State>,
    context: Option<Context>,
}

impl<State> WinitApp<State> {
    pub fn new(state: State, update: Update<State>, draw: Draw<State>) -> Self {
        Self {
            state,
            update,
            draw,
            context: None,
        }
    }
}

impl<State> ApplicationHandler for WinitApp<State> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.context = Some(Context::new(event_loop));
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let context = self.context.as_mut().unwrap();
        match &event {
            winit::event::WindowEvent::Resized(new_size) => {
                context.resize(new_size);
            }
            winit::event::WindowEvent::CloseRequested => event_loop.exit(),
            winit::event::WindowEvent::RedrawRequested => {
                context.update_camera(1.0 / 60.0);
                (self.update)(&mut self.state);
                let mut canvas = context.create_canvas();
                (self.draw)(&mut canvas, &self.state);
                context.render(canvas.commands);
            }
            _ => (),
        }
        context.event(&winit::event::Event::<()>::WindowEvent {
            window_id,
            event,
        });
    }

    fn device_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        self.context.as_mut().unwrap().event(
            &winit::event::Event::<()>::DeviceEvent { device_id, event },
        );
    }
}
